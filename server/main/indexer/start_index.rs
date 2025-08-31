use std::{
    collections::{HashSet, VecDeque},
    fs::DirEntry,
    sync::Arc,
};

use tokio::task::JoinSet;
use tracing::{debug, error, info};

use crate::{
    AppState,
    indexer::{
        dir_entry_guesser::{DirEntryType, DirEntryTypeGuesser},
        index_oneshot::{OneshotType, index_oneshot},
        index_series::index_series,
    },
    utils::{
        absolute_path::{AbsolutePath, AbsolutePathErr, ToAbsolute},
        archive_file::ItemInArchive,
        okay::MapErrorThenOk,
    },
};

#[derive(Debug)]
struct ScannedEntry {
    entry: DirEntry,
    parent: Option<AbsolutePath>, // None for the library root
}

/// ONLY add errors that would need to handle differently, e.g. [`IsIgnored`]
/// would tell the caller the function "failed" because the file is ignored,
/// not something wrong happened.
///
/// [`IsIgnored`]: UpsertTitleError::IsIgnored
#[derive(Debug, thiserror::Error)]
pub enum UpsertTitleErr {
    #[error("it's empty")]
    IsEmpty,

    #[error("can't transform absolute path: {0:?}")]
    PathAbsoluteConv(#[from] AbsolutePathErr),

    #[error("{0:?}")]
    CantReadChapterDir(std::io::Error),
}

type PageInTitle = ItemInArchive;

#[allow(clippy::cognitive_complexity)]
pub async fn start_index(app_state: Arc<AppState>) {
    // scan library directory
    let Some(library_dir) = std::fs::read_dir(app_state.config.library_path.as_ref()).okay(|e| {
        error!(
            "can't read library path `{}`: {e:?}",
            app_state.config.library_path
        );
    }) else {
        return;
    };

    let mut queue: VecDeque<ScannedEntry> = VecDeque::new();

    // populate first layer of the library tree to queue
    for entry in library_dir {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                error!("can't extract entry from root library: {e:?}",);
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
                            .map_err(|e| (e, entry_path))
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
                        .map_err(|e| (e, entry_path))
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
                        .map_err(|e| (e, entry_path))
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

    if app_state.first_time_index_content {
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(Err((e, title_path))) => {
                    error!("failed to index title `{}`: {e:?}", title_path.display());
                }
                Err(e) => {
                    error!("can't join thread: {e:?}");
                }
                _ => (),
            }
        }
    } else {
        let mut indexed_categories_identity_paths = HashSet::new();
        let mut indexed_titles_identity_paths = HashSet::new();

        while let Some(result) = join_set.join_next().await {
            match result {
                Ok(Ok((category_identity_path, title_identity_path))) => {
                    if let Some(cat_path) = category_identity_path {
                        indexed_categories_identity_paths.insert(cat_path);
                    }
                    indexed_titles_identity_paths.insert(title_identity_path);
                }
                Ok(Err((e, title_path))) => {
                    error!("failed to index title `{}`: {e:?}", title_path.display());
                }
                Err(e) => {
                    error!("can't join thread: {e:?}");
                }
            }
        }

        // TODO: rm invalid titles from DB
    }

    info!("finished indexing library");
}
