pub mod auth;
pub mod file;
pub mod index;
pub mod middlewares;
pub mod user;
pub mod utils;

pub use self::{auth::*, file::*, index::*, user::*, utils::*};
pub use middlewares::auth::auth;

use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand_core::OsRng;
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "session-id",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("session-id"))),
            );
            components.add_security_scheme(
                "session-secret",
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new("session-secret"))),
            );
        }
    }
}

use crate::utils::app_error::AppError;

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    info(
        description = "yomuyume's api documentations",
        license(name = "MIT or Apache-2.0"),
    ),
    tags(
        (
            name = "auth",
            description = "login, register, logout"
        ),
        (
            name = "index",
            description = "all the routes related to fetching index data"
        ),
        (
            name = "user",
            description = "all the routes related to user"
        ),
        (
            name = "utils",
            description = "getting server status, item/category id-name map"
        ),
        (
            name = "file",
            description = "all the routes related to file fetching"
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
        user::post_modify_info,
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
        ValidateEmailRequestBody,

        // Index
        CategoryResponseBody,
        TitleResponseBody,
        TitlePageResponse,
        TitleTagResponse,
        FilterRequestBody,
        FilterResponseBody,
        FilterTitleResponseBody,

        // Utils
        StatusRequestBody,
        StatusResponseBody,
        TagResponseBody,
        ScanningProgressResponseBody,
    ))
)]
pub struct ApiDoc;

/// Check if a [`password_input`] after hashing matches a [`password_hash`].
///
/// [`password_hash`]: String
/// [`password_input`]: String
fn check_pass(password_hash: impl AsRef<str>, password_input: impl AsRef<str>) -> bool {
    PasswordHash::new(password_hash.as_ref()).is_ok_and(|parsed_hash| {
        Argon2::default()
            .verify_password(password_input.as_ref().as_bytes(), &parsed_hash)
            .is_ok_and(|_| true)
    })
}

fn hash_pass(input: impl AsRef<str>) -> Result<String, AppError> {
    let input = input.as_ref().as_bytes();

    Argon2::default()
        .hash_password(input, &SaltString::generate(&mut OsRng))
        .map(|hash| hash.to_string())
        .map_err(|e| anyhow::anyhow!("can't hash password: {}", e).into())
}
