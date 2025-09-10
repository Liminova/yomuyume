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
    indexer::utils::{IndexedChapterPages, PageInDB, PageToUpsert},
    utils::{
        absolute_path::AbsolutePath,
        archive_file::{ArchiveFile, ItemInArchive, ItemsInArchiveUtils},
        average_color::AverageColor,
        nanoid::nanoid,
        result_utils::ResultUtils,
    },
};

pub async fn read_chap_pages_archive(
    app_state: &Arc<AppState>,
    chapter_path: &AbsolutePath,
    files_in_archive: Vec<ItemInArchive>,
    pages_in_db: &[PageInDB],
) -> Option<IndexedChapterPages> {
    let pages_in_archive = files_in_archive.keep_images(app_state.config.feature_nomedia);
    if pages_in_archive.is_empty() {
        return None;
    }

    let to_delete = {
        let page_in_archive_paths = pages_in_archive
            .iter()
            .map(|p| &p.path)
            .collect::<HashSet<_>>();

        pages_in_db
            .iter()
            .filter(|p| !page_in_archive_paths.contains(&p.path))
            .map(|p| p.id.clone())
            .collect::<Vec<_>>()
    };

    let pages_in_db_map = pages_in_db
        .iter()
        .map(|p| (&p.path, p))
        .collect::<HashMap<_, _>>();

    let to_upsert = {
        let partial = pages_in_archive
            .iter()
            .filter(|p| {
                if let Some(old_modified) =
                    pages_in_db_map.get(&p.path).and_then(|p| p.last_modified)
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
                chapter_path
                    .as_ref()
                    .read_file_from_archive(&p.path)
                    .okay(|e| {
                        warn!("can't read {} from archive: {e}", p.path);
                    })
                    .map(Cursor::new)
                    .map(image::ImageReader::new)
                    .and_then(|img| {
                        img.with_guessed_format()
                            .okay(|e| warn!("can't guess {} format: {e}", p.path))
                    })
                    .and_then(|img| {
                        img.decode().okay(|e| {
                            warn!("can't decode {}: {e}", p.path);
                        })
                    })
                    .map(|img| {
                        let (width, height) = img.dimensions();
                        let color = img.average_color();
                        (&p.path, (width, height, color))
                    })
            })
            .collect::<HashMap<_, _>>();

        let mut tmp = partial
            .into_iter()
            .map(|p| {
                let (width, height, avg_hex_color) = width_height_color
                    .remove(&p.path)
                    .map(|(w, h, c)| (Some(w), Some(h), c.map(|c| c.to_string())))
                    .unwrap_or_default();

                PageToUpsert {
                    id: pages_in_db_map
                        .get(&p.path)
                        .map_or_else(nanoid, |p| p.id.clone()),
                    path: p.path.clone(),
                    width,
                    height,
                    avg_hex_color,
                    last_modified: p.last_modified.map(|d| d.naive_utc()),
                    size: p.size,
                }
            })
            .collect::<Vec<_>>();

        tmp.sort_by(|a, b| a.path.cmp(&b.path));
        tmp
    };

    Some(IndexedChapterPages {
        upsert: to_upsert,
        delete: to_delete,
    })
}
