mod blurhash;
mod cover_finder;
mod handle_category;
mod handle_title;
mod scan_category;
mod scan_library;

use self::scan_library::{scan_library, ScannedCategory};
use crate::{models::prelude::*, AppState};
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter};
use std::{path::PathBuf, sync::Arc};

#[derive(Debug)]
pub struct Scanner {
    pub(super) app_state: Arc<AppState>,
    pub(super) categories: Vec<ScannedCategory>,
}

impl Scanner {
    pub async fn new(app_state: Arc<AppState>) -> Self {
        Self {
            app_state: Arc::clone(&app_state),
            categories: scan_library(&app_state.config.library_path).await,
        }
    }
    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut category_ids = Vec::new();

        for category in &self.categories {
            category_ids.push(self.handle_category(category).await?);
        }

        let mut condition = Condition::all();
        for id in &category_ids {
            condition = condition.add(categories::Column::Id.ne(id));
        }

        let _ = Categories::delete_many()
            .filter(condition)
            .exec(&self.app_state.db)
            .await?;

        let titles = Titles::find().all(&self.app_state.db).await?.into_iter();
        for title in titles {
            if !PathBuf::from(&title.path).exists() {
                let _ = Titles::delete_by_id(&title.id)
                    .exec(&self.app_state.db)
                    .await?;
            }
        }

        tracing::info!("✅ finished scanning library");

        let mut scanning_state = self.app_state.scanning_complete.lock().await;
        *scanning_state = true;

        Ok(())
    }
}
