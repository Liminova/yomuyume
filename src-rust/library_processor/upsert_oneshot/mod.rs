mod handle_as_archive;
mod handle_as_directory;

use std::collections::HashMap;
use std::fs::DirEntry;
use std::{path::PathBuf, sync::Arc};

use chrono::{DateTime, Utc};
use futures_util::future::join_all;
use tokio::sync::RwLock;

use crate::types::absolute_path::ToAbsolute;
use crate::{
    app_state::AppState,
    library_processor::{
        upsert_category::upsert_category,
        upsert_oneshot::{
            handle_as_archive::handle_title_as_archive,
            handle_as_directory::handle_title_as_directory,
        },
        upsert_tags::upsert_tags,
        PageInTitle, UpsertTitleErr,
    },
    types::{absolute_path::AbsolutePath, comic_info::ComicInfo, TitleID},
    utils::archive_file::ItemInArchive,
};

#[derive(Debug)]
pub enum OneshotType {
    Archive(Vec<ItemInArchive>),
    Directory(Vec<DirEntry>),
}

#[derive(Debug)]
struct TitleHandlerOk {
    comicinfo: ComicInfo,
    title_last_modified: Option<DateTime<Utc>>,
    pages_in_title: Vec<PageInTitle>,

    cover_path: Option<String>,
    cover_blurhash: Option<String>,
    cover_width: Option<i32>,
    cover_height: Option<i32>,

    is_dir: bool,
}

/// upsert a oneshot to the database and return its ID
pub async fn upsert_oneshot(
    app_state: Arc<AppState>,
    title_path: PathBuf,
    oneshot_type: OneshotType,
    parent_path: Option<PathBuf>,
    category_path_to_id: Arc<RwLock<HashMap<AbsolutePath, i64>>>,
    nomedia_support: bool,
) -> Result<TitleID, UpsertTitleErr> {
    let title_path = title_path.to_absolute(None)?;

    let title = match oneshot_type {
        OneshotType::Archive(files_in_archive) => {
            handle_title_as_archive(&title_path, files_in_archive, nomedia_support)?
        }
        OneshotType::Directory(sub_entries) => {
            handle_title_as_directory(&title_path, &sub_entries, nomedia_support)?
        }
    };

    let mut txn = app_state
        .pool
        .begin()
        .await
        .map_err(UpsertTitleErr::TransactionBegin)?;

    let category_id = upsert_category(
        &app_state,
        parent_path.as_ref(),
        &category_path_to_id,
        &mut *txn,
    )
    .await?;

    // upsert title and get its id
    let title_id = sqlx::query!(
        "INSERT INTO titles
            (id, title, category_id, author, description, release, path, is_dir,
            is_series, cover_path, cover_blurhash, cover_width, cover_height,
            date_updated)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, FALSE, $9, $10, $11, $12, $13)
        ON CONFLICT (path)
        DO UPDATE SET
            title = EXCLUDED.title,
            category_id = EXCLUDED.category_id,
            author = EXCLUDED.author,
            description = EXCLUDED.description,
            release = EXCLUDED.release,
            path = EXCLUDED.path,
            is_dir = EXCLUDED.is_dir,
            is_series = FALSE,
            cover_path = EXCLUDED.cover_path,
            cover_blurhash = EXCLUDED.cover_blurhash,
            cover_width = EXCLUDED.cover_width,
            cover_height = EXCLUDED.cover_height,
            date_updated = EXCLUDED.date_updated
        RETURNING id",
        app_state
            .id_generator
            .snowflake()
            .await
            .map_err(UpsertTitleErr::GenTitleID)?,
        title.comicinfo.title,
        category_id,
        title.comicinfo.penciller.as_ref(),
        title.comicinfo.summary.as_ref(),
        title.comicinfo.get_release(),
        title_path
            .to_relative(Some(&app_state.config.library_path))?
            .to_string_lossy()
            .to_string(),
        title.is_dir,
        title.cover_path,
        title.cover_blurhash,
        title.cover_width,
        title.cover_height,
        title.title_last_modified,
    )
    .fetch_one(&mut *txn)
    .await
    .map_err(UpsertTitleErr::UpsertOneShot)?
    .id;

    upsert_tags(&app_state, &title.comicinfo, &title_id, &mut *txn).await?;

    '_upsert_pages: {
        let page_count = title.pages_in_title.len();

        let page_ids = join_all((0..page_count).map(|_| app_state.id_generator.snowflake()))
            .await
            .into_iter()
            .collect::<Result<Vec<i64>, _>>()
            .map_err(UpsertTitleErr::GenPageIDs)?;

        let mut page_paths = Vec::with_capacity(page_count);
        let mut page_filesizes = Vec::with_capacity(page_count);
        let mut page_descriptions = Vec::with_capacity(page_count);

        for page in &title.pages_in_title {
            page_paths.push(page.path.clone());
            page_filesizes.push(page.size.unwrap_or_default());
            page_descriptions.push(
                title
                    .comicinfo
                    .get_page_description(&page.path)
                    .unwrap_or_default(),
            );
        }

        sqlx::query!(
            "WITH _ AS (
            INSERT INTO oneshots_pages (id, title_id, path, filesize, description)
            SELECT id, $1, path, NULLIF(filesize, 0), NULLIF(description, '')
                FROM UNNEST($2::bigint[], $3::text[], $4::bigint[], $5::text[])
                AS t(id, path, filesize, description)
            ON CONFLICT (title_id, path) DO UPDATE
                SET description = EXCLUDED.description
            )
            DELETE FROM oneshots_pages WHERE title_id = $1
                AND path NOT IN (SELECT UNNEST($3::text[]))",
            title_id,
            &page_ids,
            &page_paths,
            &page_filesizes,
            &page_descriptions
        )
        .execute(&mut *txn)
        .await
        .map_err(UpsertTitleErr::UpsertPages)?;
    }

    txn.commit()
        .await
        .map_err(UpsertTitleErr::TransactionCommit)?;

    Ok(title_id)
}
