use std::{collections::HashMap, fs::DirEntry, sync::Arc};

use chrono::{DateTime, Utc};
use image::{GenericImageView, ImageReader};
use rayon::prelude::*;
use tracing::{error, warn};

use crate::{
    AppState,
    indexer::utils::{IndexedChapterPages, PageInDB, PageToUpsert},
    utils::{
        absolute_path::AbsolutePath, average_color::AverageColor, nanoid::nanoid,
        pathbuf_utils::PathBufUtils, result_utils::ResultUtils,
    },
};

#[derive(Debug)]
struct PageInDir {
    pub path: AbsolutePath,
    pub rel_path: String,
    pub last_modified: Option<DateTime<Utc>>,
    pub size: Option<i64>,
}

pub async fn read_chap_pages_dir(
    app_state: &Arc<AppState>,
    chapter_path: &AbsolutePath,
    files_in_chapter: Option<Vec<DirEntry>>,
    pages_in_db: &[PageInDB],
) -> Option<IndexedChapterPages> {
    let pages_in_dir = files_in_chapter
        .unwrap_or_else(|| {
            std::fs::read_dir(chapter_path.as_ref())
                .map(|rd| {
                    rd.filter_map(|f| {
                        f.okay(|e| {
                            error!("can't read dir entry in {}: {e}", chapter_path.display());
                        })
                    })
                    .collect()
                })
                .okay(|e| {
                    error!(
                        "can't read chapter directory {}: {e}",
                        chapter_path.display()
                    );
                })
                .unwrap_or_default()
        })
        .into_iter()
        .fold(Vec::new(), |mut acc, e| {
            let path = e.path();
            let Some(abs_path) = AbsolutePath::from(&e.path(), None).okay(|err| {
                warn!("can't convert {} to absolute path: {err}", path.display());
            }) else {
                return acc;
            };
            let Some(rel_path) = abs_path
                .to_relative(Some(chapter_path))
                .map(|p| p.to_string_lossy().to_string())
                .okay(|e| {
                    warn!("can't convert {} to relative path: {e}", path.display());
                })
            else {
                return acc;
            };

            if abs_path.is_file() {
                acc.push(PageInDir {
                    path: abs_path,
                    rel_path,
                    last_modified: path.last_modified().okay(|err| {
                        warn!("can't get last modified for {}: {err}", path.display());
                    }),
                    size: e
                        .metadata()
                        .okay(|err| {
                            warn!("can't get metadata for {}: {err}", path.display());
                        })
                        .map(|m| m.len() as i64),
                });
                return acc;
            }
            acc.append(
                path.scan_dir_recursively_for_image(app_state.config.feature_nomedia)
                    .into_iter()
                    .fold(&mut Vec::new(), |acc, img| {
                        let Some(abs_path) = AbsolutePath::from(&img, None).okay(|err| {
                            warn!("can't convert {} to absolute path: {err}", img.display());
                        }) else {
                            return acc;
                        };
                        let Some(rel_path) = abs_path
                            .to_relative(Some(chapter_path))
                            .map(|p| p.to_string_lossy().to_string())
                            .okay(|e| {
                                warn!(
                                    "can't convert {} to relative path: {e}",
                                    abs_path.as_ref().display()
                                );
                            })
                        else {
                            return acc;
                        };

                        acc.push(PageInDir {
                            path: abs_path,
                            rel_path,
                            last_modified: img.last_modified().okay(|err| {
                                warn!("can't get last modified for {}: {err}", img.display());
                            }),
                            size: img.metadata().map(|m| m.len() as i64).okay(|err| {
                                warn!("can't get metadata for {}: {err}", img.display());
                            }),
                        });
                        acc
                    }),
            );
            acc
        });
    if pages_in_dir.is_empty() {
        return None;
    }

    let to_delete = {
        let pages_in_dir_rel_paths = pages_in_dir
            .iter()
            .filter_map(|p| {
                p.path.to_relative(Some(chapter_path)).okay(|e| {
                    warn!(
                        "can't convert {} to relative path: {e}",
                        p.path.as_ref().display()
                    );
                })
            })
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>();

        pages_in_db
            .iter()
            .filter(|p| !pages_in_dir_rel_paths.contains(&p.path))
            .map(|p| p.id.clone())
            .collect::<Vec<_>>()
    };

    let pages_in_db_map = pages_in_db
        .iter()
        .map(|p| (&p.path, p))
        .collect::<HashMap<_, _>>();

    let to_upsert = {
        let partial = pages_in_dir
            .iter()
            .filter(|p| {
                if let Some(old_modified) = pages_in_db_map
                    .get(&p.rel_path)
                    .and_then(|p| p.last_modified)
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
                let img = ImageReader::open(p.path.as_ref())
                    .okay(|e| {
                        warn!("can't open {}: {e}", p.path);
                    })?
                    .decode()
                    .okay(|e| {
                        warn!("can't decode {}: {e}", p.path);
                    })?;

                let (width, height) = img.dimensions();
                let color = img.average_color();

                Some((p.path.clone(), (width, height, color)))
            })
            .collect::<HashMap<_, _>>();

        partial
            .into_iter()
            .map(|p| {
                let (width, height, avg_hex_color) = width_height_color
                    .remove(&p.path)
                    .map(|(w, h, c)| (Some(w), Some(h), c.map(|c| c.to_string())))
                    .unwrap_or_default();

                PageToUpsert {
                    id: pages_in_db_map
                        .get(&p.rel_path)
                        .map(|p| p.id.clone())
                        .unwrap_or_else(nanoid),
                    path: p.rel_path.clone(),
                    width,
                    height,
                    avg_hex_color,
                    last_modified: p.last_modified.map(|d| d.naive_utc()),
                    size: p.size,
                }
            })
            .collect::<Vec<_>>()
    };

    Some(IndexedChapterPages {
        upsert: to_upsert,
        delete: to_delete,
    })
}
