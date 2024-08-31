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
        user::get_delete_account,
        user::get_reset_password,
        user::get_validate_email,
        user::post_delete_account,
        user::post_modify,
        user::post_reset_password,
        user::post_validate_email,
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
        LoginRequestBody,
        LoginResponseBody,
        RegisterRequestBody,

        // User
        DeleteRequestBody,
        ModifyRequestBody,
        ResetRequestBody,

        // Index
        Categories,
        CategoriesResponseBody,
        TitleResponseBody,
        FilterRequestBody,
        FilterResponseBody,
        FilterTitleResponseBody,

        // Utils
        StatusRequestBody,
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

fn hash_pass(input: impl AsRef<str>) -> Result<String, AppError> {
    let input = input.as_ref().as_bytes();

    Argon2::default()
        .hash_password(input, &SaltString::generate(&mut OsRng))
        .map(|hash| hash.to_string())
        .map_err(|e| anyhow::anyhow!("can't hash password: {}", e).into())
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
