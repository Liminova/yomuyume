use super::{scan_category::ScannedTitle, Scanner};
use crate::{
    livescan::thumbnail_finder::title_thumbnail_finder,
    models::{metadata::TitleMetadata, prelude::*},
};
#[cfg(target_pointer_width = "64")]
use murmur3::murmur3_x64_128 as murmur3_128;
#[cfg(target_pointer_width = "32")]
use murmur3::murmur3_x86_128 as murmur3_128;
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, EntityTrait, QueryFilter, Set};
use std::{fs::File, path::PathBuf};
use tracing::{debug, error, info};
use uuid::Uuid;
use zip::ZipArchive;

impl Scanner {
    pub async fn handle_title(
        &self,
        title: &ScannedTitle,
        category_id: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("✅ found title: {}", title.path.to_string_lossy());

        /* #region - read <title>.toml */
        let mut title_metadata = TitleMetadata::from(&{
            let mut title_metadata_path = title.path.clone();
            title_metadata_path.set_extension("toml");
            title_metadata_path
        })
        .await;
        /* #endregion */

        /* #region - title's name defined in <title>.toml ? use it : use title file_stem */
        let title_name = match title_metadata.title.clone() {
            Some(title) => title,
            None => title
                .path
                .file_stem()
                .ok_or_else(|| {
                    error!("error getting title name");
                    "error getting title name"
                })?
                .to_string_lossy()
                .to_string(),
        };
        debug!("title | {:?}", &title_name);
        /* #endregion */

        /* #region - check if title exist; gen uuid if needed */
        let title_hash_current = {
            let content = tokio::fs::read(&title.path).await?;
            match murmur3_128(&mut &content[..], 0) {
                Ok(hash) => hash.to_string(),
                Err(e) => {
                    error!("error hashing: {}", e);
                    return Err(e.into());
                }
            }
        };
        let mut title_path_exist_in_db = false;
        let mut title_id = String::new();

        // By path -> hash change ? by hash : update metadata -> return
        match Titles::find()
            .filter(titles::Column::Path.eq(title.path_lossy()))
            .one(&self.app_state.db)
            .await
        {
            Ok(Some(title_model)) => {
                title_path_exist_in_db = true;
                title_id = title_model.id.clone();

                tracing::debug!("hash in db: {}", title_model.hash);
                tracing::debug!("hash current: {}", title_hash_current);

                /* update fields if metadata changed */
                'scoped: {
                    let mut need_update = false;
                    let mut active_title: titles::ActiveModel =
                        match Titles::find_by_id(&title_model.id)
                            .one(&self.app_state.db)
                            .await
                        {
                            Ok(Some(title_model)) => title_model.into(),
                            Ok(None) => {
                                error!("error search title in DB, this should not happend");
                                break 'scoped;
                            }
                            Err(e) => {
                                error!("error search title in DB: {}", e);
                                break 'scoped;
                            }
                        };
                    if title_model.title != title_name {
                        need_update = true;
                        active_title.title = Set(title_name.clone());
                    }
                    if title_model.category_id != category_id {
                        need_update = true;
                        active_title.category_id = Set(category_id.clone());
                    }
                    if title_model.description != title_metadata.description {
                        need_update = true;
                        active_title.description = Set(title_metadata.description.clone());
                    }
                    if title_model.author != title_metadata.author {
                        need_update = true;
                        active_title.author = Set(title_metadata.author.clone());
                    }
                    if title_model.release != title_metadata.release_date {
                        need_update = true;
                        active_title.release = Set(title_metadata.release_date.clone());
                    }
                    if title_model.hash != title_hash_current {
                        need_update = true;
                        active_title.date_updated = Set(chrono::Utc::now().timestamp().to_string());
                    }
                    if !need_update {
                        break 'scoped;
                    }
                    match active_title.update(&self.app_state.db).await {
                        Ok(_) => {}
                        Err(e) => {
                            error!("error update metadata in DB: {}", e);
                            break 'scoped;
                        }
                    }
                }

                /* update thumbnail if metadata changed */
                'scoped: {
                    let thumbnail_filename_in_db = match Thumbnails::find()
                        .filter(thumbnails::Column::Id.eq(&title_id))
                        .one(&self.app_state.db)
                        .await
                    {
                        Ok(Some(thumbnail_model)) => Some(thumbnail_model.path),
                        Ok(None) => {
                            break 'scoped;
                        }
                        Err(e) => {
                            error!("error search thumbnail in DB: {}", e);
                            break 'scoped;
                        }
                    };

                    if thumbnail_filename_in_db != title_metadata.thumbnail {
                        info!("thumbnail in DB != in metadata, re-encoding");
                        match Thumbnails::delete_many()
                            .filter(thumbnails::Column::Id.eq(&title_id))
                            .exec(&self.app_state.db)
                            .await
                        {
                            Ok(_) => {}
                            Err(e) => {
                                error!("error delete thumbnail in DB: {}", e);
                                break 'scoped;
                            }
                        };
                        self.update_thumbnail(&mut title_metadata, &title_id, &title.path)
                            .await?;
                    }
                }

                /* update pages' descs if metadata changed */
                'scoped: {
                    let page_models = match Pages::find()
                        .filter(pages::Column::TitleId.eq(&title_id))
                        .all(&self.app_state.db)
                        .await
                    {
                        Ok(page_models) => page_models,
                        Err(e) => {
                            error!("error search pages in DB: {}", e);
                            break 'scoped;
                        }
                    };
                    'iteration: for page in page_models {
                        let page_desc_metadata = title_metadata.get_page_desc(page.path.as_str());
                        if page.description == page_desc_metadata {
                            continue 'iteration;
                        }
                        let mut active_page: pages::ActiveModel = page.into();
                        active_page.description = Set(page_desc_metadata);
                        match active_page.update(&self.app_state.db).await {
                            Ok(_) => {}
                            Err(e) => {
                                error!("error update page in DB: {}", e);
                                break 'scoped;
                            }
                        };
                    }
                }

                if title_model.hash == title_hash_current {
                    info!("found in DB by path, hash match, skipping");
                    return Ok(());
                }
                info!("found in DB by path, hash not match, finding hash");
            }
            Ok(None) => {
                info!("not found in DB by path, finding hash");
            }
            Err(e) => {
                error!("error search title in DB: {}", e);
                return Err(e.into());
            }
        }

        // By hash -> found match ? update metadata to match : encode -> return
        // Found match means nothing in the title.zip changed, so we can skip encoding pages
        match Titles::find()
            .filter(titles::Column::Hash.eq(&title_hash_current))
            .one(&self.app_state.db)
            .await
        {
            Ok(Some(found_title_in_db)) => {
                info!("found in DB by hash, updating metadata and skipping encoding pages");

                let mut active_title: titles::ActiveModel = found_title_in_db.into();
                active_title.title = Set(title_name);
                active_title.category_id = Set(category_id.clone());
                active_title.description = Set(title_metadata.description.clone());
                active_title.author = Set(title_metadata.author.clone());
                active_title.release = Set(title_metadata.release_date.clone());
                active_title.date_updated = Set(chrono::Utc::now().timestamp().to_string());

                let _ = active_title.update(&self.app_state.db).await.map_err(|e| {
                    error!("error update metadata in DB: {}", e);
                    e
                })?;

                return Ok(()); // return this handle_title function
            }
            Ok(None) => {
                info!("not found in DB by hash, inserting title to DB and encoding pages");
            }
            Err(e) => {
                error!("error check if exist in DB: {}", e);
                return Err(e.into());
            }
        }
        /* #endregion */

        /* #region - create if title is new, else update hash */
        if !title_path_exist_in_db {
            title_id = Uuid::new_v4().to_string();
            let now = chrono::Utc::now().timestamp().to_string();
            let _ = titles::ActiveModel {
                id: Set(title_id.clone()),
                category_id: Set(category_id),
                description: Set(title_metadata.description.clone()),
                title: Set(title_name),
                author: Set(title_metadata.author.clone()),
                release: Set(title_metadata.release_date.clone()),
                path: Set(title.path_lossy()),
                hash: Set(title_hash_current),
                date_added: Set(now.clone()),
                date_updated: Set(now),
            }
            .insert(&self.app_state.db)
            .await
            .map_err(|e| {
                error!("error inserting title to DB: {}", e);
                e
            })?;
        } else {
            let mut active_title: titles::ActiveModel = Titles::find_by_id(&title_id)
                .one(&self.app_state.db)
                .await
                .map_err(|e| {
                    error!("error search title in DB: {}", e);
                    e
                })?
                .ok_or_else(|| {
                    let err_msg = "error search title in DB, this should not happend";
                    error!("{}", err_msg);
                    err_msg
                })?
                .into();
            active_title.hash = Set(title_hash_current);
            let _ = active_title.update(&self.app_state.db).await.map_err(|e| {
                error!("error update hash in DB: {}", e);
                e
            })?;
        }
        /* #endregion */

        /* tags */
        'scoped: {
            if title_metadata.tags.is_none() {
                break 'scoped;
            }
            let tags = title_metadata.tags.clone().unwrap_or_default();
            if tags.is_empty() {
                break 'scoped;
            }
            'iteration: for tag in tags {
                let tag_model = Tags::find()
                    .filter(tags::Column::Name.eq(&tag))
                    .one(&self.app_state.db)
                    .await
                    .map_err(|e| {
                        error!("error finding tag: {}", e);
                        e
                    });

                let tag_model = match tag_model {
                    Ok(Some(tag_model)) => Some(tag_model),
                    Ok(None) => None,
                    Err(_) => continue 'iteration,
                };

                let tag_id = match tag_model {
                    Some(tag_model) => tag_model.id,
                    None => {
                        let active_tag = tags::ActiveModel {
                            id: NotSet,
                            name: Set(tag.clone()),
                        };

                        let result = Tags::insert(active_tag).exec(&self.app_state.db).await;

                        match result {
                            Ok(result) => result.last_insert_id,
                            Err(e) => {
                                error!("error inserting tag: {}", e);
                                continue 'iteration;
                            }
                        }
                    }
                };

                let result = titles_tags::ActiveModel {
                    id: NotSet,
                    title_id: Set(title_id.clone()),
                    tag_id: Set(tag_id),
                }
                .insert(&self.app_state.db)
                .await;
                match result {
                    Ok(_) => {}
                    Err(e) => {
                        error!("error inserting title_tag: {}", e);
                        continue 'iteration;
                    }
                };
            }
        }

        /* #region - pages */
        let _ = Pages::delete_many()
            .filter(pages::Column::TitleId.eq(&title_id))
            .exec(&self.app_state.db)
            .await
            .map_err(|e| {
                error!("error deleting pages in DB: {}", e);
                e
            })?;

        let pages = list_files_in_zip(&title.path)?;
        debug!("file_names: {:?}", pages);

        'iteration: for page in &pages {
            let result = pages::ActiveModel {
                id: Set(Uuid::new_v4().to_string()),
                title_id: Set(title_id.clone()),
                path: Set(page.clone()),
                description: Set(title_metadata.get_page_desc(page)),
            }
            .insert(&self.app_state.db)
            .await;
            match result {
                Ok(_) => {}
                Err(e) => {
                    error!("error inserting page to DB: {}", e);
                    continue 'iteration;
                }
            };
        }
        /* #endregion */

        /* thumbnail */
        self.update_thumbnail(&mut title_metadata, &title_id, &title.path)
            .await?;

        Ok(())
    }

    async fn update_thumbnail(
        &self,
        title_metadata: &mut TitleMetadata,
        title_id: &str,
        title_path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let thumbnail_model = Thumbnails::find_by_id(title_id)
            .one(&self.app_state.db)
            .await
            .map_err(|e| {
                error!("error search thumbnail in DB: {}", e);
                e
            })?;

        if let Some(thumbnail_model) = thumbnail_model {
            if Some(thumbnail_model.path) == title_metadata.thumbnail {
                info!("thumbnail in DB == thumbnail in metadata, skipping");
                return Ok(());
            }
        }

        let _ = Thumbnails::delete_by_id(title_id)
            .exec(&self.app_state.db)
            .await
            .map_err(|e| {
                error!("error delete thumbnail in DB: {}", e);
                e
            })?;

        let thumbnail = title_thumbnail_finder(
            &self.temp_path,
            title_path,
            &title_metadata.thumbnail,
            &self.blurhash,
        )
        .await;

        // write BHResult -> <title>.toml and DB
        if let Some(thumbnail) = thumbnail {
            title_metadata.set_thumbnail(thumbnail.file_name.clone());
            let _ = thumbnails::ActiveModel {
                id: Set(title_id.to_string()),
                path: Set(thumbnail.file_name),
                blurhash: Set(thumbnail.blurhash),
                ratio: Set(thumbnail.ratio),
            }
            .insert(&self.app_state.db)
            .await
            .map_err(|e| {
                error!("error inserting thumbnail to DB: {}", e);
                e
            })?;
        }

        Ok(())
    }
}

fn list_files_in_zip(path: &PathBuf) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let reader = File::open(path).map_err(|e| {
        error!("error openning title: {}", e);
        e
    })?;

    let mut archive = ZipArchive::new(reader)?;

    let mut file_names = Vec::new();
    for i in 0..archive.len() {
        let file = archive.by_index(i).map_err(|e| {
            error!("error reading zip: {}", e);
            e
        })?;
        file_names.push(file.name().to_string());
    }

    Ok(file_names)
}
