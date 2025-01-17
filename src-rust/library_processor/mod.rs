mod blurhash_encode;
mod dir_entry_guesser;
mod upsert_category;
mod upsert_oneshot;
mod upsert_series;
mod upsert_tags;

use std::{
    collections::{HashMap, VecDeque},
    fs::DirEntry,
    mem::drop,
    path::PathBuf,
    sync::Arc,
};

use anyhow::{Context, Result};
use dir_entry_guesser::{DirEntryType, DirEntryTypeGuesser};
use futures_core::future::BoxFuture;
use futures_util::future::join_all;
use tokio::sync::{Mutex, RwLock, Semaphore};
use tracing::{debug, error, info};
use upsert_series::upsert_series;

use crate::{
    types::absolute_path::AbsolutePath,
    utils::{app_state::AppState, archive_file::ItemInArchive},
};
use upsert_oneshot::{upsert_oneshot, OneshotType};

#[derive(Debug)]
struct ScannedEntry {
    entry: DirEntry,
    parent: Option<PathBuf>, // None for the library root
}

/// ONLY add errors that would need to handle differently, e.g. [`IsIgnored`]
/// would tell the caller the function "failed" because the file is ignored,
/// not something wrong happened.
///
/// [`IsIgnored`]: UpsertTitleError::IsIgnored
#[derive(Debug, thiserror::Error)]
enum UpsertTitleError {
    #[error("content file is empty")]
    IsEmpty,
    #[error("other error: {0:?}")]
    Other(anyhow::Error),
}

impl From<anyhow::Error> for UpsertTitleError {
    fn from(e: anyhow::Error) -> Self {
        Self::Other(e)
    }
}

type PageInTitle = ItemInArchive;

pub async fn full_scan(app_state: Arc<AppState>) -> Result<()> {
    // scan library directory
    let library_dir =
        std::fs::read_dir(app_state.config.library_path.as_ref()).context(format!(
            "can't read library path: {:?}",
            app_state.config.library_path
        ))?;

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

    // extract necessary values, avoid read lock
    let (nomedia_support, komga_oneshot_support, komga_recycle_support) = {
        let live_config = app_state.live_config.read().await;
        (
            live_config.nomedia_support,
            live_config.komga_oneshot_support,
            live_config.komga_recycle_support,
        )
    };

    // processing tasks waiting to be .await-ed
    let mut tasks: Vec<(BoxFuture<'static, Result<i64, UpsertTitleError>>, PathBuf)> = vec![];
    let category_path_to_id: Arc<RwLock<HashMap<AbsolutePath, i64>>> =
        Arc::new(RwLock::new(HashMap::new()));

    // BFS
    while let Some(scanned) = queue.pop_back() {
        let entry_path = scanned.entry.path();
        match scanned.entry.guess(
            nomedia_support,
            komga_oneshot_support,
            komga_recycle_support,
        ) {
            Ok(entry_type) => match entry_type {
                DirEntryType::CategoryDir(sub_entries) => {
                    sub_entries.into_iter().for_each(|sub_entry| {
                        queue.push_front(ScannedEntry {
                            entry: sub_entry,
                            parent: Some(entry_path.clone()),
                        });
                    });
                }
                DirEntryType::SeriesDir(chapters) => {
                    tasks.push((
                        Box::pin(upsert_series(
                            app_state.clone(),
                            entry_path.clone(),
                            chapters,
                            scanned.parent,
                            category_path_to_id.clone(),
                            nomedia_support,
                        )),
                        entry_path,
                    ));
                }
                DirEntryType::OneShotDir(pages) => {
                    tasks.push((
                        Box::pin(upsert_oneshot(
                            app_state.clone(),
                            entry_path.clone(),
                            OneshotType::Directory(pages),
                            scanned.parent,
                            category_path_to_id.clone(),
                            nomedia_support,
                        )),
                        entry_path,
                    ));
                }
                DirEntryType::OneShotArchiveFile(files_in_archive) => {
                    tasks.push((
                        Box::pin(upsert_oneshot(
                            app_state.clone(),
                            entry_path.clone(),
                            OneshotType::Archive(files_in_archive),
                            scanned.parent,
                            category_path_to_id.clone(),
                            nomedia_support,
                        )),
                        entry_path,
                    ));
                }
                DirEntryType::Ignored => {
                    debug!("ignored entry `{}`", entry_path.display())
                }
            },
            Err(e) => {
                error!(
                    "can't guess the directory type for `{}`: {e:?}",
                    entry_path.display()
                );
                continue;
            }
        }
    }

    let sem = Arc::new(Semaphore::new(num_cpus::get()));
    let mut futs = vec![];
    let upserted_title_ids = Arc::new(Mutex::new(vec![]));

    for (task, title_path) in tasks {
        let sem = sem.clone();
        let upserted_title_ids = upserted_title_ids.clone();
        futs.push(tokio::spawn(async move {
            let title_path = title_path.display();

            let permit = match sem.acquire().await {
                Ok(permit) => permit,
                Err(e) => {
                    error!("can't process `{title_path}`: can't acquire a permit from semaphore: {e:?}");
                    return;
                }
            };

            match task.await {
                Ok(upserted_title_id) => upserted_title_ids.lock().await.push(upserted_title_id),
                Err(e) => match e {
                    UpsertTitleError::IsEmpty => {
                        info!("title `{title_path}` is empty")
                    }
                    UpsertTitleError::Other(e) => {
                        error!("can't process `{title_path}`: {e:?}")
                    }
                },
            }
            drop(permit);
        }));
    }

    for join_err in join_all(futs).await {
        if let Err(e) = join_err {
            error!("can't join task: {e:?}");
        }
    }

    if let Err(e) = sqlx::query!(
        "WITH _ AS (
            DELETE FROM categories
                WHERE id NOT IN (SELECT id FROM UNNEST($1::bigint[]))
        )
        DELETE FROM titles
            WHERE id NOT IN (SELECT id FROM UNNEST($2::bigint[]))",
        &category_path_to_id
            .read()
            .await
            .values()
            .copied()
            .collect::<Vec<i64>>(),
        &*upserted_title_ids.lock().await,
    )
    .execute(&app_state.pool)
    .await
    {
        error!("can't cleanup non-exist categories and titles from database: {e:?}");
    };

    info!("finished processing library");

    Ok(())
}
