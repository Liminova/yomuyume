use std::{
    collections::{HashMap, HashSet},
    io::Cursor,
    sync::Arc,
};

use image::GenericImageView;
use rayon::prelude::*;
use tracing::warn;

use crate::{
    AppState,
    database::{self, content::PageIdentityPath},
    indexer::{
        start_index::UpsertTitleErr,
        utils::{IndexedChapterPages, PageToUpsert},
    },
    utils::{
        absolute_path::AbsolutePath,
        archive_file::{ArchiveFile, ItemInArchive, ItemsInArchiveUtils},
        average_color::AverageColor,
        okay::MapErrorThenOk,
    },
};

pub async fn read_chap_pages_archive(
    app_state: &Arc<AppState>,
    chapter_path: &AbsolutePath,
    files_in_archive: Vec<ItemInArchive>,
    pages_in_db: &Vec<(PageIdentityPath, database::content::PageInfo)>,
) -> Result<IndexedChapterPages, UpsertTitleErr> {
    let pages_in_archive = files_in_archive.keep_images(app_state.config.feature_nomedia);
    if pages_in_archive.is_empty() {
        return Err(UpsertTitleErr::IsEmpty);
    }

    let to_delete = 'scoped: {
        if app_state.first_time_index_content {
            break 'scoped Vec::new();
        }

        let page_in_archive_paths = pages_in_archive
            .iter()
            .map(|p| &p.path)
            .collect::<HashSet<_>>();

        pages_in_db
            .iter()
            .filter(|(iden_path, _)| !page_in_archive_paths.contains(iden_path))
            .map(|(p, _)| p.clone())
            .collect::<Vec<_>>()
    };

    let pages_in_db_map = pages_in_db
        .iter()
        .map(|p| (&p.0, p))
        .collect::<HashMap<_, _>>();

    let to_upsert = {
        let partial = pages_in_archive
            .iter()
            .filter(|p| {
                if let Some(old_modified) = pages_in_db_map
                    .get(&p.path)
                    .and_then(|(_, info)| info.last_modified)
                    && let Some(new_modified) = p.last_modified
                {
                    return new_modified.timestamp() > old_modified.timestamp();
                }
                true
            })
            .collect::<Vec<_>>();

        let mut width_height_color = partial
            .par_iter()
            .filter_map(|p| {
                let img_buffer =
                    chapter_path
                        .as_ref()
                        .read_file_from_archive(&p.path)
                        .okay(|e| {
                            warn!("can't read {} from archive: {e}", p.path);
                        })?;

                let img = image::ImageReader::new(Cursor::new(img_buffer))
                    .with_guessed_format()
                    .okay(|e| warn!("can't guess {} format: {e}", p.path))?
                    .decode()
                    .okay(|e| warn!("can't decode {}: {e}", p.path))?;

                let (width, height) = img.dimensions();
                let color = img.average_color();

                Some((p.path.clone(), (width, height, color)))
            })
            .collect::<HashMap<_, _>>();

        partial
            .into_iter()
            .map(|p| {
                let (width, height, color) = width_height_color
                    .remove(&p.path)
                    .map(|(width, height, color)| (Some(width), Some(height), color))
                    .unwrap_or_default();

                PageToUpsert {
                    identity_path: p.path.clone(),
                    width,
                    height,
                    color,
                    last_modified: p.last_modified,
                    size: p.size,
                }
            })
            .collect::<Vec<_>>()
    };

    Ok(IndexedChapterPages {
        upsert: to_upsert,
        delete: to_delete,
    })
}
