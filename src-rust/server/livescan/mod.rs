mod blurhash;
mod handle_category;
mod handle_title;
mod scan_category;
mod scan_library;
mod thumbnail_finder;
mod title_ssim_score;

use self::{
    blurhash::Blurhash,
    scan_library::{scan_library, ScannedCategory},
};
use crate::{livescan::title_ssim_score::title_ssim_score, models::prelude::*, AppState};
use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter};
use std::{path::PathBuf, sync::Arc};

#[derive(Debug)]
pub struct Scanner {
    pub(super) app_state: Arc<AppState>,
    pub(super) temp_path: PathBuf,
    pub(super) blurhash: Blurhash,
    pub(super) categories: Vec<ScannedCategory>,
}

impl Scanner {
    pub async fn new(app_state: Arc<AppState>) -> Self {
        let app_state = Arc::clone(&app_state);
        let temp_path = PathBuf::from(&app_state.env.temp_path.clone());
        let ffmpeg_path = app_state.env.ffmpeg_path.clone();
        let djxl_path = app_state.env.djxl_path.clone();
        let ffmpeg_log_path = app_state.env.ffmpeg_log_path.clone();
        let categories = scan_library(&app_state.env.library_path).await;
        Self {
            app_state,
            temp_path,
            blurhash: Blurhash {
                ffmpeg_path,
                djxl_path,
                ffmpeg_log_path,
            },
            categories,
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

        match self.app_state.env.sentence_embedding_model_path.clone() {
            Some(path) => {
                title_ssim_score(&self.app_state.db, path).await?;
            }
            None => {
                tracing::warn!("no sentence embedding model path provided, skipping title relevance score calculation");
            }
        };

        let mut scanning_state = self.app_state.scanning_complete.lock().await;
        *scanning_state = true;

        Ok(())
    }
}
