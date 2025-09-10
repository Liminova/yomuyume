use std::sync::Arc;

use rayon::prelude::*;
use tracing::error;

use crate::{
    AppState,
    indexer::IndexedContent,
    utils::{
        absolute_path::AbsolutePath, nanoid::nanoid, pathbuf_utils::PathBufUtils,
        result_utils::ResultUtils,
    },
};

pub async fn index_category(
    app_state: Arc<AppState>,
    path: AbsolutePath,
) -> Option<IndexedContent> {
    let category_relative_path = path
        .to_relative(Some(&app_state.config.library_path))
        .okay(|e| {
            error!(
                "failed to get relative path for category {}: {e}",
                path.display()
            );
        })?;

    let mut categories = vec![];
    let mut current_path = category_relative_path.clone();

    while let Some(parent) = current_path.parent() {
        categories.push(parent.to_path_buf());
        current_path = parent.to_path_buf();
    }

    if categories.is_empty() {
        return None;
    }

    if categories.len() == 1 {
        let category_info = categories[0].read_category_info().okay(|e| {
            error!(
                "failed to read category info for {}: {e}",
                categories[0].display()
            );
        });

        let new_id = nanoid();
        let category_path_str = categories[0].to_string_lossy().to_string();
        let name = category_info
            .as_ref()
            .and_then(|info| info.name.clone())
            .unwrap_or_else(|| {
                categories[0]
                    .file_name()
                    .map_or("Untitled".to_string(), |os_str| {
                        os_str.to_string_lossy().to_string()
                    })
            });
        let description = category_info
            .as_ref()
            .and_then(|info| info.description.as_ref());

        return Some(IndexedContent::CategoryID(
            sqlx::query!(
                "INSERT INTO categories (id, path, name, description)
                VALUES (?, ?, ?, ?)
                    ON CONFLICT(path) DO UPDATE SET name=excluded.name, description=excluded.description
                    RETURNING id",
                    new_id,
                    category_path_str,
                    name,
                    description,
                )
                .fetch_one(&app_state.pool)
                .await
                .okay(|e| error!("can't upsert category {category_path_str}: {e}"))?
                .id,
            ),
        );
    }

    let mut new_id_pool = (0..categories.len())
        .into_par_iter()
        .map(|_| nanoid())
        .collect::<Vec<_>>();

    let category_infos = categories
        .par_iter()
        .map(|category_path| {
            (
                category_path,
                category_path
                    .read_category_info()
                    .okay(|e| {
                        error!(
                            "failed to read category info for {}: {e}",
                            category_path.display()
                        );
                    })
                    .map(|info| {
                        (
                            info.name.unwrap_or_else(|| {
                                category_path
                                    .file_name()
                                    .map_or("Untitled".to_string(), |os_str| {
                                        os_str.to_string_lossy().to_string()
                                    })
                            }),
                            info.description,
                        )
                    }),
            )
        })
        .collect::<Vec<_>>(); // unless there are thousands of nested categories, use Vec, not HashMap

    let mut tx = app_state
        .pool
        .begin()
        .await
        .okay(|e| error!("can't start transaction: {e}"))?;

    let mut indexed_category_ids = Vec::with_capacity(categories.len());

    for index in (1..categories.len()).rev() {
        let category = &categories[index];
        let category_str = category.to_string_lossy().to_string();
        if category_str.is_empty() {
            continue;
        }
        let parent_id = if index == 1 {
            None
        } else {
            let new_id = new_id_pool.pop().unwrap_or_else(nanoid);
            let parent_path = &categories[index - 1];
            let parent_path_str = parent_path.to_string_lossy().to_string();
            let Some((name, description)) = category_infos
                .iter()
                .find(|(p, _)| *p == parent_path)
                .and_then(|(_, info)| info.as_ref())
            else {
                unreachable!("parent category info must exist");
            };

            sqlx::query!(
                "INSERT INTO categories (id, path, name, description)
                VALUES (?, ?, ?, ?)
                ON CONFLICT(path) DO UPDATE SET name=excluded.name, description=excluded.description
                RETURNING id",
                new_id,
                parent_path_str,
                name,
                description,
            )
            .fetch_one(&mut *tx)
            .await
            .okay(|e| error!("can't upsert category {parent_path_str}: {e}"))?
            .id
            .into()
        };

        if let Some(ref parent_id) = parent_id
            && !indexed_category_ids.contains(parent_id)
        {
            indexed_category_ids.push(parent_id.clone());
        }

        let new_id = new_id_pool.pop().unwrap_or_else(nanoid);
        let Some((name, description)) = category_infos
            .iter()
            .find(|(p, _)| *p == &categories[index])
            .and_then(|(_, info)| info.as_ref())
        else {
            unreachable!("category info must exist");
        };

        let category_id = sqlx::query!(
            "INSERT INTO categories (id, path, name, description, parent_id)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(path) DO UPDATE SET name=excluded.name, description=excluded.description, parent_id=excluded.parent_id
            RETURNING id",
            new_id,
            category_str,
            name,
            description,
            parent_id,
        )
        .fetch_one(&mut *tx)
        .await
        .okay(|e| error!("can't upsert category {category_str}: {e}"))?.id;

        indexed_category_ids.push(category_id);
    }

    tx.commit()
        .await
        .okay(|e| error!("can't commit transaction: {e}"))?;

    Some(IndexedContent::CategoryIDs(indexed_category_ids))
}
