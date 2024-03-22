use std::{collections::HashSet, fs::File};

use super::{scan_category::ScannedTitle, Scanner};
use crate::{
    livescan::cover_finder::title_cover_finder,
    models::{metadata::TitleMetadata, prelude::*},
};

use murmur3::murmur3_32;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TryIntoModel};
use tracing::info;
use zip::ZipArchive;

type PagePath = String;

impl Scanner {
    pub async fn handle_title(
        &self,
        scanned_title: &ScannedTitle,
        category_id: String,
    ) -> Result<TitleID, Box<dyn std::error::Error + Send + Sync>> {
        info!("✅ found title: {}", scanned_title.path.to_string_lossy());

        let mut title_metadata = TitleMetadata::from(&scanned_title.path);

        let title_name = match title_metadata.title.clone() {
            Some(title) => Ok(title),
            None => scanned_title
                .path
                .file_stem()
                .and_then(|s| s.to_string_lossy().to_string().into())
                .ok_or_else(|| "can't get name".to_string()),
        }?;

        let mut is_new = false; // to decide to use SeaORM's insert or update
        let (title_model, mut title_active): (titles::Model, titles::ActiveModel) = 'scoped: {
            let title_file = tokio::fs::read(&scanned_title.path).await?;
            let current_hash = match murmur3_32(&mut &title_file[..], 0) {
                Ok(hash) => hash.to_string(),
                Err(e) => return Err(format!("can't hash: {}", e).into()),
            };

            let find_by_path = Titles::find()
                .filter(titles::Column::Path.eq(scanned_title.path_lossy()))
                .one(&self.app_state.db)
                .await
                .map_err(|e| format!("can't find by path: {}", e))?;

            if let Some(title_model) = find_by_path {
                let mut title_active: titles::ActiveModel = title_model.clone().into();
                if title_model.hash != current_hash {
                    title_active.hash = Set(current_hash)
                }
                break 'scoped (title_model, title_active);
            }

            let find_by_hash = Titles::find()
                .filter(titles::Column::Hash.eq(&current_hash))
                .one(&self.app_state.db)
                .await
                .map_err(|e| format!("can't find by hash: {}", e))?;

            if let Some(title_model) = find_by_hash {
                let mut title_active: titles::ActiveModel = title_model.clone().into();
                title_active.path = Set(scanned_title.path_lossy());
                break 'scoped (title_model, title_active);
            }

            is_new = true;
            let new = titles::ActiveModel {
                id: Set(TitleID::new()),
                hash: Set(current_hash),
                path: Set(scanned_title.path_lossy()),
                date_added: Set(chrono::Utc::now().timestamp().to_string()),
                date_updated: Set(chrono::Utc::now().timestamp().to_string()),

                title: Set(String::new()),
                category_id: Set(String::new()),
                author: Set(None),
                description: Set(None),
                release: Set(None),
            };
            (
                new.clone()
                    .try_into_model()
                    .map_err(|e| format!("can't convert to model: {}", e))?,
                new,
            )
        };

        if title_model.title != title_name {
            title_active.title = Set(title_name.clone());
        }
        if title_model.category_id != category_id {
            title_active.category_id = Set(category_id.clone());
        }
        if title_model.author != title_metadata.author {
            title_active.author = Set(title_metadata.author.clone());
        }
        if title_model.description != title_metadata.description {
            title_active.description = Set(title_metadata.description.clone());
        }
        if title_model.release != title_metadata.release {
            title_active.release = Set(title_metadata.release.clone());
        }
        if title_model.path != scanned_title.path_lossy() {
            title_active.path = Set(scanned_title.path_lossy());
        }
        if title_active.is_changed() {
            title_active.date_updated = Set(chrono::Utc::now().timestamp().to_string());
        }
        if is_new {
            title_active
                .insert(&self.app_state.db)
                .await
                .map_err(|e| format!("can't insert to DB: {}", e))?;
        } else {
            title_active
                .update(&self.app_state.db)
                .await
                .map_err(|e| format!("can't update in DB: {}", e))?;
        }

