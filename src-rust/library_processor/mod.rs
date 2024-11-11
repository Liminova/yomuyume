mod blurhash;
mod dir_entry_guesser;
mod upsert_category;
mod upsert_oneshot;
mod upsert_series;

use std::{
    collections::{HashMap, VecDeque},
    fs::DirEntry,
    mem::drop,
    path::PathBuf,
    sync::Arc,
};

use anyhow::{Context, Result};
use dir_entry_guesser::{DirEntryType, DirEntryTypeGuesser};
use futures_util::future::join_all;
use tokio::sync::{Mutex, Semaphore};
use tracing::{debug, error, info};

use crate::AppState;
use upsert_oneshot::{upsert_oneshot, OneshotType};

#[derive(Debug)]
struct ScannedEntry {
    entry: DirEntry,
    parent: Option<PathBuf>, // None for the library root
}

pub async fn full_scan(app_state: Arc<AppState>) -> Result<()> {
    // scan library directory
    let library_dir = std::fs::read_dir(&app_state.config.library_path).context(format!(
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
    let mut tasks = vec![];
    let category_path_to_id: Arc<Mutex<HashMap<PathBuf, i64>>> =
        Arc::new(Mutex::new(HashMap::new()));

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
                DirEntryType::SeriesDir(chapter_path_and_number) => {
                    println!("TODO: handle series w/ {chapter_path_and_number:?}")
                }
                DirEntryType::OneShotDir(pages) => {
                    tasks.push((
                        upsert_oneshot(
                            app_state.clone(),
                            entry_path.clone(),
                            OneshotType::InDirectory(pages),
                            scanned.parent,
                            category_path_to_id.clone(),
                            nomedia_support,
                        ),
                        entry_path,
                    ));
                }
                DirEntryType::OneShotArchiveFile(archive_file) => {
                    tasks.push((
                        upsert_oneshot(
                            app_state.clone(),
                            entry_path.clone(),
                            OneshotType::InArchive(archive_file),
                            scanned.parent,
                            category_path_to_id.clone(),
                            nomedia_support,
                        ),
                        entry_path,
                    ));
                }
                DirEntryType::Ignored => {
                    debug!("ignored directory {}", entry_path.display())
                }
            },
            Err(e) => {
                error!(
                    "can't guess the directory type for {}: {e:?}",
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
                    error!(
                        "can't process {title_path}: can't acquire a permit from semaphore: {:?}",
                        e
                    );
                    return;
                }
            };

            match task.await {
                Ok(upserted_title_id) => upserted_title_ids.lock().await.push(upserted_title_id),
                Err(e) => match e {
                    upsert_oneshot::UpsertOneshotErr::IsIgnored => {
                        info!("title {title_path} is ignored")
                    }
                    upsert_oneshot::UpsertOneshotErr::IsEmpty => {
                        info!("title {title_path} is empty")
                    }
                    upsert_oneshot::UpsertOneshotErr::Other(e) => {
                        error!("can't process {title_path}: {e:?}")
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
            .lock()
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
