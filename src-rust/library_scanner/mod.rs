pub mod blurhash;
mod upsert_category;
mod upsert_title;

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use tracing::debug;

use crate::{types::custom_id::CategoryID, AppState};
use upsert_category::upsert_category;
use upsert_title::upsert_title;

#[derive(Debug)]
pub struct Scanner {
    app_state: Arc<AppState>,
}

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

impl Scanner {
    pub async fn new(app_state: Arc<AppState>) -> Self {
        Self {
            app_state: Arc::clone(&app_state),
        }
    }

    pub async fn run(&self) -> Result<()> {
        let files_in_lib = read_dir_iterative(&self.app_state.config.library_path).await?;
        debug!("found {} files in library", files_in_lib.len());

        let mut category_path_id_map: HashMap<PathBuf, CategoryID> = HashMap::new();

        for title_path in files_in_lib {
            if !title_path
                .extension()
                .map(|e| e.to_string_lossy() == "zip")
                .unwrap_or(false)
            {
                debug!("skipping {}", title_path.display());
                continue;
            }
            debug!("processing {}", title_path.display());

            // title inside a subdir -> in a category
            // title inside library root -> no category
            let category_path = title_path
                .parent()
                .filter(|p| {
                    p.strip_prefix(&self.app_state.config.library_path)
                        .map(|p| !p.to_string_lossy().to_string().is_empty())
                        .unwrap_or(false)
                })
                .map(PathBuf::from);

            if let Some(category_path) = category_path {
                let category_id = category_path_id_map
                    .entry(category_path.clone())
                    .or_insert(
                        upsert_category(self.app_state.clone(), &category_path)
                            .await
                            .context(format!(
                                "can't insert category to database: {}",
                                category_path.display()
                            ))?,
                    )
                    .clone();

                upsert_title(self.app_state.clone(), Some(category_id), &title_path)
                    .await
                    .context(format!(
                        "can't insert title to database: {}",
                        title_path.display()
                    ))?;

                continue;
            }

            upsert_title(self.app_state.clone(), None, &title_path)
                .await
                .context(format!(
                    "can't insert title to database: {}",
                    title_path.display()
                ))?;
        }

        Ok(())
    }
}
