use std::{collections::VecDeque, fs::DirEntry, sync::Arc};

use sqlx::QueryBuilder;
use tokio::task::JoinSet;
use tracing::{debug, error, info};

use crate::{
    AppState,
    indexer::{
        IndexedContent,
        dir_entry_guesser::{DirEntryType, DirEntryTypeGuesser},
        index_category::index_category,
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

    let mut writer = app_state.indexer.writer.lock().await;

    if writer
        .delete_all_documents()
        .okay(|e| {
            error!("can't clear tantivy index: {e:?}");
        })
        .is_some()
    {
        let _ = writer.commit().okay(|e| {
            error!("can't commit changes to tantivy: {e:?}");
        });
    }
    drop(writer);

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
                        let entry_path = entry_path.clone();
                        let app_state = app_state.clone();

                        queue.push_front(ScannedEntry {
                            entry: sub_entry,
                            parent: Some(entry_path.clone()),
                        });
                        join_set.spawn(async move {
                            index_category(app_state, entry_path.clone())
                                .await
                                .ok_or(entry_path)
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

    app_state
        .indexer
        .writer
        .lock()
        .await
        .commit()
        .okay(|e| error!("can't commit changes to tantivy: {e:?}"));

    let mut indexed_title_ids = Vec::new();
    let mut indexed_category_ids = Vec::new();

    while let Some(result) = join_set.join_next().await {
        match result {
            Ok(Ok(indexed_content)) => match indexed_content {
                IndexedContent::TitleID(title_id) => {
                    indexed_title_ids.push(title_id);
                }
                IndexedContent::CategoryID(category_id) => {
                    indexed_category_ids.push(category_id);
                }
                IndexedContent::CategoryIDs(category_ids) => {
                    indexed_category_ids.extend(category_ids);
                }
            },
            Ok(Err(title_path)) => {
                error!(
                    "can't index title `{}`, check previous logs",
                    title_path.display()
                );
            }
            Err(e) => error!("can't join thread: {e:?}"),
        }
    }

    indexed_title_ids.sort();
    indexed_title_ids.dedup();
    indexed_category_ids.sort();
    indexed_category_ids.dedup();

    let mut delete_invalid_titles_query =
        QueryBuilder::new("DELETE FROM titles WHERE id NOT IN ( ");
    delete_invalid_titles_query
        .push_values(indexed_title_ids.iter(), |mut b, id| {
            b.push_bind(id);
        })
        .push(" )");
    let mut delete_invalid_categories_query =
        QueryBuilder::new("DELETE FROM categories WHERE id NOT IN ( ");
    delete_invalid_categories_query
        .push_values(indexed_category_ids.iter(), |mut b, id| {
            b.push_bind(id);
        })
        .push(" )");

    let mut tx = app_state
        .pool
        .begin()
        .await
        .okay(|e| error!("can't start SQL transaction: {e}"))?;

    delete_invalid_titles_query
        .build()
        .execute(&mut *tx)
        .await
        .okay(|e| error!("can't delete invalid titles: {e}"))?;

    delete_invalid_categories_query
        .build()
        .execute(&mut *tx)
        .await
        .okay(|e| error!("can't delete invalid categories: {e}"))?;

    info!("finished re-indexing library");

    Some(())
}
