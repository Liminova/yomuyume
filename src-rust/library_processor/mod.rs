mod blurhash;
mod upsert_category;
mod upsert_title;

use std::{
    mem::drop,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, Result};
use futures_util::future::join_all;
use tokio::sync::Semaphore;
use tracing::{error, info};

use crate::{
    types::{CategoryID, TitleID},
    AppState, SUPPORTED_ARCHIVE_FORMATS,
};
use upsert_title::{upsert_title, UpsertTitleError, UpsertTitleOk};

async fn read_dir_iterative(path: &Path) -> Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = Vec::new();
    let mut stack: Vec<PathBuf> = vec![path.to_path_buf()];

    while let Some(current_path) = stack.pop() {
        let mut entries = tokio::fs::read_dir(&current_path)
            .await
            .context("can't read category dir")?;

        while let Some(entry) = entries.next_entry().await.unwrap_or_default() {
            let path = entry.path();
            if path.is_file() {
                files.push(path);
            } else if path.is_dir() {
                stack.push(path);
            }
        }
    }

    Ok(files)
}

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

impl LibraryProcessor {
    pub fn new(app_state: Arc<AppState>) -> Self {
        Self {
            app_state,
            sem: Arc::new(Semaphore::new(num_cpus::get())),
        }
    }

    pub async fn process_title(
        &self,
        title_path: PathBuf,
    ) -> Result<ProcessTitleOk, ProcessTitleError> {
        let app_state_clone = self.app_state.clone();
        let sem_clone = self.sem.clone();
        let title_path_clone = title_path.clone();

        let task = tokio::spawn(async move {
            let permit = sem_clone
                .acquire()
                .await
                .context("can't acquire a permit from semaphore")
                .map_err(|e| ProcessTitleError::CantAcquirePermit((e, title_path_clone.clone())))?;

            let res = upsert_title(app_state_clone, &title_path_clone.clone())
                .await
                .map_err(|e| ProcessTitleError::Other((e, title_path_clone.clone())));

            drop(permit);

            res
        });

        match task.await {
            Ok(task) => task,
            Err(e) => Err(ProcessTitleError::CantSpawnTask((e.into(), title_path))),
        }
    }

    pub async fn full_scan(&self) {
        let files_in_lib = match read_dir_iterative(&self.app_state.config.library_path).await {
            Ok(files) => files
                .into_iter()
                .filter(|title_path| {
                    SUPPORTED_ARCHIVE_FORMATS.contains(
                        &title_path
                            .extension()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default(),
                    )
                })
                .collect::<Vec<_>>(),
            Err(e) => {
                error!("can't scan library: {e:#?}");
                return;
            }
        };
        info!("found {} files in library", files_in_lib.len());

        let upsert_title_results = files_in_lib
            .into_iter()
            .map(|title_path| self.process_title(title_path));

        let mut category_ids: Vec<CategoryID> = Vec::with_capacity(upsert_title_results.len());
        let mut title_ids: Vec<TitleID> = Vec::with_capacity(upsert_title_results.len());

        for result in join_all(upsert_title_results).await {
            match result {
                Ok(result) => {
                    title_ids.push(result.title_id);
                    if let Some(category_id) = result.category_id {
                        category_ids.push(category_id);
                    }
                }
                Err(e) => match e {
                    ProcessTitleError::CantAcquirePermit((e, t)) => {
                        error!("can't acquire a permit to process title {t:?}: {e:#?}")
                    }
                    ProcessTitleError::CantSpawnTask((e, t)) => {
                        error!("can't spawn task to process title {t:?}: {e:#?}")
                    }
                    ProcessTitleError::Other((e, t)) => match e {
                        UpsertTitleError::IsIgnored => info!("title {t:?} is ignored"),
                        UpsertTitleError::Other(error) => {
                            error!("can't process title: {error:#?}")
                        }
                    },
                },
            }
        }

        if let Err(e) = sqlx::query!(
            r#"WITH _ AS (
                DELETE FROM categories
                    WHERE id NOT IN (SELECT id FROM UNNEST($1::bigint[]))
            )
            DELETE FROM titles
                WHERE id NOT IN (SELECT id FROM UNNEST($2::bigint[]))"#,
            &category_ids,
            &title_ids,
        )
        .execute(&self.app_state.pool)
        .await
        {
            error!("can't cleanup non-exist categories and titles from database: {e:#?}");
        };

        info!("finished processing library");
    }
}
