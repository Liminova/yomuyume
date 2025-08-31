// mod handle_archive_chapter;
// mod handle_directory_chapter;
// mod try_everything_as_cover;
// mod try_get_configured_cover;

use std::{collections::HashSet, sync::Arc};

use chrono::{DateTime, Utc};
use redb::ReadableDatabase;
use tokio::task::JoinSet;
use tracing::warn;

use crate::{
    AppState,
    database::{
        self,
        content::{CategoryIdentityPath, PageIdentityPath, TitleIdentityPath},
    },
    indexer::{
        dir_entry_guesser::{IndexedChapterKind, PartialIndexedChapter},
        start_index::UpsertTitleErr,
        utils::{
            IndexedChapterPages, find_chapter_cover::find_chapter_cover,
            index_archive_chap_pages::read_chap_pages_archive,
            index_directory_chap_pages::read_chap_pages_dir,
        },
    },
    utils::{
        absolute_path::AbsolutePath, archive_file::ArchiveFile, average_color::HexColor,
        comic_info::ComicInfo, constants::COMICINFO, okay::MapErrorThenOk,
    },
};

#[derive(Debug)]
struct IndexedChapter {
    pub identity_path: String,
    pub fallback_vol_num: u32,
    pub last_modified: Option<DateTime<Utc>>,
    pub cover: Option<(PageIdentityPath, HexColor)>,
    pub pages: IndexedChapterPages,
}

