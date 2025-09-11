use std::sync::Arc;

use chrono::{NaiveDateTime, TimeZone, Utc};
use sqlx::QueryBuilder;
use tokio::task::JoinSet;
use tracing::{error, info, warn};

use crate::{
    AppState,
    indexer::{
        IndexedContent,
        dir_entry_guesser::{IndexedChapterKind, PartialIndexedChapter},
        utils::{
            IndexedChapterPages, PageInDB, comic_info_to_tantivy::comic_info_to_tantivy,
            find_chapter_cover::find_chapter_cover_page_id,
            index_archive_chap_pages::read_chap_pages_archive,
            index_directory_chap_pages::read_chap_pages_dir,
        },
    },
    utils::{
        absolute_path::AbsolutePath,
        average_color::HexColor,
        nanoid::nanoid,
        pathbuf_utils::PathBufUtils,
        result_utils::{OptionUtils, ResultUtils},
    },
};

#[derive(Debug)]
struct IndexedChapter {
    pub id: String,
    pub rel_path: String,
    pub number: i32,
    pub last_modified: Option<NaiveDateTime>,
    pub cover_page_id: Option<String>,
    pub pages: IndexedChapterPages,
}

/// index a series to the database and return its ID
#[allow(clippy::cognitive_complexity)]
pub async fn index_series(
    app_state: Arc<AppState>,
    title_path: AbsolutePath,
    partial_indexed_chapters: Vec<PartialIndexedChapter>,
    parent_path: Option<AbsolutePath>,
) -> Option<IndexedContent> {
    let title_relative_path = title_path
        .to_relative(Some(&app_state.config.library_path))
        .okay(|e| error!("can't convert title path to relative: {e:?}"))?;
    let title_relative_path_str = title_relative_path.to_string_lossy().to_string();

    let mut join_set = JoinSet::new();

    for partial_indexed_chapter in partial_indexed_chapters {
        let app_state = app_state.clone();
        let title_path = title_path.clone();
        let title_relative_path_str = title_relative_path_str.clone();

        join_set.spawn(async move {
            let chapter_path = partial_indexed_chapter.path;

            let chapter_relative_path = chapter_path
                .to_relative(Some(&title_path))
                .okay(|e| {
                    error!(
                        "can't convert chapter path to relative: {e:?}, chapter path: {}",
                        chapter_path.display()
                    );
                })
                .map(|p| p.to_string_lossy().to_string())?;

            let chapter_id = sqlx::query!(
                "SELECT chapters.id FROM chapters JOIN titles ON chapters.title_id = titles.id
                WHERE titles.path = ? AND chapters.path = ?",
                title_relative_path_str,
                chapter_relative_path,
            )
            .fetch_optional(&app_state.pool)
            .await
            .okay(|e| {
                error!("can't query existing chapter id of chapter {chapter_relative_path}: {e:?}");
            })?
            .map_or_else(nanoid, |r| r.id);

            let pages_in_db = sqlx::query!(
                "SELECT pages.id, pages.path, pages.last_modified, pages.avg_hex_color
                FROM pages JOIN chapters ON pages.chapter_id = chapters.id
                WHERE chapters.path = ?",
                chapter_relative_path,
            )
            .fetch_all(&app_state.pool)
            .await
            .ok()?
            .into_iter()
            .map(|r| PageInDB {
                id: r.id,
                path: r.path,
                last_modified: r.last_modified.map(|d| Utc.from_utc_datetime(&d)),
                avg_color: r.avg_hex_color.as_ref().and_then(|c| {
                    HexColor::from_str(c).inspect_err(|| error!("can't parse hex color: {c}"))
                }),
            })
            .collect::<Vec<_>>();

            if let IndexedChapterKind::Archive(items_in_archive) = partial_indexed_chapter.kind {
                let comic_info = chapter_path
                    .as_ref()
                    .read_comic_info_from_archive()
                    .okay(|e| warn!("can't read ComicInfo.xml from chapter: {e}"));

                read_chap_pages_archive(&app_state, &chapter_path, items_in_archive, &pages_in_db)
                    .await
                    .map(|pages| IndexedChapter {
                        id: nanoid(),
                        number: comic_info
                            .as_ref()
                            .map_or(partial_indexed_chapter.fallback_vol_num, |ci| ci.volume),
                        last_modified: chapter_path
                            .last_modified()
                            .okay(|e| {
                                warn!("can't get last modified of {}: {e}", chapter_relative_path);
                            })
                            .map(|d| d.naive_utc()),
                        cover_page_id: find_chapter_cover_page_id(
                            &pages,
                            comic_info.as_ref().map(|ci| ci.pages().as_slice()),
                            &pages_in_db,
                        ),
                        rel_path: chapter_relative_path,
                        pages,
                    })
            } else {
                let comic_info = chapter_path
                    .as_ref()
                    .read_comic_info_from_dir()
                    .okay(|e| warn!("can't read ComicInfo.xml from chapter: {e}"));

                read_chap_pages_dir(&app_state, &chapter_path, None, &pages_in_db)
                    .await
                    .map(|pages| IndexedChapter {
                        id: chapter_id,
                        number: partial_indexed_chapter.fallback_vol_num,
                        last_modified: chapter_path
                            .last_modified()
                            .okay(|e| {
                                warn!("can't get last modified of {chapter_relative_path}: {e}");
                            })
                            .map(|d| d.naive_utc()),
                        cover_page_id: find_chapter_cover_page_id(
                            &pages,
                            comic_info.as_ref().map(|ci| ci.pages().as_slice()),
                            &pages_in_db,
                        ),
                        rel_path: chapter_relative_path,
                        pages,
                    })
            }
        });
    }

    let indexed_chapters = join_set
        .join_all()
        .await
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    if indexed_chapters.is_empty() {
        info!(
            "no chapter indexed for title {}, aborting",
            title_path.display()
        );
        return None;
    }

    let category_id = if let Some(parent_path) = parent_path
        .and_then(|p| {
            p.to_relative(Some(&app_state.config.library_path))
                .okay(|e| error!("can't convert category path to relative: {e:?}"))
        })
        .map(|p| p.to_string_lossy().to_string())
    {
        sqlx::query!("SELECT id FROM categories WHERE path = ?", parent_path)
            .fetch_optional(&app_state.pool)
            .await
            .okay(|e| error!("can't query category id of path {parent_path}: {e:?}"))?
            .map(|r| r.id)
    } else {
        None
    };

    let comic_info = title_path.as_ref().read_comic_info_from_dir().okay(|e| {
        warn!(
            "can't read ComicInfo.xml from title {}: {e}",
            title_path.display()
        );
    });
    let comic_info_json = comic_info.as_ref().and_then(|ci| {
        serde_json::to_string(ci).okay(|e| {
            error!(
                "can't serialize ComicInfo.xml to json for title {}: {e}",
                title_path.display()
            );
        })
    });
    let comic_info_bytes = comic_info_json.as_ref().map(String::as_bytes);

    let new_title_id = nanoid();

    let chapter_id_to_cover_page = indexed_chapters
        .iter()
        .filter_map(|c| c.cover_page_id.as_ref().map(|p| (&c.id, p)))
        .collect::<Vec<_>>();

    let mut tx = app_state
        .pool
        .begin()
        .await
        .okay(|e| error!("can't begin SQL transaction: {e:?}"))?;

    let title_id = sqlx::query!(
        "INSERT INTO titles (id, category_id, path, comic_info) VALUES (?, ?, ?, ?)
        ON CONFLICT(path) DO UPDATE SET
            category_id = excluded.category_id,
            last_modified = excluded.last_modified
        RETURNING id",
        new_title_id,
        category_id,
        title_relative_path_str,
        comic_info_bytes,
    )
    .fetch_one(&mut *tx)
    .await
    .okay(|e| error!("can't upsert title {title_relative_path_str}: {e:?}"))?
    .id;

    '_chapter: {
        let mut upsert_query =
            QueryBuilder::new("INSERT INTO chapters (id, title_id, path, number, last_modified) ");
        upsert_query
            .push_values(indexed_chapters.iter(), |mut b, c| {
                b.push_bind(&c.id)
                    .push_bind(&title_id)
                    .push_bind(&c.rel_path)
                    .push_bind(c.number)
                    .push_bind(c.last_modified);
            })
            .push(
                " ON CONFLICT(title_id, path) DO UPDATE SET
                number = excluded.number,
                last_modified = excluded.last_modified
            ",
            )
            .build()
            .execute(&mut *tx)
            .await
            .okay(|e| error!("can't upsert chapters for title {title_relative_path_str}: {e:?}"))?;

        let mut delete_query =
            QueryBuilder::new("DELETE FROM chapters WHERE title_id = ? AND id NOT IN (");
        delete_query
            .push_bind(&title_id)
            .push_values(indexed_chapters.iter(), |mut b, chapter| {
                b.push_bind(&chapter.id);
            })
            .push(")")
            .build()
            .execute(&mut *tx)
            .await
            .okay(|e| {
                error!("can't delete old chapters for title {title_relative_path_str}: {e:?}");
            })?;
    }

    '_page: for indexed_chapter in &indexed_chapters {
        let mut upsert_query = QueryBuilder::new(
            "INSERT INTO pages (id, chapter_id, path, width, height, avg_hex_color, size, last_modified) ",
        );
        upsert_query
            .push_values(indexed_chapter.pages.upsert.iter(), |mut b, p| {
                b.push_bind(&p.id)
                    .push_bind(&indexed_chapter.id)
                    .push_bind(&p.path)
                    .push_bind(p.width)
                    .push_bind(p.height)
                    .push_bind(&p.avg_hex_color)
                    .push_bind(p.size)
                    .push_bind(p.last_modified);
            })
            .push(
                " ON CONFLICT(chapter_id, path) DO UPDATE SET
                width = excluded.width,
                height = excluded.height,
                avg_hex_color = excluded.avg_hex_color,
                size = excluded.size,
                last_modified = excluded.last_modified
            ",
            )
            .build()
            .execute(&mut *tx)
            .await
            .okay(|e| {
                error!(
                    "can't upsert pages for chapter {}: {e:?}",
                    indexed_chapter.rel_path
                );
            })?;

        let mut delete_query =
            QueryBuilder::new("DELETE FROM pages WHERE chapter_id = ? AND id NOT IN (");
        delete_query
            .push_bind(&indexed_chapter.id)
            .push_values(indexed_chapter.pages.upsert.iter(), |mut b, p| {
                b.push_bind(&p.id);
            })
            .push(")")
            .build()
            .execute(&mut *tx)
            .await
            .okay(|e| {
                error!(
                    "can't delete old pages for chapter {}: {e:?}",
                    indexed_chapter.rel_path
                );
            })?;
    }

    let mut upsert_chapter_covers_query =
        QueryBuilder::new("INSERT INTO chapter_cover (chapter_id, page_id) VALUES ");
    upsert_chapter_covers_query
        .push_values(chapter_id_to_cover_page.iter(), |mut b, (ch_id, p_id)| {
            b.push_bind(ch_id).push_bind(p_id);
        })
        .push(
            " ON CONFLICT (chapter_id) DO UPDATE SET page_id = excluded.page_id
            ON CONFLICT (chapter_id, page_id) DO NOTHING
        ",
        )
        .build()
        .execute(&mut *tx)
        .await
        .okay(|e| {
            error!("can't upsert chapter covers for title {title_relative_path_str}: {e:?}");
        })?;

    tx.commit()
        .await
        .okay(|e| error!("can't commit SQL transaction: {e:?}"))?;

    comic_info_to_tantivy(
        &app_state,
        comic_info.as_ref(),
        &title_id,
        &title_relative_path,
    );

    Some(IndexedContent::TitleID(title_id))
}