        'update_cover: {
            let cover_model = Covers::find()
                .filter(covers::Column::Id.eq(&title_model.id))
                .one(&self.app_state.db)
                .await
                .map_err(|e| format!("can't find cover in DB: {}", e))?;

            if let Some(cover_model) = cover_model {
                if Some(cover_model.path) == title_metadata.cover {
                    break 'update_cover;
                }
            }

            let _ = Covers::delete_by_id(&title_model.id)
                .exec(&self.app_state.db)
                .await
                .map_err(|e| format!("can't delete cover in DB: {}", e))?;

            let cover = title_cover_finder(
                &self.temp_dir,
                &scanned_title.path,
                &title_metadata.cover,
                &self.blurhash,
            )
            .await;

            if let Some(cover) = cover {
                title_metadata.set_cover(cover.file_name.clone());
                let cover_active = covers::ActiveModel {
                    id: Set(title_model.id.clone()),
                    path: Set(cover.file_name),
                    blurhash: Set(cover.blurhash),
                    ratio: Set(cover.ratio),
                };
                cover_active
                    .insert(&self.app_state.db)
                    .await
                    .map_err(|e| format!("can't insert cover to DB: {}", e))?;
            }
        }

        let pages_in_file: HashSet<PagePath> = {
            let reader = File::open(&scanned_title.path)
                .map_err(|e| format!("can't read title file: {}", e))?;
            let mut archive = ZipArchive::new(reader)
                .map_err(|e| format!("can't read title file as zip: {}", e))?;
            (0..archive.len())
                .map(|i| {
                    archive
                        .by_index(i)
                        .map_err(|e| format!("can't read title file as zip: {}", e))
                        .map(|f| f.name().to_string())
                })
                .collect::<Result<_, _>>()?
        };

        let pages_in_db: HashSet<PagePath> = Pages::find()
            .filter(pages::Column::TitleId.eq(&title_model.id))
            .all(&self.app_state.db)
            .await
            .map_err(|e| format!("can't find pages in DB: {}", e))?
            .into_iter()
            .map(|p| p.path)
            .collect();

        let to_be_delete: Vec<&PagePath> = pages_in_db.difference(&pages_in_file).collect();
        let to_be_insert: Vec<&PagePath> = pages_in_file.difference(&pages_in_db).collect();
        let to_be_update: Vec<&PagePath> = pages_in_file.intersection(&pages_in_db).collect();

        for page in to_be_delete {
            Pages::delete_many()
                .filter(pages::Column::TitleId.eq(&title_model.id))
                .filter(pages::Column::Path.eq(page))
                .exec(&self.app_state.db)
                .await
                .map_err(|e| format!("can't delete pages in DB: {}", e))?;
        }

        for page in to_be_insert {
            let page_active = pages::ActiveModel {
                id: Set(PageID::new()),
                title_id: Set(title_model.id.clone()),
                path: Set(page.clone()),
                description: Set(title_metadata.get_page_desc(page)),
            };
            page_active
                .insert(&self.app_state.db)
                .await
                .map_err(|e| format!("can't insert pages to DB: {}", e))?;
        }

        for page in to_be_update {
            let page_model = Pages::find()
                .filter(pages::Column::Path.eq(page))
                .one(&self.app_state.db)
                .await
                .map_err(|e| format!("can't find page in DB: {}", e))?;

            if let Some(page_model) = page_model {
                let page_desc = title_metadata.get_page_desc(page);
                if page_model.description == page_desc {
                    continue;
                }

                let mut page_active: pages::ActiveModel = page_model.into();
                page_active.description = Set(page_desc);
                page_active
                    .update(&self.app_state.db)
                    .await
                    .map_err(|e| format!("can't update page in DB: {}", e))?;
            }
        }

        Ok(title_model.id)
    }
}