/// index a series to the database and return its ID
#[allow(clippy::cognitive_complexity)]
pub async fn index_series(
    app_state: Arc<AppState>,
    title_path: AbsolutePath,
    partial_indexed_chapters: Vec<PartialIndexedChapter>,
    parent_path: Option<AbsolutePath>,
) -> Result<(Option<CategoryIdentityPath>, TitleIdentityPath), UpsertTitleErr> {
    let title_identity_path = title_path
        .to_relative(Some(
            parent_path
                .as_ref()
                .unwrap_or(&app_state.config.library_path),
        ))?
        .to_string_lossy()
        .to_string();

    let category_identity_path = parent_path
        .as_ref()
        .map(|p| {
            p.to_relative(Some(&app_state.config.library_path))
                .map(|p| p.to_string_lossy().to_string())
        })
        .transpose()?;

    let mut join_set = JoinSet::new();

    let read_txn = app_state.db.content.begin_read().unwrap();
    let pages_table = Arc::new(read_txn.open_table(database::content::PAGES).unwrap());

    let chapters_table = Arc::new(read_txn.open_table(database::content::CHAPTERS).unwrap());
    for partial_indexed_chapter in partial_indexed_chapters {
        let chapters_table = chapters_table.clone();
        let pages_table = pages_table.clone();
        let title_path = title_path.clone();
        let title_identity_path = title_identity_path.clone();
        let app_state = app_state.clone();

        join_set.spawn(async move {
            let chapter_path = &partial_indexed_chapter.path;

            let chapter_identity_path = chapter_path
                .to_relative(Some(&title_path))
                .map(|p| p.to_string_lossy().to_string())
                .map_err(UpsertTitleErr::PathAbsoluteConv)?;

            let chapter_info = chapters_table
                .get((
                    title_identity_path.clone(),
                    Some(chapter_identity_path.clone()),
                ))
                .unwrap()
                .map(|v| v.value());

            let pages_in_db = app_state
                .first_time_index_content
                .then(|| Vec::new())
                .unwrap_or_else(|| {
                    chapter_info
                        .map(|c| c.pages)
                        .map(|page_identity_paths| {
                            page_identity_paths
                                .iter()
                                .filter_map(|page_identity_path| {
                                    pages_table
                                        .get((
                                            title_identity_path.clone(),
                                            Some(chapter_identity_path.clone()),
                                            page_identity_path.clone(),
                                        ))
                                        .okay(|e| warn!("can't get PageInfo: {e:?}"))
                                        .flatten()
                                        .map(|p| (page_identity_path.clone(), p.value()))
                                })
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default()
                });

            match partial_indexed_chapter.kind {
                IndexedChapterKind::Directory => read_chap_pages_dir(
                    &app_state,
                    &partial_indexed_chapter.path,
                    partial_indexed_chapter
                        .path
                        .as_ref()
                        .read_dir()
                        .map_err(UpsertTitleErr::CantReadChapterDir)?
                        .filter_map(|e| {
                            e.okay(|e| {
                                warn!("can't read chapter entry: {e:?}");
                            })
                        })
                        .collect::<Vec<_>>(),
                    &pages_in_db,
                )
                .await
                .map(|pages| IndexedChapter {
                    identity_path: chapter_identity_path,
                    fallback_vol_num: partial_indexed_chapter.fallback_vol_num,
                    last_modified: chapter_path.last_modified().okay(|e| {
                        warn!("can't get last modified of {}: {e}", chapter_path.display())
                    }),
                    cover: find_chapter_cover(
                        &pages,
                        chapter_path
                            .as_ref()
                            .join(COMICINFO)
                            .exists()
                            .then(|| {
                                std::fs::read_to_string(chapter_path.as_ref().join(COMICINFO))
                                    .okay(|e| {
                                        warn!(
                                            "can't read ComicInfo.xml from chapter {}: {e}",
                                            chapter_path.display()
                                        )
                                    })
                                    .and_then(|s| {
                                        ComicInfo::from_str(&s).okay(|e| {
                                            warn!(
                                                "can't parse ComicInfo.xml from chapter {}: {e}",
                                                chapter_path.display()
                                            )
                                        })
                                    })
                            })
                            .flatten()
                            .as_ref()
                            .map(|ci| ci.pages().as_slice()),
                        &pages_in_db,
                    ),
                    pages,
                }),

                IndexedChapterKind::Archive(items_in_archive) => read_chap_pages_archive(
                    &app_state,
                    &chapter_path,
                    items_in_archive,
                    &pages_in_db,
                )
                .await
                .map(|pages| IndexedChapter {
                    identity_path: chapter_identity_path,
                    fallback_vol_num: partial_indexed_chapter.fallback_vol_num,
                    last_modified: chapter_path.last_modified().okay(|e| {
                        warn!("can't get last modified of {}: {e}", chapter_path.display())
                    }),
                    cover: find_chapter_cover(
                        &pages,
                        chapter_path
                            .as_ref()
                            .read_file_from_archive(COMICINFO)
                            .okay(|e| {
                                warn!(
                                    "can't read ComicInfo.xml from chapter {}: {e}",
                                    chapter_path.display()
                                )
                            })
                            .and_then(|b| {
                                String::from_utf8(b).okay(|e| {
                                    warn!(
                                        "can't convert ComicInfo.xml from chapter {} to UTF-8: {e}",
                                        chapter_path.display()
                                    )
                                })
                            })
                            .and_then(|s| {
                                ComicInfo::from_str(&s).okay(|e| {
                                    warn!(
                                        "can't parse ComicInfo.xml from chapter {}: {e}",
                                        chapter_path.display()
                                    )
                                })
                            })
                            .as_ref()
                            .map(|ci| ci.pages().as_slice()),
                        &pages_in_db,
                    ),
                    pages,
                }),
            }
        });
    }
    drop(chapters_table);
    drop(pages_table);

    let indexed_chapters = join_set
        .join_all()
        .await
        .into_iter()
        .filter_map(|r| {
            r.okay(|e| {
                warn!("can't index chapter: {e:?}");
            })
        })
        .collect::<Vec<_>>();

    let titles_table = Arc::new(read_txn.open_table(database::content::TITLES).unwrap());
    let chapters_to_remove = 'scoped: {
        if app_state.first_time_index_content {
            break 'scoped Vec::new();
        }

        let chapters_in_db = titles_table
            .get((category_identity_path.clone(), title_identity_path.clone()))
            .unwrap()
            .map(|t| t.value().chapters)
            .flatten()
            .unwrap_or_default();

        let chapters_in_title = indexed_chapters
            .iter()
            .map(|c| c.identity_path.clone())
            .collect::<HashSet<_>>();

        chapters_in_db
            .into_iter()
            .filter(|c| !chapters_in_title.contains(c))
            .collect::<Vec<_>>()
    };
    drop(titles_table);

    let write_txn = app_state.db.content.begin_write().unwrap();

    let mut categories_table = write_txn.open_table(database::content::CATEGORIES).unwrap();
    if let Some(ref category_identity_path) = category_identity_path
        && let Some(category) = categories_table
            .get_mut(category_identity_path.clone())
            .unwrap()
    {
        if !category.value().contains(&title_identity_path) {
            category.value().push(title_identity_path.clone());
        }
    } else if let Some(category_identity_path) = category_identity_path.clone() {
        categories_table.insert(category_identity_path, vec![title_identity_path.clone()]);
    }
    drop(categories_table);

    let mut titles_table = write_txn.open_table(database::content::TITLES).unwrap();
    titles_table.insert(
        (category_identity_path.clone(), title_identity_path.clone()),
        database::content::TitleInfo {
            chapters: Some(
                indexed_chapters
                    .iter()
                    .map(|c| c.identity_path.clone())
                    .collect::<Vec<_>>(),
            ),
            // TODO: handle directory
            last_modified: title_path
                .is_file()
                .then(|| {
                    title_path.last_modified().okay(|e| {
                        warn!(
                            "can't get last modified of title {}: {e}",
                            title_path.display()
                        );
                    })
                })
                .flatten(),
        },
    );
    drop(titles_table);

    let mut chapters_table = write_txn.open_table(database::content::CHAPTERS).unwrap();
    for indexed_chapter in &indexed_chapters {
        chapters_table.insert(
            (
                title_identity_path.clone(),
                Some(indexed_chapter.identity_path.clone()),
            ),
            database::content::ChapterInfo {
                pages: indexed_chapter
                    .pages
                    .upsert
                    .iter()
                    .map(|p| p.identity_path.clone())
                    .collect(),
                fallback_vol_num: Some(indexed_chapter.fallback_vol_num),
                cover: indexed_chapter.cover.clone(),
                last_modified: indexed_chapter.last_modified,
            },
        );
    }
    for chapter_to_remove in chapters_to_remove {
        chapters_table.remove((title_identity_path.clone(), Some(chapter_to_remove)));
    }
    drop(chapters_table);

    let mut pages_table = write_txn.open_table(database::content::PAGES).unwrap();
    for indexed_chapter in indexed_chapters {
        for page in indexed_chapter.pages.upsert.into_iter() {
            pages_table.insert(
                (
                    title_identity_path.clone(),
                    Some(indexed_chapter.identity_path.clone()),
                    page.identity_path,
                ),
                database::content::PageInfo {
                    width: page.width,
                    height: page.height,
                    color: page.color,
                    size: page.size,
                    last_modified: page.last_modified,
                    parent_path: title_path.to_relative(Some(&app_state.config.library_path))?,
                },
            );
        }
        for page_to_delete in indexed_chapter.pages.delete {
            pages_table.remove((
                title_identity_path.clone(),
                Some(indexed_chapter.identity_path.clone()),
                page_to_delete,
            ));
        }
    }
    drop(pages_table);

    write_txn.commit().unwrap();

    Ok((category_identity_path, title_identity_path))
}
