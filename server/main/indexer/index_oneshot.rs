use std::{fs::DirEntry, sync::Arc};

use chrono::{TimeZone, Utc};
use rayon::prelude::*;
use sqlx::QueryBuilder;
use tracing::{error, warn};

use crate::{
    AppState,
    indexer::{
        IndexedContent,
        utils::{
            PageInDB, comic_info_to_tantivy::comic_info_to_tantivy,
            find_chapter_cover::find_chapter_cover_page_id,
            index_archive_chap_pages::read_chap_pages_archive,
            index_directory_chap_pages::read_chap_pages_dir,
        },
    },
    utils::{
        absolute_path::AbsolutePath,
        archive_file::ItemInArchive,
        average_color::HexColor,
        nanoid::nanoid,
        pathbuf_utils::PathBufUtils,
        result_utils::{OptionUtils, ResultUtils},
    },
};

#[derive(Debug)]
pub enum OneshotType {
    Archive(Vec<ItemInArchive>),
    Directory(Vec<DirEntry>),
}

/// index a oneshot to the database and return its ID
#[allow(clippy::cognitive_complexity)]
pub async fn index_oneshot(
    app_state: Arc<AppState>,
    title_path: AbsolutePath,
    oneshot_type: OneshotType,
    parent_path: Option<AbsolutePath>,
) -> Option<IndexedContent> {
    let title_relative_path = title_path
        .to_relative(Some(&app_state.config.library_path))
        .okay(|e| error!("can't convert title path to relative: {e:?}"))?;
    let title_relative_path_str = title_relative_path.to_string_lossy().to_string();

    'skip_process_when_db_is_newer: {
        if title_path.is_dir() {
            break 'skip_process_when_db_is_newer;
        }

        let Some((id, old_last_modified)) = sqlx::query!(
            "SELECT id, last_modified FROM titles WHERE path = ?",
            title_relative_path_str,
        )
        .fetch_optional(&app_state.pool)
        .await
        .okay(|e| error!("can't query title info: {e:?}"))?
        .and_then(|r| r.last_modified.map(|d| (r.id, Utc.from_utc_datetime(&d)))) else {
            break 'skip_process_when_db_is_newer;
        };

        let Some(new_last_modified) = title_path.last_modified().okay(|e| {
            warn!(
                "can't get last modified of title {}: {e}",
                title_path.display()
            );
        }) else {
            break 'skip_process_when_db_is_newer;
        };

        if old_last_modified >= new_last_modified {
            return Some(IndexedContent::TitleID(id));
        }
    }

    let pages_in_db = sqlx::query!(
        "SELECT
            pages.id,
            pages.path,
            pages.last_modified,
            pages.avg_hex_color
        FROM
            pages
            LEFT JOIN chapters ON pages.chapter_id = chapters.id
            LEFT JOIN titles ON chapters.title_id = titles.id
        WHERE
            titles.path = ?
        ORDER BY
            chapters.number,
            pages.path",
        title_relative_path_str,
    )
    .fetch_all(&app_state.pool)
    .await
    .okay(|e| error!("can't query existing pages of title {title_relative_path_str}: {e:?}"))?
    .into_iter()
    .map(|r| PageInDB {
        avg_color: r.avg_hex_color.and_then(|c| {
            HexColor::from_str(&c)
                .log_err(|| error!("can't parse hex color for page {}: {c}", r.id))
        }),
        id: r.id,
        path: r.path,
        last_modified: r.last_modified.map(|d| Utc.from_utc_datetime(&d)),
    })
    .collect::<Vec<_>>();

    let (indexed_pages, comic_info) = match oneshot_type {
        OneshotType::Archive(files_in_archive) => (
            read_chap_pages_archive(&app_state, &title_path, files_in_archive, &pages_in_db)
                .await?,
            title_path
                .as_ref()
                .read_comic_info_from_archive()
                .okay(|e| {
                    warn!(
                        "can't read ComicInfo.xml from oneshot {}: {e}",
                        title_path.display()
                    );
                }),
        ),

        OneshotType::Directory(sub_entries) => (
            read_chap_pages_dir(&app_state, &title_path, Some(sub_entries), &pages_in_db).await?,
            title_path.as_ref().read_comic_info_from_dir().okay(|e| {
                warn!(
                    "can't read ComicInfo.xml from oneshot {}: {e}",
                    title_path.display()
                );
            }),
        ),
    };

    let category_id = 'scoped: {
        let Some(parent_path) = parent_path
            .and_then(|p| {
                p.to_relative(Some(&app_state.config.library_path))
                    .okay(|e| error!("can't convert category path to relative: {e:?}"))
            })
            .map(|p| p.to_string_lossy().to_string())
        else {
            break 'scoped None;
        };
        sqlx::query!("SELECT id FROM categories WHERE path = ?", parent_path)
            .fetch_optional(&app_state.pool)
            .await
            .okay(|e| error!("can't query category id of path {parent_path}: {e:?}"))?
            .map(|r| r.id)
    };

    let cover_page_id = find_chapter_cover_page_id(
        &indexed_pages,
        comic_info.as_ref().map(|p| p.pages().as_slice()),
        &pages_in_db,
    );

    let new_title_id = nanoid();
    let new_chapter_id = nanoid();

    let title_last_modified = title_path
        .is_file()
        .then(|| {
            title_path.last_modified().okay(|e| {
                warn!(
                    "can't get last modified of title {}: {e}",
                    title_path.display()
                );
            })
        })
        .flatten();

    let comic_info_json = comic_info
        .as_ref()
        .and_then(|ci| {
            serde_json::to_string(ci).okay(|e| {
                error!(
                    "can't serialize ComicInfo.xml of title {}: {e}",
                    title_path.display()
                );
            })
        })
        .map(|s| s.into_bytes());

    let mut tx = app_state
        .pool
        .begin()
        .await
        .okay(|e| error!("can't begin SQL transaction: {e:?}"))?;

    let title_id = sqlx::query!(
        "INSERT INTO titles (id, category_id, path, last_modified, comic_info) VALUES (?, ?, ?, ?, ?)
        ON CONFLICT(path) DO UPDATE SET
            category_id=excluded.category_id,
            last_modified=excluded.last_modified,
            comic_info=excluded.comic_info
        RETURNING id",
        new_title_id,
        category_id,
        title_relative_path_str,
        title_last_modified,
        comic_info_json,
    )
    .fetch_one(&mut *tx)
    .await
    .okay(|e|
        error!(
            "can't upsert title {title_relative_path_str}: {e:?}"
        )
    )?
    .id;

    sqlx::query!(
        "INSERT INTO title_covers (title_id, page_id) VALUES (?, ?)
        ON CONFLICT(title_id) DO UPDATE SET page_id=excluded.page_id",
        title_id,
        cover_page_id,
    )
    .execute(&mut *tx)
    .await
    .okay(|e| error!("can't upsert title cover page of title {title_relative_path_str}: {e:?}"))?;

    let chapter_id = sqlx::query!(
        "INSERT INTO chapters (id, title_id, path) VALUES (?, ?, ?)
        ON CONFLICT (title_id, path) DO UPDATE SET id=id RETURNING id",
        new_chapter_id,
        title_id,
        "",
    )
    .fetch_one(&mut *tx)
    .await
    .okay(|e| {
        error!("can't upsert or fetch chapter (oneshot) of title {title_relative_path_str}: {e:?}");
    })?
    .id;

    sqlx::query!(
        "DELETE FROM chapters WHERE title_id = ? AND id != ?",
        title_id,
        chapter_id
    )
    .execute(&mut *tx)
    .await
    .okay(|e| {
        error!(
            "can't delete old chapters (oneshot) of title {}: {e:?}",
            title_relative_path.display()
        );
    })?;

    '_pages: {
        if !indexed_pages.delete.is_empty() {
            let mut delete_query = QueryBuilder::new("DELETE FROM pages WHERE id IN (");
            delete_query
                .push_values(indexed_pages.delete.into_iter(), |mut b, page_id| {
                    b.push_bind(page_id);
                })
                .push(")")
                .build()
                .execute(&mut *tx)
                .await
                .okay(|e| {
                    error!("can't delete old pages of title {title_relative_path_str}: {e:?}");
                })?;
        }

        let mut new_page_ids = (0..indexed_pages.upsert.len())
            .into_par_iter()
            .map(|_| nanoid())
            .collect::<Vec<_>>();

        let mut upsert_query = QueryBuilder::new(
            "INSERT INTO pages (id, chapter_id, path, width, height, avg_hex_color, size, last_modified) ",
        );
        upsert_query
            .push_values(indexed_pages.upsert.into_iter(), |mut b, page| {
                b.push_bind(new_page_ids.pop().unwrap_or_else(nanoid))
                    .push_bind(chapter_id.clone())
                    .push_bind(page.path)
                    .push_bind(page.width)
                    .push_bind(page.height)
                    .push_bind(page.avg_hex_color)
                    .push_bind(page.size)
                    .push_bind(page.last_modified);
            })
            .push(
                " ON CONFLICT(chapter_id, path) DO UPDATE SET
                last_modified=excluded.last_modified,
                width=excluded.width,
                height=excluded.height,
                avg_hex_color=excluded.avg_hex_color,
                size=excluded.size,
                last_modified=excluded.last_modified",
            )
            .build()
            .execute(&mut *tx)
            .await
            .okay(|e| error!("can't upsert pages of title {title_relative_path_str}: {e:?}"))?;
    }

    tx.commit()
        .await
        .okay(|e| error!("can't commit SQL transaction: {e:?}"))?;

    comic_info_to_tantivy(
        &app_state,
        comic_info.as_ref(),
        &title_id,
        &title_relative_path,
    )
    .await;

    Some(IndexedContent::TitleID(title_id))
}
