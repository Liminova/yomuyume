// mod handle_archive_chapter;
// mod handle_directory_chapter;
// mod try_everything_as_cover;
// mod try_get_configured_cover;

use std::{collections::HashSet, sync::Arc};

use chrono::{DateTime, Utc};
use redb::ReadableDatabase;
use tokio::task::JoinSet;
use tracing::{error, info, warn};

use crate::{
    AppState,
    database::{
        self,
        content::{
            CategoryIdentityPath, PageIdentityPath, TitleIdentityPath, chapter_key, page_key,
            title_key,
        },
    },
    indexer::{
        dir_entry_guesser::{IndexedChapterKind, PartialIndexedChapter},
        utils::{
            IndexedChapterPages, find_chapter_cover::find_chapter_cover,
            index_archive_chap_pages::read_chap_pages_archive,
            index_directory_chap_pages::read_chap_pages_dir,
        },
    },
    utils::{
        absolute_path::AbsolutePath, archive_file::ArchiveFile, average_color::HexColor,
        comic_info::ComicInfo, constants::COMICINFO, result_utils::ResultUtils,
    },
};

#[derive(Debug)]
struct IndexedChapter {
    pub path: AbsolutePath,
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
) -> Option<(Option<CategoryIdentityPath>, TitleIdentityPath)> {
    let title_identity_path = title_path
        .to_relative(Some(
            parent_path
                .as_ref()
                .unwrap_or(&app_state.config.library_path),
        ))
        .okay(|e| error!("can't convert title path to relative: {e:?}"))?
        .to_string_lossy()
        .to_string();

    let category_identity_path = parent_path
        .as_ref()
        .map(|p| {
            p.to_relative(Some(&app_state.config.library_path))
                .map(|p| p.to_string_lossy().to_string())
                .okay(|e| {
                    warn!("can't convert category path to relative: {e:?}");
                })
        })
        .flatten();

    let mut join_set = JoinSet::new();

    let read_txn = app_state
        .db
        .content
        .begin_read()
        .okay(|e| error!("can't begin read transaction: {e:?}"))?;
    let pages_table = Arc::new(
        read_txn
            .open_table(database::content::PAGES)
            .okay(|e| error!("can't open pages table: {e:?}"))?,
    );

    let chapters_table = Arc::new(
        read_txn
            .open_table(database::content::CHAPTERS)
            .okay(|e| error!("can't open chapters table: {e:?}"))?,
    );
    for partial_indexed_chapter in partial_indexed_chapters {
        let chapters_table = chapters_table.clone();
        let pages_table = pages_table.clone();
        let title_path = title_path.clone();
        let category_identity_path = category_identity_path.clone();
        let title_identity_path = title_identity_path.clone();
        let app_state = app_state.clone();

        join_set.spawn(async move {
            let chapter_path = partial_indexed_chapter.path;
            let chapter_path_display = format!("{}", chapter_path.display());
            let chapter_identity_path = chapter_path
                .to_relative(Some(&title_path))
                .map(|p| p.to_string_lossy().to_string())
                .okay(|e| error!("can't convert chapter path to relative: {e:?}"))?;

            let chapter_info = chapters_table
                .get(chapter_key(
                    category_identity_path.clone(),
                    title_identity_path.clone(),
                    Some(chapter_identity_path.clone()),
                ))
                .okay(|e| warn!("can't get ChapterInfo: {e:?}"))?
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
                                        .get(page_key(
                                            category_identity_path.clone(),
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
                IndexedChapterKind::Directory => {
                    let chapter_comic_info_path = chapter_path.as_ref().join(COMICINFO);
                    read_chap_pages_dir(&app_state, &chapter_path, None, &pages_in_db)
                        .await
                        .map(|pages| IndexedChapter {
                            identity_path: chapter_identity_path,
                            fallback_vol_num: partial_indexed_chapter.fallback_vol_num,
                            last_modified: chapter_path.last_modified().okay(|e| {
                                warn!("can't get last modified of {chapter_path_display}: {e}")
                            }),
                            cover: find_chapter_cover(
                                &pages,
                                chapter_comic_info_path
                                    .exists()
                                    .then(|| {
                                        std::fs::read_to_string(chapter_comic_info_path).okay(|e| {
                                            warn!(
                                                "can't read ComicInfo.xml from chapter {chapter_path_display}: {e}"
                                            )
                                        })
                                    })
                                    .flatten()
                                    .and_then(|s| {
                                        ComicInfo::from_str(&s).okay(|e| {
                                            warn!(
                                                "can't parse ComicInfo.xml from chapter {chapter_path_display}: {e}"
                                            )
                                        })
                                    })
                                    .as_ref()
                                    .map(|ci| ci.pages().as_slice()),
                                &pages_in_db,
                            ),
                            path: chapter_path,
                            pages,
                        })
                }

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
                        warn!("can't get last modified of {}: {e}", chapter_path_display)
                    }),
                    cover: find_chapter_cover(
                        &pages,
                        chapter_path
                            .as_ref()
                            .read_file_from_archive(COMICINFO)
                            .okay(|e| {
                                warn!(
                                    "can't read ComicInfo.xml from chapter {chapter_path_display}: {e}"
                                )
                            })
                            .and_then(|b| {
                                String::from_utf8(b).okay(|e| {
                                    warn!(
                                        "can't convert ComicInfo.xml from chapter {chapter_path_display} to UTF-8: {e}"
                                    )
                                })
                            })
                            .and_then(|s| {
                                ComicInfo::from_str(&s).okay(|e| {
                                    warn!(
                                        "can't parse ComicInfo.xml from chapter {chapter_path_display}: {e}"
                                    )
                                })
                            })
                            .as_ref()
                            .map(|ci| ci.pages().as_slice()),
                        &pages_in_db,
                    ),
                    path: chapter_path,
                    pages,
                })
            }
        });
    }
    drop(chapters_table);
    drop(pages_table);

    let indexed_chapters = join_set
        .join_all()
        .await
        .into_iter()
        .filter_map(|c| c)
        .collect::<Vec<_>>();

    if indexed_chapters.is_empty() {
        info!(
            "no chapter indexed for title {}, aborting",
            title_path.display()
        );
        return None;
    }

    let titles_table = Arc::new(
        read_txn
            .open_table(database::content::TITLES)
            .okay(|e| error!("can't open titles table: {e:?}"))?,
    );
    let chapters_to_remove = 'scoped: {
        if app_state.first_time_index_content {
            break 'scoped Vec::new();
        }

        let chapters_in_db = titles_table
            .get(title_key(
                category_identity_path.clone(),
                title_identity_path.clone(),
            ))
            .okay(|e| warn!("can't get TitleInfo: {e:?}"))?
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

    let write_txn = app_state
        .db
        .content
        .begin_write()
        .okay(|e| error!("can't begin write transaction: {e:?}"))?;

    let mut categories_table = write_txn
        .open_table(database::content::CATEGORIES)
        .okay(|e| error!("can't open categories table: {e:?}"))?;
    if let Some(ref category_identity_path) = category_identity_path
        && let Some(category) = categories_table
            .get_mut(category_identity_path.clone())
            .okay(|e| error!("can't get mut Category: {e:?}"))?
    {
        if !category.value().contains(&title_identity_path) {
            category.value().push(title_identity_path.clone());
        }
    } else if let Some(category_identity_path) = category_identity_path.clone() {
        categories_table.insert(category_identity_path, vec![title_identity_path.clone()]);
    }
    drop(categories_table);

    let mut titles_table = write_txn
        .open_table(database::content::TITLES)
        .okay(|e| error!("can't open titles table: {e:?}"))?;
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

    let mut chapters_table = write_txn
        .open_table(database::content::CHAPTERS)
        .okay(|e| error!("can't open chapters table: {e:?}"))?;
    for indexed_chapter in &indexed_chapters {
        chapters_table.insert(
            chapter_key(
                category_identity_path.clone(),
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
        chapters_table.remove(chapter_key(
            category_identity_path.clone(),
            title_identity_path.clone(),
            Some(chapter_to_remove),
        ));
    }
    drop(chapters_table);

    let mut pages_table = write_txn
        .open_table(database::content::PAGES)
        .okay(|e| error!("can't open pages table: {e:?}"))?;
    for indexed_chapter in indexed_chapters {
        let parent_path = match indexed_chapter.path.to_relative(Some(&title_path)) {
            Ok(path) => path,
            Err(e) => {
                warn!("can't convert chapter path to relative: {e:?}");
                continue;
            }
        };
        for page in indexed_chapter.pages.upsert.into_iter() {
            pages_table.insert(
                page_key(
                    category_identity_path.clone(),
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
                    parent_path: parent_path.clone(),
                },
            );
        }
        for page_to_delete in indexed_chapter.pages.delete {
            pages_table.remove(page_key(
                category_identity_path.clone(),
                title_identity_path.clone(),
                Some(indexed_chapter.identity_path.clone()),
                page_to_delete,
            ));
        }
    }
    drop(pages_table);

    write_txn
        .commit()
        .okay(|e| error!("can't commit write transaction: {e:?}"))?;

    // TODO: write ComicInfo.xml into tantivy

    Some((category_identity_path, title_identity_path))
}
