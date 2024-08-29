// NEW AND IMPROVED

use std::{collections::HashSet, fs::File, io::Read, path::PathBuf, sync::Arc};

use anyhow::anyhow;
use murmur3::murmur3_32;
use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, Condition, EntityTrait, QueryFilter, Set,
    TryIntoModel,
};
use tracing::warn;
use zip::ZipArchive;

use crate::{
    library_scanner::{
        blurhash::{encode, BlurhashResult},
        title_metadata::TitleMetadata,
    },
    models::prelude::*,
    AppError, AppState,
};

/// One way sync from title.zip + title.toml -> database
pub async fn title_to_db(
    app_state: Arc<AppState>,
    category_id: &CustomID,
    content_file_path: &PathBuf,
) -> Result<(TitleID, TitleMetadata), AppError> {
    if !content_file_path.exists() {
        return Err(AppError::from(anyhow!("content file not found")));
    }
    // micro optimization, this value is used frequently
    let title_file_path_string = content_file_path.to_string_lossy().to_string();

    let metadata = TitleMetadata::load(&content_file_path)?;

    // #region - getting up-to-date models

    // true when
    // - content file's hash changed
    // - in metadata file, either "cover" or "descriptions" changed
    let mut need_scan_content_file = false;
    let current_content_file_hash = {
        let title_file = tokio::fs::read(&content_file_path).await?;
        match murmur3_32(&mut &title_file[..], 0) {
            Ok(hash) => hash.to_string(),
            Err(e) => return Err(anyhow!("can't hash content file: {}", e).into()),
        }
    };
    let (newest_title_model, mut newest_title_active) = Titles::find()
        .filter(
            Condition::any()
                .add(titles::Column::ContentFileHash.eq(&current_content_file_hash))
                .add(titles::Column::Path.eq(&title_file_path_string)),
        )
        .one(&app_state.db)
        .await
        .map_err(|e| anyhow!("can't find existing title by hash: {}", e))?
        .map(|model| {
            let mut active: titles::ActiveModel = model.clone().into();
            active.id = NotSet;
            if model.path != title_file_path_string {
                active.path = Set(title_file_path_string.clone());
            }
            if model.title != metadata.title {
                active.title = Set(metadata.title.clone());
            }
            if model.category_id != Some(category_id.clone()) {
                active.category_id = Set(Some(category_id.clone()));
            }
            if model.author != metadata.author {
                active.author = Set(metadata.author.clone());
            }
            if model.description != metadata.description {
                active.description = Set(metadata.description.clone());
            }
            if model.release != metadata.release {
                active.release = Set(metadata.release.clone());
            }
            if model.content_file_hash != current_content_file_hash {
                active.content_file_hash = Set(current_content_file_hash.clone());
            }
            if model.cover_and_page_desc_hash != metadata.cover_and_page_desc_hash {
                active.cover_and_page_desc_hash = Set(metadata.cover_and_page_desc_hash.clone());
            }

            let content_hash_changed = model.content_file_hash != current_content_file_hash;
            let content_related_metadata_hash_changed =
                model.cover_and_page_desc_hash != metadata.cover_and_page_desc_hash;
            need_scan_content_file = content_hash_changed || content_related_metadata_hash_changed;

            Ok((model, active))
        })
        .unwrap_or_else(|| {
            let new_active_model = titles::ActiveModel {
                id: Set(TitleID::new()),
                title: Set(metadata.title.clone()),
                category_id: Set(Some(category_id.clone())),
                author: Set(metadata.author.clone()),
                description: Set(metadata.description.clone()),
                release: Set(metadata.release.clone()),
                path: Set(title_file_path_string.clone()),

                content_file_hash: Set(current_content_file_hash),
                cover_and_page_desc_hash: Set(metadata.cover_and_page_desc_hash.clone()),

                cover_path: Set(None),
                cover_blurhash: Set(None),
                blurhash_width: Set(None),
                blurhash_height: Set(None),

                date_added: Set(chrono::Utc::now()),
                date_updated: Set(None),
            };

            need_scan_content_file = true;

            match new_active_model.clone().try_into_model() {
                Ok(model) => Ok((model, new_active_model)),
                Err(e) => Err(AppError::from(anyhow!(
                    "can't convert new active model to model: {}",
                    e
                ))),
            }
        })?;
    // #endregion

    // #region - return if no need to scan; scan & return if archive empty
    if !need_scan_content_file {
        metadata.save()?;
        newest_title_active.save(&app_state.db).await?;
        return Ok((newest_title_model.id, metadata));
    }

    let mut archive = ZipArchive::new(
        File::open(&content_file_path)
            .map_err(|e| AppError::from(anyhow!("can't read content file: {}", e)))?,
    )
    .map_err(|e| AppError::from(anyhow!("can't open content file: {}", e)))?;

    if archive.len() == 0 {
        return Err(AppError::from(anyhow!("content file is empty")));
    }

    // #endregion

    // #region - update Pages table
    let pages_in_file: HashSet<String> = (0..archive.len())
        .filter_map(|i| archive.by_index(i).ok().map(|f| f.name().to_string()))
        .filter(|p| {
            app_state
                .config
                .supported_img_formats
                .contains(&p.split('.').last().unwrap_or_default())
        })
        .collect();

    let pages_in_db: HashSet<String> = Pages::find()
        .filter(pages::Column::TitleId.eq(newest_title_model.id.clone()))
        .all(&app_state.db)
        .await
        .map_err(|e| anyhow!("can't find pages in DB: {}", e))?
        .into_iter()
        .map(|p| p.path)
        .collect();

    let to_be_delete: Vec<&String> = pages_in_db.difference(&pages_in_file).collect();
    let to_be_delete_condition = Condition::any()
        .add(pages::Column::TitleId.eq(newest_title_model.id.clone()))
        .add(pages::Column::Path.is_in(to_be_delete));
    Pages::delete_many()
        .filter(to_be_delete_condition)
        .exec(&app_state.db)
        .await
        .map_err(|e| anyhow!("can't delete pages in DB: {}", e))?;

    let to_be_insert: Vec<&String> = pages_in_file.difference(&pages_in_db).collect();
    let to_be_insert_models = to_be_insert
        .into_iter()
        .map(|path| {
            let page_active = pages::ActiveModel {
                id: Set(PageID::new()),
                title_id: Set(newest_title_model.id.clone()),
                path: Set(path.clone()),
                description: Set(metadata.get_page_description(path)),
            };
            page_active
        })
        .collect::<Vec<_>>();
    Pages::insert_many(to_be_insert_models)
        .on_empty_do_nothing()
        .exec(&app_state.db)
        .await
        .map_err(|e| anyhow!("can't insert pages to DB: {}", e))?;

    let to_be_update: Vec<&String> = pages_in_file.intersection(&pages_in_db).collect();
    for page in to_be_update {
        let page_model = Pages::find()
            .filter(
                Condition::all()
                    .add(pages::Column::TitleId.eq(newest_title_model.id.clone()))
                    .add(pages::Column::Path.eq(page)),
            )
            .one(&app_state.db)
            .await
            .map_err(|e| anyhow!("can't find page in DB: {}", e))?;

        if let Some(page_model) = page_model {
            let page_desc = metadata.get_page_description(page);
            if page_model.description == page_desc {
                continue;
            }
            let mut page_active: pages::ActiveModel = page_model.into();
            page_active.description = Set(page_desc);
            page_active
                .update(&app_state.db)
                .await
                .map_err(|e| anyhow!("can't update page in DB: {}", e))?;
        }
    }
    // #endregion

    // #region - validate cover file
    let mut pages_in_file = pages_in_file;
    let mut valid_cover = false;

    // try using the configured cover file
    if let Some(cover) = metadata.cover.clone() {
        let configured_cover_img: Result<BlurhashResult, String> = pages_in_file
            .get(&cover)
            .ok_or_else(|| format!("cover not found in content file: {}", cover))
            .and_then(|p| {
                archive
                    .by_name(p)
                    .map_err(|e| format!("can't read cover file from content file: {}", e))
            })
            .and_then(|mut f| {
                let mut buf: Vec<u8> = Vec::new();
                f.read_to_end(&mut buf)
                    .map_err(|e| format!("can't read cover file from content file: {}", e))?;
                Ok(buf)
            })
            .and_then(|buf| {
                image::load_from_memory(&buf).map_err(|e| format!("can't decode image: {}", e))
            })
            .and_then(|img| encode(&img));

        match configured_cover_img {
            Ok(blurhash_result) => {
                valid_cover = true;
                newest_title_active.cover_blurhash = Set(Some(blurhash_result.blurhash));
                newest_title_active.blurhash_width = Set(Some(blurhash_result.small_width));
                newest_title_active.blurhash_height = Set(Some(blurhash_result.small_height));
            }
            Err(e) => {
                pages_in_file.remove(&cover);
                warn!(
                    "configured cover file for \"{}\" is invalid: {}",
                    metadata.title, e
                );
            }
        }
    }

    // try to find file contains any of the `config.cover_filestems` strings
    if !valid_cover {
        let mut failed_to_decode: Vec<String> = vec![];

        'scoped: for page_file_name in pages_in_file.iter() {
            let mut page_file_name_has_the_stem = false;
            'scoped2: for stem in app_state.config.cover_filestems.iter() {
                if page_file_name.contains(stem) {
                    page_file_name_has_the_stem = true;
                    break 'scoped2;
                }
            }
            if !page_file_name_has_the_stem {
                continue 'scoped;
            }

            let cover_img: Result<BlurhashResult, String> = archive
                .by_name(&page_file_name)
                .map_err(|e| format!("can't read cover file from content file: {}", e))
                .and_then(|mut f| {
                    let mut buf: Vec<u8> = Vec::new();
                    f.read_to_end(&mut buf)
                        .map_err(|e| format!("can't read cover file from content file: {}", e))?;
                    Ok(buf)
                })
                .and_then(|buf| {
                    image::load_from_memory(&buf).map_err(|e| format!("can't decode image: {}", e))
                })
                .and_then(|img| encode(&img));

            match cover_img {
                Ok(blurhash_result) => {
                    newest_title_active.cover_blurhash = Set(Some(blurhash_result.blurhash));
                    newest_title_active.blurhash_width = Set(Some(blurhash_result.small_width));
                    newest_title_active.blurhash_height = Set(Some(blurhash_result.small_height));
                    valid_cover = true;
                    break 'scoped;
                }
                Err(e) => {
                    failed_to_decode.push(page_file_name.clone());
                    warn!(
                        "configured cover file for \"{}\" is invalid: {}",
                        metadata.title, e
                    );
                }
            }
        }

        failed_to_decode.iter().for_each(|p| {
            pages_in_file.remove(p);
        });
    }

    // last resort, try everything else
    if !valid_cover {
        'scoped: for page_file_name in pages_in_file.iter() {
            let cover_img: Result<BlurhashResult, String> = archive
                .by_name(&page_file_name)
                .map_err(|e| format!("can't read cover file from content file: {}", e))
                .and_then(|mut f| {
                    let mut buf: Vec<u8> = Vec::new();
                    f.read_to_end(&mut buf)
                        .map_err(|e| format!("can't read cover file from content file: {}", e))?;
                    Ok(buf)
                })
                .and_then(|buf| {
                    image::load_from_memory(&buf).map_err(|e| format!("can't decode image: {}", e))
                })
                .and_then(|img| encode(&img));

            match cover_img {
                Ok(blurhash_result) => {
                    newest_title_active.cover_blurhash = Set(Some(blurhash_result.blurhash));
                    newest_title_active.blurhash_width = Set(Some(blurhash_result.small_width));
                    newest_title_active.blurhash_height = Set(Some(blurhash_result.small_height));
                    valid_cover = true;
                    break 'scoped;
                }
                Err(e) => {
                    warn!(
                        "configured cover file for \"{}\" is invalid: {}",
                        metadata.title, e
                    );
                }
            }
        }
    }

    // gave up
    if !valid_cover {
        warn!("can't find a valid cover file for \"{}\"", metadata.title);
    }
    // #endregion

    metadata.save()?;
    newest_title_active.save(&app_state.db).await?;

    Ok((newest_title_model.id, metadata))
}
