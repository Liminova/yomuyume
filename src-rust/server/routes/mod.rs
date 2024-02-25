pub mod auth;
pub mod file;
pub mod index;
pub mod middlewares;
pub mod user;
pub mod utils;

pub use self::{auth::*, file::*, index::*, user::*, utils::*};
pub use middlewares::auth::auth;
use sea_orm::DbErr;

use crate::{
    constants::{blurhash_dimension_cap, ratio_percision},
    models::categories::Model as Categories,
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

/* #region - to replace the clunky (StatusCode, Json<ErrorResponseBody>) with ErrorResponse */
#[derive(Clone, Deserialize, Serialize, ToSchema, Debug)]
pub struct ErrorResponseBody {
    pub message: String,
}

pub struct ErrRsp {
    status: StatusCode,
    body: Json<ErrorResponseBody>,
}
impl ErrRsp {
    pub fn new<S: AsRef<str>>(status: StatusCode, body: S) -> Self {
        Self {
            status,
            body: Json(ErrorResponseBody {
                message: body.as_ref().to_string(),
            }),
        }
    }

    /// Internal Server Error, but add "Database error: " to the message
    pub fn db(body: DbErr) -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", body),
        )
    }

    /// Internal Server Error
    pub fn internal<S: AsRef<str>>(body: S) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, body)
    }

    pub fn bad_request<S: AsRef<str>>(body: S) -> Self {
        Self::new(StatusCode::BAD_REQUEST, body)
    }

    pub fn not_found<S: AsRef<str>>(body: S) -> Self {
        Self::new(StatusCode::NOT_FOUND, body)
    }

    pub fn no_token() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "You're not logged in, please provide a token.",
        )
    }
}
impl IntoResponse for ErrRsp {
    fn into_response(self) -> Response {
        (self.status, self.body).into_response()
    }
}
/* #endregion */

/* #region - GenericResponse::new() looks more elegant than build_resp() */
#[derive(Clone, Deserialize, Serialize, ToSchema, Debug)]
struct GenericResponseBody {
    pub message: String,
}

struct GenericRsp;
impl GenericRsp {
    pub fn create<S: AsRef<str>>(body: S) -> (StatusCode, Json<GenericResponseBody>) {
        (
            StatusCode::OK,
            Json(GenericResponseBody {
                message: body.as_ref().to_string(),
            }),
        )
    }
}
/* #endregion */

#[derive(OpenApi)]
#[openapi(
    info(
        description = "yomuyume's backend documentations.",
        license(name = "MIT or Apache-2.0"),
    ),
    tags(
        (
            name = "auth",
            description = "login, register, logout."
        ),
        (
            name = "index",
            description = "all the routes related to fetching index data."
        ),
        (
            name = "user",
            description = "all the routes related to user."
        ),
        (
            name = "utils",
            description = "getting server status, item/category id-name map"
        ),
        (
            name = "file",
            description = "all the routes related to file fetching."
        )
    ),
    paths(
        auth::post_login,
        auth::post_register,
        auth::get_logout,

        user::delete_bookmark,
        user::delete_favorite,
        user::get_check,
        user::get_delete,
        user::get_reset,
        user::get_verify,
        user::post_delete,
        user::post_modify,
        user::post_reset,
        user::post_verify,
        user::put_bookmark,
        user::put_favorite,
        user::put_progress,

        index::get_categories,
        index::post_filter,
        index::get_title,

        utils::get_status,
        utils::post_status,
        utils::get_tags,
        utils::get_scanning_progress,
        utils::get_ssim_eval,

        file::get_page,
        file::get_thumbnail,
    ),
    components(schemas(
        // Auth
        LoginRequest,
        LoginResponseBody,
        RegisterRequest,

        // User
        DeleteRequest,
        ModifyRequest,
        ResetRequest,

        // Index
        Categories,
        CategoriesResponseBody,
        TitleResponseBody,
        FilterRequest,
        FilterResponseBody,
        FilterTitleResponseBody,

        // Utils
        StatusRequest,
        StatusResponseBody,
        TagsMapResponseBody,
        TitleResponseBody,
        ScanningProgressResponseBody,

        // Other
        GenericResponseBody,
        ErrorResponseBody,
    ))
)]
pub struct ApiDoc;

fn check_pass(real: &str, input: &String) -> bool {
    match PasswordHash::new(real) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(input.as_bytes(), &parsed_hash)
            .map_or(false, |_| true),
        Err(_) => false,
    }
}

fn calculate_dimension(ratio: u32) -> (u32, u32) {
    let max_dimension = blurhash_dimension_cap();
    let ratio = ratio as f32 / ratio_percision() as f32;

    let (width, height) = if ratio >= 1.0 {
        (max_dimension, max_dimension / ratio) // Landscape
    } else {
        (max_dimension * ratio, max_dimension) // Portrait
    };

    (width as u32, height as u32)
}
