use std::{
    collections::{HashSet, VecDeque},
    fs::DirEntry,
    sync::Arc,
};

use redb::ReadableDatabase;
use tokio::task::JoinSet;
use tracing::{debug, error, info};

use crate::{
    AppState,
    database::{
        self,
        content::{chapter_key, page_key},
    },
    indexer::{
        dir_entry_guesser::{DirEntryType, DirEntryTypeGuesser},
        index_oneshot::{OneshotType, index_oneshot},
        index_series::index_series,
    },
    utils::{
        absolute_path::{AbsolutePath, ToAbsolute},
        archive_file::ItemInArchive,
        result_utils::ResultUtils,
    },
};

#[derive(Debug)]
struct ScannedEntry {
    entry: DirEntry,
    parent: Option<AbsolutePath>, // None for the library root
}

type PageInTitle = ItemInArchive;

#[allow(clippy::cognitive_complexity)]
pub async fn start_index(app_state: Arc<AppState>) -> Option<()> {
    // scan library directory
    let library_dir = std::fs::read_dir(app_state.config.library_path.as_ref()).okay(|e| {
        error!(
            "can't read library path `{}`: {e:?}",
            app_state.config.library_path
        );
    })?;

    let mut queue: VecDeque<ScannedEntry> = VecDeque::new();

    // populate first layer of the library tree to queue
    for entry in library_dir {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                error!("can't extract entry from root library: {e:?}");
                continue;
            }
        };
        queue.push_front(ScannedEntry {
            entry,
            parent: None,
        });
    }

    let mut join_set = JoinSet::new();

    // BFS
    while let Some(scanned) = queue.pop_back() {
        let Ok(entry_path) = scanned.entry.path().to_absolute(None) else {
            // this should not happen since library path is already absolute
            unreachable!();
        };

        let app_state = app_state.clone();

        match scanned.entry.guess(
            app_state.config.feature_nomedia,
            app_state.config.feature_komga_oneshot,
            app_state.config.feature_komga_recycle,
        ) {
            Ok(entry_type) => match entry_type {
                DirEntryType::CategoryDir(sub_entries) => {
                    for sub_entry in sub_entries {
                        queue.push_front(ScannedEntry {
                            entry: sub_entry,
                            parent: Some(entry_path.clone()),
                        });
                    }
                }
                DirEntryType::SeriesDir(chapters) => {
                    join_set.spawn(async move {
                        index_series(app_state, entry_path.clone(), chapters, scanned.parent)
                            .await
                            .ok_or(entry_path)
                    });
                }
                DirEntryType::OneShotDir(pages) => {
                    join_set.spawn(async move {
                        index_oneshot(
                            app_state,
                            entry_path.clone(),
                            OneshotType::Directory(pages),
                            scanned.parent,
                        )
                        .await
                        .ok_or(entry_path)
                    });
                }
                DirEntryType::OneShotArchiveFile(files_in_archive) => {
                    join_set.spawn(async move {
                        index_oneshot(
                            app_state,
                            entry_path.clone(),
                            OneshotType::Archive(files_in_archive),
                            scanned.parent,
                        )
                        .await
                        .ok_or(entry_path)
                    });
                }
                DirEntryType::Ignored => {
                    debug!("ignored entry `{}`", entry_path.display());
                }
            },
            Err(e) => {
                error!(
                    "can't guess the directory type for `{}`: {e:?}",
                    entry_path.display()
                );
            }
        }
    }

    if app_state.indexer.first_time {
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(Err(title_path)) => error!(
                    "can't index title `{}`, check previous logs",
                    title_path.display()
                ),
                Err(e) => error!("can't join thread: {e:?}"),
                _ => (),
            }
        }

        info!("finished first-time indexing, skipping cleanup");

        return Some(());
    }
    let mut indexed_category_keys = HashSet::new();
    let mut indexed_title_keys = HashSet::new();

    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok((category_identity_path, title_identity_path))) => {
                if let Some(ref cat_path) = category_identity_path {
                    indexed_category_keys.insert(cat_path.clone());
                }
                indexed_title_keys.insert((category_identity_path, title_identity_path));
            }
            Ok(Err(title_path)) => {
                error!(
                    "can't index title `{}`, check previous logs",
                    title_path.display()
                );
            }
            Err(e) => error!("can't join thread: {e:?}"),
        }
    }

    let read_txn = app_state
        .db
        .content
        .begin_read()
        .okay(|e| error!("can't begin read transaction: {e:?}"))?;

    let indexed_titles_to_chaps = {
        let mut join_set = JoinSet::new();

        let titles_table = Arc::new(
            read_txn
                .open_table(database::content::TITLES)
                .okay(|e| error!("can't open titles table: {e:?}"))?,
        );

        for title_key in indexed_title_keys.iter() {
            let titles_table = titles_table.clone();
            let title_key = title_key.clone();
            join_set.spawn(async move {
                titles_table
                    .get(&title_key)
                    .okay(|e| error!("can't get TitleInfo `{title_key:?}`: {e:?}"))
                    .flatten()
                    .and_then(|info| Some((title_key, info.value().chapters)))
            });
        }

        join_set
            .join_all()
            .await
            .into_iter()
            .filter_map(|t| t)
            .collect::<Vec<_>>()
    };

    let indexed_chaps_to_pages = {
        let indexed_chap_keys = indexed_titles_to_chaps
            .iter()
            .flat_map(
                |((category_identity_path, title_identity_path), chapter_ips)| {
                    if let Some(chapter_ips) = chapter_ips {
                        return chapter_ips
                            .iter()
                            .map(|chapter_identity_path| {
                                chapter_key(
                                    category_identity_path.clone(),
                                    title_identity_path.clone(),
                                    Some(chapter_identity_path.clone()),
                                )
                            })
                            .collect::<Vec<_>>();
                    }

                    vec![chapter_key(
                        category_identity_path.clone(),
                        title_identity_path.clone(),
                        None,
                    )]
                },
            )
            .collect::<Vec<_>>();

        let mut join_set = JoinSet::new();

        let chapters_table = Arc::new(
            read_txn
                .open_table(database::content::CHAPTERS)
                .okay(|e| error!("can't open chapters table: {e:?}"))?,
        );

        for chap_key in indexed_chap_keys {
            let chapters_table = chapters_table.clone();
            join_set.spawn(async move {
                chapters_table
                    .get(&chap_key)
                    .okay(|e| error!("can't get ChapterInfo `{chap_key:?}`: {e:?}"))
                    .flatten()
                    .map(|info| (chap_key, info.value().pages))
            });
        }

        join_set
            .join_all()
            .await
            .into_iter()
            .filter_map(|t| t)
            .collect::<Vec<_>>()
    };

    let indexed_chap_keys = indexed_chaps_to_pages
        .iter()
        .map(|(chap_key, _)| chap_key.clone())
        .collect::<HashSet<_>>();

    let indexed_pages = indexed_chaps_to_pages
        .into_iter()
        .flat_map(
            |((category_identity_path, title_identity_path, chapter_identity_path), page_ips)| {
                page_ips.into_iter().map(move |page_identity_path| {
                    page_key(
                        category_identity_path.clone(),
                        title_identity_path.clone(),
                        chapter_identity_path.clone(),
                        page_identity_path,
                    )
                })
            },
        )
        .collect::<HashSet<_>>();

    let write_txn = app_state
        .db
        .content
        .begin_write()
        .okay(|e| error!("can't begin write transaction: {e:?}"))?;

    let mut categories_table = write_txn
        .open_table(database::content::CATEGORIES)
        .okay(|e| error!("can't open categories table: {e:?}"))?;
    categories_table.retain(|key, _| indexed_category_keys.contains(&key));
    drop(categories_table);

    let mut titles_table = write_txn
        .open_table(database::content::TITLES)
        .okay(|e| error!("can't open titles table: {e:?}"))?;
    titles_table.retain(|key, _| indexed_title_keys.contains(&key));
    drop(titles_table);

    let mut chapters_table = write_txn
        .open_table(database::content::CHAPTERS)
        .okay(|e| error!("can't open chapters table: {e:?}"))?;
    chapters_table.retain(|key, _| indexed_chap_keys.contains(&key));
    drop(chapters_table);

    let mut pages_table = write_txn
        .open_table(database::content::PAGES)
        .okay(|e| error!("can't open pages table: {e:?}"))?;
    pages_table.retain(|key, _| indexed_pages.contains(&key));
    drop(pages_table);

    if let Err(e) = write_txn.commit() {
        error!("can't commit write transaction: {e:?}");
    }

    info!("finished re-indexing library");

    Some(())
}
