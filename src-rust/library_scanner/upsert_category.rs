use std::{path::Path, sync::Arc};

use anyhow::{anyhow, Context, Result};
use quick_xml::{de::from_str, se::to_string};
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use tracing::warn;

use crate::{
    library_scanner::{blurhash, category_info::CategoryInfo},
    models::prelude::{categories, Categories, CategoryID},
    AppState,
};

/// Upsert a category to the database and return its ID.
pub async fn upsert_category(
    app_state: Arc<AppState>,
    category_dir_path: &Path,
) -> Result<CategoryID> {
    if !category_dir_path.exists() {
        return Err(anyhow!("category dir not found"));
    }
    if !category_dir_path.is_dir() {
        return Err(anyhow!("category dir is not a directory"));
    }

    let category_info_path = category_dir_path.join("CategoryInfo.xml");
    let category_info = {
        if !category_info_path.exists() {
            std::fs::File::create(&category_info_path).context("can't create CategoryInfo.xml")?;
        }
        let xml =
            std::fs::read_to_string(&category_info_path).context("can't read CategoryInfo.xml")?;
        from_str::<CategoryInfo>(&xml).context("can't parse CategoryInfo.xml")?
    };

    let category_id = category_info.id.clone().unwrap_or_default();

    let mut use_insert = true;
    let mut category_model: Option<categories::Model> = None;
    if let Some(ref category_id) = category_info.id {
        if let Some(model) = Categories::find_by_id(category_id)
            .one(&app_state.db)
            .await
            .context("can't find category in database")?
        {
            use_insert = false;
            category_model = Some(model);
        }
    };

    let mut category_active =
        category_model
            .clone()
            .map(|m| m.into())
            .unwrap_or_else(|| categories::ActiveModel {
                id: Set(category_id.clone()),
                name: Set(category_info.name.to_string()),
                description: Set(category_info.description.clone_inner()),
                cover_path: Set(None),
                cover_blurhash: Set(None),
                blurhash_resolution_x: Set(None),
                blurhash_resolution_y: Set(None),
                cover_file_hash: Set(None),
            });

    if let Some(ref category_model) = category_model {
        if category_model.name != category_info.name.to_string() {
            category_active.name = Set(category_info.name.to_string());
        }
    }

    let category_cover_blurhash = category_info
        .cover
        .as_ref()
        .ok_or_else(|| anyhow!("not configured yet"))
        .and_then(|cover_path| {
            image::open(cover_path)
                .context("can't decode")
                .map(|img| (img, cover_path))
        })
        .and_then(|(img, cover_path)| {
            blurhash::encode(&img)
                .context("can't encode to blurhash")
                .map(|cover_blurhash| (cover_blurhash, cover_path))
        });

    match category_cover_blurhash {
        Ok((cover_blurhash, cover_path)) => {
            if let Some(ref category_model) = category_model {
                if category_model.cover_blurhash.as_ref() != Some(&cover_blurhash.blurhash) {
                    category_active.cover_blurhash = Set(Some(cover_blurhash.blurhash.clone()));
                }
                if category_model.blurhash_resolution_x != Some(cover_blurhash.small_width) {
                    category_active.blurhash_resolution_x = Set(Some(cover_blurhash.small_width));
                }
                if category_model.blurhash_resolution_y != Some(cover_blurhash.small_height) {
                    category_active.blurhash_resolution_y = Set(Some(cover_blurhash.small_height));
                }
                if category_model.cover_path != Some(cover_path.to_string_lossy().to_string()) {
                    category_active.cover_path =
                        Set(Some(cover_path.to_string_lossy().to_string()));
                }
            } else {
                category_active.cover_blurhash = Set(Some(cover_blurhash.blurhash));
                category_active.blurhash_resolution_x = Set(Some(cover_blurhash.small_width));
                category_active.blurhash_resolution_y = Set(Some(cover_blurhash.small_height));
                category_active.cover_path = Set(Some(cover_path.to_string_lossy().to_string()));
            }
        }
        Err(e) => {
            if let Some(ref category_model) = category_model {
                if category_model.cover_blurhash.is_some() {
                    category_active.cover_blurhash = Set(None);
                }
                if category_model.blurhash_resolution_x.is_some() {
                    category_active.blurhash_resolution_x = Set(None);
                }
                if category_model.blurhash_resolution_y.is_some() {
                    category_active.blurhash_resolution_y = Set(None);
                }
                if category_model.cover_path.is_some() {
                    category_active.cover_path = Set(None);
                }
            } else {
                category_active.cover_path = Set(None);
                category_active.cover_blurhash = Set(None);
                category_active.blurhash_resolution_x = Set(None);
                category_active.blurhash_resolution_y = Set(None);
            }
            warn!(
                "no cover file for category {}: {e:?}",
                category_dir_path.to_string_lossy()
            );
        }
    };

    if category_active.is_changed() {
        if use_insert {
            category_active.insert(&app_state.db).await?;
        } else {
            category_active.update(&app_state.db).await?;
        }
        let xml = to_string(&category_info)?;
        std::fs::write(&category_info_path, xml).context("can't write to CategoryInfo.xml")?;
    }

    Ok(CategoryID::new())
}
