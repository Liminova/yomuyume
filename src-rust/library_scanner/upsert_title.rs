// NEW AND IMPROVED

use std::{path::PathBuf, sync::Arc};

use anyhow::{anyhow, Context, Result};
use chrono::Timelike;
use chrono::{DateTime, Utc};
use tracing::warn;

const COMICINFO_FILENAME: &str = "ComicInfo.xml";

use crate::{
    library_scanner::blurhash::encode,
    types::{
        comic_info::{ComicInfo, ComicPageInfo, ComicPageType},
        CategoryID, TitleID,
    },
    AppState, ArchiveFile, IteratorExt, SUPPORTED_IMAGE_FORMATS,
};

/// Upsert a title to the database and return its ID and ComicInfo.
pub async fn upsert_title(
    app_state: Arc<AppState>,
    category_id: Option<CategoryID>,
    title_file_path: &PathBuf,
) -> Result<(TitleID, ComicInfo)> {
    '_pre_checks: {
        if !title_file_path.exists() {
            return Err(anyhow!("content file not exists"));
        }
        if !title_file_path.is_file() {
            return Err(anyhow!("content file is not a file"));
        }
        if !title_file_path
            .extension()
            .map(|s| s.to_string_lossy() == "zip")
            .unwrap_or(false)
        {
            return Err(anyhow!("content file is not a zip file"));
        }
    }

    // micro optimization, this value is used frequently
    let title_file_path_string = title_file_path.to_string_lossy().to_string();

    let mut archive_file = ArchiveFile::from(title_file_path.clone())
        .context("can't create ArchiveFile from content file")?;
    let pages_in_archive = archive_file
        .list_files()
        .await
        .context("can't list files in archive")?
        .into_iter()
        .filter(|item| {
            SUPPORTED_IMAGE_FORMATS.contains(
                &item
                    .path
                    .split('.')
                    .last()
                    .unwrap_or_default()
                    .to_ascii_lowercase()
                    .as_str(),
            )
        })
        .collect::<Vec<_>>();

    let mut comic_info = archive_file
        .read_file(COMICINFO_FILENAME)
        .context(format!("can't get {COMICINFO_FILENAME} in content file"))
        .and_then(|b| {
            String::from_utf8(b)
                .context(format!("can't decode {COMICINFO_FILENAME} in content file"))
        })
        .and_then(|s| ComicInfo::from_str(&s))?;

    let (cover_path, cover_blurhash, cover_width, cover_height): (
        Option<String>,
        Option<String>,
        Option<i32>,
        Option<i32>,
    ) = 'scoped: {
        // if found a FrontCover page in ComicInfo.xml
        if let Some((cover, cover_path)) = comic_info
            .pages_mut()
            .iter_mut()
            .find(|p| p.page_type == ComicPageType::FrontCover)
            .and_then(|p| p.image_path.clone().map(|path| (p, path)))
        {
            let real_modified_date = pages_in_archive
                .iter()
                .find(|i| i.path == *cover_path)
                .map(|i| i.last_modified);

            if real_modified_date.is_none() {
                warn!("can't get modified date for files inside {title_file_path_string}")
            }

            // and all the following fields are valid
            if let (Some(blurhash), Some(modified_date_at_encode), Some(ref real_modified_date)) = (
                cover.blurhash.as_ref(),
                cover.modified_date_at_encode,
                real_modified_date,
            ) {
                let valid_dimension = cover.image_width > 0 && cover.image_height > 0;
                let modified_date_at_encode = modified_date_at_encode.with_nanosecond(0);
                let real_modified_date = real_modified_date.with_nanosecond(0);
                let unmodified = modified_date_at_encode == real_modified_date;
                // then use them
                if valid_dimension && unmodified {
                    break 'scoped (
                        Some(cover_path.clone()),
                        Some(blurhash.clone()),
                        Some(cover.image_width),
                        Some(cover.image_height),
                    );
                }
            }

            // else re-encode the page
            let blurhash_result = archive_file
                .read_file(&cover_path)
                .context("can't get cover file")
                .and_then(|buf| image::load_from_memory(&buf).context("can't decode image"))
                .and_then(|img| encode(&img).context("can't encode image to blurhash"));
            match blurhash_result {
                Ok(blurhash_result) => {
                    cover.blurhash = Some(blurhash_result.blurhash.clone());
                    cover.image_width = blurhash_result.width;
                    cover.image_height = blurhash_result.height;
                    cover.modified_date_at_encode = real_modified_date;
                    break 'scoped (
                        Some(cover_path.clone()),
                        Some(blurhash_result.blurhash),
                        Some(cover.image_width),
                        Some(cover.image_height),
                    );
                }
                Err(e) => {
                    warn!("can't encode the configured cover of {title_file_path_string}: {e:#}");
                }
            }
        }

        // else try every single pages
        match pages_in_archive.iter().try_find_map(|item| {
            archive_file
                .read_file(&item.path)
                .context("can't get file in archive")
                .and_then(|buf| image::load_from_memory(&buf).context("can't decode image"))
                .and_then(|img| encode(&img).context("can't encode image to blurhash"))
                .map(|blurhash_result| (item, blurhash_result))
        }) {
            Ok((item, blurhash_result)) => {
                comic_info.pages_mut().push(ComicPageInfo {
                    page_type: ComicPageType::FrontCover,
                    blurhash: Some(blurhash_result.blurhash.clone()),
                    image_path: Some(item.path.clone()),
                    image_width: blurhash_result.width,
                    image_height: blurhash_result.height,
                    modified_date_at_encode: Some(item.last_modified),
                    ..Default::default()
                });
                break 'scoped (
                    Some(item.path.clone()),
                    Some(blurhash_result.blurhash),
                    Some(blurhash_result.width),
                    Some(blurhash_result.height),
                );
            }
            Err(e) => {
                warn!("there's no file in {title_file_path_string} that can be encoded to blurhash: {e:#}");
            }
        };

        (None, None, None, None)
    };

    // no longer need to modify anything in the ComicInfo.xml
    // beside the cover page, so we can write it back here
    archive_file
        .upsert_file(
            COMICINFO_FILENAME,
            Arc::new(comic_info.to_pretty_string()?.as_bytes().to_vec()),
        )
        .context("can't write back metadata to content file")?;

    let mut txn = app_state.pool.begin().await?;

    let title_id = sqlx::query!(
        r#"INSERT INTO titles
            (id, title, category_id, author, description, release,
            path, cover_path, cover_blurhash, cover_width,
            cover_height, date_updated)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        ON CONFLICT (path)
        DO UPDATE SET
            title = EXCLUDED.title,
            category_id = EXCLUDED.category_id,
            author = EXCLUDED.author,
            description = EXCLUDED.description,
            release = EXCLUDED.release,
            path = EXCLUDED.path,
            cover_path = EXCLUDED.cover_path,
            cover_blurhash = EXCLUDED.cover_blurhash,
            cover_width = EXCLUDED.cover_width,
            cover_height = EXCLUDED.cover_height,
            date_updated = EXCLUDED.date_updated
        RETURNING id
        "#,
        app_state.generate_snowflake_id().await?,
        comic_info.title,
        category_id,
        comic_info.penciller.as_ref(),
        comic_info.summary.as_ref(),
        comic_info.get_release(),
        title_file_path_string.to_string(),
        cover_path,
        cover_blurhash,
        cover_width,
        cover_height,
        tokio::fs::metadata(&title_file_path)
            .await
            .and_then(|m| { m.modified().map(|d| DateTime::<Utc>::from(d)) })
            .map_err(|e| warn!("can't get modified date of title file: {}", e))
            .ok()
            .unwrap_or_default(),
    )
    .fetch_one(&mut *txn)
    .await
    .context("can't upsert title model to DB")?
    .id;

    '_upsert_pages: {
        let mut page_ids = Vec::with_capacity(pages_in_archive.len());
        let mut page_paths = Vec::with_capacity(pages_in_archive.len());
        let mut page_filesizes = Vec::with_capacity(pages_in_archive.len());
        let mut page_descriptions = Vec::with_capacity(pages_in_archive.len());

        for item in pages_in_archive.iter() {
            page_ids.push(
                app_state
                    .generate_snowflake_id()
                    .await
                    .context("can't generate page id")?,
            );
            page_paths.push(item.path.clone());
            page_filesizes.push(item.size.unwrap_or_default());
            page_descriptions.push(
                comic_info
                    .get_page_description(&item.path)
                    .unwrap_or_default(),
            );
        }

        sqlx::query!(
            r#"
            WITH _ AS (
            INSERT INTO pages (id, title_id, path, filesize, description)
            SELECT id, $1, path, NULLIF(filesize, 0), NULLIF(description, '')
                FROM UNNEST($2::bigint[], $3::text[], $4::bigint[], $5::text[])
                AS t(id, path, filesize, description)
            ON CONFLICT (title_id, path) DO UPDATE
                SET description = EXCLUDED.description
            )
            DELETE FROM pages WHERE title_id = $1
                AND path NOT IN (SELECT UNNEST($3::text[]))
            "#,
            title_id,
            &page_ids,
            &page_paths,
            &page_filesizes,
            &page_descriptions
        )
        .execute(&mut *txn)
        .await
        .context("can't upsert new pages")?;
    }

    'upsert_tags: {
        if comic_info.tags.is_empty() {
            sqlx::query!(r#"DELETE FROM titles_tags WHERE title_id = $1"#, title_id)
                .execute(&mut *txn)
                .await
                .context("can't delete tags")?;
            break 'upsert_tags;
        }

        let mut tag_ids = Vec::with_capacity(comic_info.tags.len());
        for _ in 0..comic_info.tags.len() {
            tag_ids.push(
                app_state
                    .generate_snowflake_id()
                    .await
                    .context("can't generate tag id")?,
            );
        }

        sqlx::query!(
            r#"
            WITH tag_ids AS (
                INSERT INTO tags (id, name)
                SELECT id, name
                    FROM UNNEST($1::bigint[], $2::text[])
                    AS t(id, name)
                ON CONFLICT (name) DO UPDATE SET
                    name = EXCLUDED.name WHERE FALSE
                RETURNING id
            )
            INSERT INTO titles_tags (title_id, tag_id)
                SELECT $3, id
                FROM tag_ids
            ON CONFLICT DO NOTHING
            "#,
            &tag_ids,
            &comic_info.tags,
            title_id,
        )
        .execute(&mut *txn)
        .await
        .context("can't insert tags to database")?;
    }

    txn.commit().await.context("can't commit transaction")?;

    Ok((title_id, comic_info))
}
