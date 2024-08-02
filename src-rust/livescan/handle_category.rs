use crate::{
    livescan::{
        cover_finder::category_cover_finder, scan_category::scan_category,
        scan_library::ScannedCategory, Scanner,
    },
    models::{metadata::CategoryMetadata, prelude::*},
};

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use std::path::PathBuf;
use tracing::{debug, error, info, warn};

impl Scanner {
    pub async fn handle_category(
        &self,
        category: &ScannedCategory,
    ) -> Result<CategoryID, Box<dyn std::error::Error>> {
        info!("✅ found category: {}", category.path.to_string_lossy());

        /* pre-cleanup to make sure there's no residual temp category */
        let mut temp_dir_category: PathBuf = PathBuf::from("/tmp");
        temp_dir_category.push(&category.name);
        match std::fs::remove_dir_all(&temp_dir_category) {
            Ok(_) => (),
            Err(e) => tracing::warn!("failed to clean up temp dir: {}", e),
        }

        /* read <category_folder>.toml */
        let mut category_metadata = CategoryMetadata::from(&category.path)?;

        let category_id = match category_metadata.id {
            Some(id) => id,
            None => {
                let id = CategoryID::new();
                category_metadata.set_id(id.clone())?;
                id
            }
        };

        /* category's name = folder name || metadata */
        let category_name = match category_metadata.name {
            Some(name) => name,
            None => category
                .path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
        };

        /* #region - insert/update category info to DB */
        let category_model = Categories::find_by_id(&category_id)
            .one(&self.app_state.db)
            .await
            .map_err(|e| format!("can't search category in DB: {}", e))?;

        if let Some(category_model) = category_model {
            debug!("category already exists in DB");
            let mut active_category: categories::ActiveModel = category_model.into();
            active_category.name = Set(category_name.clone());
            active_category.description = Set(category_metadata.description);
            let _ = active_category
                .update(&self.app_state.db)
                .await
                .map_err(|e| {
                    error!("error update category in DB: {}", e);
                    e
                })?;
        } else {
            debug!("category not exists in DB, insert");
            let _ = categories::ActiveModel {
                id: Set(category_id.clone()),
                name: Set(category_name),
                description: Set(category_metadata.description),
            }
            .insert(&self.app_state.db)
            .await
            .map_err(|e| {
                error!("error insert to DB: {}", e);
                e
            })?;
        }
        /* #endregion */

        /* #region - find category cover */
        let category_cover = category_cover_finder(
            &self.app_state.config,
            &category.path,
            &category_metadata.cover,
        );
        /* #endregion */

        /* #region - push cover to DB if needed */
        if let Ok(cover) = category_cover {
            info!("- cover found: {}", cover.image_path.to_string_lossy());

            // check if exists in DB by blurhash
            let cover_in_db = Covers::find()
                .filter(covers::Column::Blurhash.eq(&cover.blurhash))
                .one(&self.app_state.db)
                .await
                .map_err(|e| format!("can't check cover existence in DB: {}", e))?;

            // exists ? update path (blurhash same => dimensions same, no guarantee for path) : insert
            if let Some(cover_in_db) = cover_in_db {
                debug!("cover already exists in DB");
                let mut active_cover: covers::ActiveModel = cover_in_db.into();
                active_cover.path = Set(cover.image_path.to_string_lossy().to_string());
                let _ = active_cover
                    .update(&self.app_state.db)
                    .await
                    .map_err(|e| format!("can't update cover path in DB: {}", e))?;
            } else {
                debug!("cover not exists in DB, insert");
                let _ = Covers::delete_many()
                    .filter(covers::Column::Id.eq(&category_id))
                    .exec(&self.app_state.db)
                    .await
                    .map_err(|e| {
                        error!("error delete cover in DB: {}", e);
                        e
                    })?;
                let _ = covers::ActiveModel {
                    id: Set(category_id.clone()),
                    path: Set(cover.image_path.to_string_lossy().into_owned()),
                    blurhash: Set(cover.blurhash),
                    ratio: Set(cover.ratio),
                }
                .insert(&self.app_state.db)
                .await
                .map_err(|e| {
                    error!("error insert cover to DB: {}", e);
                    e
                })?;
            }
        } else {
            warn!("- cover not found");
        }
        /* #endregion */

        /* handle titles */
        let titles = scan_category(&category.path).await;
        let titles_count = titles.len();
        let mut processed = 0;
        for title in titles {
            if let Err(e) = self.handle_title(&title, &category_id).await {
                error!("can't process title: {}", e);
            }

            processed += 1;
            let progress = processed as f64 / titles_count as f64;
            let mut scanning_progress = self.app_state.scanning_progress.lock().await;
            *scanning_progress = progress;
        }

        /* cleanup */
        let mut temp_dir_category: PathBuf = PathBuf::from("/tmp");
        temp_dir_category.push(&category.name);
        match std::fs::remove_dir_all(&temp_dir_category) {
            Ok(_) => (),
            Err(e) => tracing::warn!("failed to clean up temp dir: {}", e),
        }

        Ok(category_id)
    }
}
