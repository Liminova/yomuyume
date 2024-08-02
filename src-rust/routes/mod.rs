pub mod auth;
pub mod file;
pub mod index;
pub mod middlewares;
pub mod user;
pub mod utils;

pub use self::{auth::*, file::*, index::*, user::*, utils::*};
pub use middlewares::auth::auth;

use crate::{config::Config, models::categories::Model as Categories};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

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

        file::get_page,
        file::get_cover,
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
        TagResponseBody,
        TagsMapResponseBody,
        TitleResponseBody,
        ScanningProgressResponseBody,


        // Other
        GenericResponseBody,
    ))
)]
pub struct ApiDoc;

/// Check if the password is correct
fn check_pass(real: impl AsRef<str>, input: impl AsRef<str>) -> bool {
    match PasswordHash::new(real.as_ref()) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(input.as_ref().as_bytes(), &parsed_hash)
            .map_or(false, |_| true),
        Err(_) => false,
    }
}

/// Calculate the (width, height) dimension of the blurhash
fn calculate_dimension(config: &Config, ratio: u32) -> (u32, u32) {
    let max_dimension = config.blurhash_dimension_cap;
    let ratio = ratio as f32 / config.ratio_percision as f32;

    let (width, height) = if ratio >= 1.0 {
        (max_dimension, max_dimension / ratio) // Landscape
    } else {
        (max_dimension * ratio, max_dimension) // Portrait
    };

    (width as u32, height as u32)
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct GenericResponseBody {
    pub message: String,
}

impl GenericResponseBody {
    pub fn new<T: ToString>(message: T) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}
