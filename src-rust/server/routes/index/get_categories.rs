use std::sync::Arc;

pub use bridge::routes::index::{CategoriesResponseBody, CategoryResponse};

use crate::{
    models::prelude::*,
    routes::{MyResponse, MyResponseBuilder},
    AppState,
};

use axum::{extract::State, http::HeaderMap, response::IntoResponse};
use sea_orm::*;

/// Get all categories to be displayed on the library page.
#[utoipa::path(get, path = "/api/index/categories", responses(
    (status = 200, description = "Fetch all categories successful", body = CategoriesResponseBody),
    (status = 500, description = "Internal server error", body = GenericResponseBody)
))]
pub async fn get_categories(
    State(data): State<Arc<AppState>>,
    header: HeaderMap,
) -> Result<impl IntoResponse, MyResponse> {
    let builder = MyResponseBuilder::new(header);

    let data = Categories::find()
        .all(&data.db)
        .await
        .map_err(|e| builder.db_error(e))?
        .into_iter()
        .map(|category| CategoryResponse {
            id: category.id.to_string(),
            name: category.name,
            description: category.description,
        })
        .collect();

    Ok(builder.success(CategoriesResponseBody { data }))
}
