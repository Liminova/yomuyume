mod blurhash;
mod dir_entry_guesser;
mod upsert_category;
mod upsert_oneshot;

use std::{
    collections::VecDeque,
    fs::{self, DirEntry},
    mem::drop,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result};
use dir_entry_guesser::{DirEntryType, DirEntryTypeGuesser};
use futures_util::future::join_all;
use tokio::sync::Semaphore;
use tracing::{debug, error, info};

use crate::{
    types::{CategoryID, TitleID},
    AppState, SUPPORTED_ARCHIVE_FORMATS,
};
use upsert_oneshot::{upsert_title, UpsertTitleError, UpsertTitleOk};

#[derive(Debug)]
pub struct LibraryProcessor {
    app_state: Arc<AppState>,
    sem: Arc<Semaphore>,
}

#[derive(Debug)]
pub enum ProcessTitleError {
    CantAcquirePermit((anyhow::Error, PathBuf)),
    CantSpawnTask((anyhow::Error, PathBuf)),
    Other((UpsertTitleError, PathBuf)),
}

type ProcessTitleOk = UpsertTitleOk;

struct ScannedEntry {
    entry: DirEntry,
    parent: Option<PathBuf>,
}

impl LibraryProcessor {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self {
            app_state,
            sem: Arc::new(Semaphore::new(num_cpus::get())),
        }
    }

    // pub async fn process_title(
    //     &self,
    //     title_path: PathBuf,
    // ) -> Result<ProcessTitleOk, ProcessTitleError> {
    //     let app_state_clone = self.app_state.clone();
    //     let sem_clone = self.sem.clone();
    //     let title_path_clone = title_path.clone();

    //     let task = tokio::spawn(async move {
    //         let permit = sem_clone
    //             .acquire()
    //             .await
    //             .context("can't acquire a permit from semaphore")
    //             .map_err(|e| ProcessTitleError::CantAcquirePermit((e, title_path_clone.clone())))?;

    //         let res = upsert_title(app_state_clone, &title_path_clone.clone())
    //             .await
    //             .map_err(|e| ProcessTitleError::Other((e, title_path_clone.clone())));

    //         drop(permit);

    //         res
    //     });

    //     match task.await {
    //         Ok(task) => task,
    //         Err(e) => Err(ProcessTitleError::CantSpawnTask((e.into(), title_path))),
    //     }
    // }

    pub async fn full_scan(&self) -> Result<()> {
        // scan library directory
        let library_dir =
            std::fs::read_dir(&self.app_state.config.library_path).context(format!(
                "can't read library path: {:?}",
                self.app_state.config.library_path
            ))?;

        let mut queue: VecDeque<ScannedEntry> = VecDeque::new();

        // populate first layer of the library tree to queue
        for entry in library_dir {
            let entry = match entry {
                Ok(entry) => entry,
                Err(e) => {
                    error!("can't extract entry from root library: {e:#?}",);
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
            let live_config = self.app_state.live_config.read().await;
            (
                live_config.nomedia_support,
                live_config.komga_oneshot_support,
                live_config.komga_recycle_support,
            )
        };

        // processing tasks waiting to be .await-ed
        // let mut food_processor = vec![];

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
                        println!("TODO: handle one-shot w/ {pages:?}")
                    }
                    DirEntryType::OneShotArchiveFile => {
                        println!("TODO: handle one-shot archive file")
                    }
                    DirEntryType::Ignored => {
                        debug!("ignored directory {}", entry_path.display())
                    }
                },
                Err(e) => {
                    error!(
                        "can't guess the directory type for {}: {e:#?}",
                        entry_path.display()
                    );
                    continue;
                }
            }
        }

        Ok(())

        // if let Err(e) = sqlx::query!(
        //     r#"WITH _ AS (
        //         DELETE FROM categories
        //             WHERE id NOT IN (SELECT id FROM UNNEST($1::bigint[]))
        //     )
        //     DELETE FROM titles
        //         WHERE id NOT IN (SELECT id FROM UNNEST($2::bigint[]))"#,
        //     &category_ids,
        //     &title_ids,
        // )
        // .execute(&self.app_state.pool)
        // .await
        // {
        //     error!("can't cleanup non-exist categories and titles from database: {e:#?}");
        // };

        // info!("finished processing library");
    }
}
