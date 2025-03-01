pub mod admin;
pub mod auth;
pub mod content;
pub mod errors;
pub mod file;
pub mod middlewares;
pub mod user;
pub mod utils;

use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use errors::InternalErr;
use serde::{Deserialize, Serialize};
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};

use crate::utils::constants::{SESSION_ID_COOKIE_NAME, SESSION_SECRET_COOKIE_NAME};

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                SESSION_ID_COOKIE_NAME,
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new(SESSION_ID_COOKIE_NAME))),
            );
            components.add_security_scheme(
                SESSION_SECRET_COOKIE_NAME,
                SecurityScheme::ApiKey(ApiKey::Cookie(ApiKeyValue::new(
                    SESSION_SECRET_COOKIE_NAME,
                ))),
            );
        }
    }
}

#[derive(OpenApi)]
#[openapi(
    modifiers(&SecurityAddon),
    info(
        description = "yomuyume's api documentations",
        license(name = "MIT or Apache-2.0"),
    ),
    tags(
        (
            name = "admin",
            description = "Admin stuffs"
        ),
        (
            name = "auth",
            description = "Login, register, logout"
        ),
        (
            name = "content",
            description = "Categories, titles, chapters,..."
        ),
        (
            name = "user",
            description = "Change info, password, verify email..."
        ),
        (
            name = "utils",
            description = "Miscellaneous stuffs"
        ),
        (
            name = "file",
            description = "File fetching"
        )
    ),
    paths(
        admin::post_live_config,

        auth::post_login,
        auth::post_register,
        auth::get_logout,
        auth::post_forgot,

        user::delete_bookmark,
        user::delete_favorite,
        user::get_whoami,
        user::post_modify,
        user::post_sensitive,
        user::put_bookmark,
        user::put_favorite,
        user::put_progress,

        content::get_categories,
        content::post_search,
        content::get_chapter,
        content::get_oneshot,
        content::get_series,
        content::get_tags,

        utils::get_status,
        utils::post_status,
        utils::get_scanning_progress,

        file::get_page,
        file::get_cover,
    ),
    components(schemas(
        // Admin
        admin::SetLiveConfigRequest,
        admin::GetLiveConfigResponse,

        // Auth
        auth::LoginRequest,
        auth::RegisterRequest,
        auth::ForgotRequest,

        // User
        user::ModifyRequest,
        user::WhoAmIResponse,
        user::SensitiveRequest,
        user::SensitiveRequestMode,
        user::SensitiveRequestPurpose,

        // Content
        content::InnerCategoriesResponse,
        content::ChapterResponse,
        content::OneshotResponse,
        content::SeriesResponse,
        content::SearchRequest,
        content::InnerTitleSearchResponse,
        content::SearchResponse,
        content::InnerSearchRequestOrderBy,
        content::BaseTitleResponse,
        content::BasePageResponse,
        content::InnerTagResponse,

        // Utils
        utils::StatusRequest,
        utils::StatusResponse,
        utils::ScanningProgressResponse,
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
            .is_ok_and(|()| true)
    })
}

fn hash_pass(input: &[u8]) -> Result<String, InternalErr> {
    Argon2::default()
        .hash_password(input, &SaltString::generate(&mut OsRng))
        .map(|hash| hash.to_string())
        .map_err(|e| {
            tracing::error!("{e}");
            InternalErr::PasswordHash(e)
        })
}

fn is_strong(input: &str) -> bool {
    let upper = input.chars().any(char::is_uppercase);
    let lower = input.chars().any(char::is_lowercase);
    let numeric = input.chars().any(char::is_numeric);
    let special = input.chars().any(|c| c.is_ascii_punctuation());
    let length = input.len() >= 8 && input.len() <= 100;

    if upper && lower && numeric && special && length {
        return true;
    }

    false
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "temp_codes_purpose", rename_all = "snake_case")]
pub enum TempCodePurpose {
    DeleteAccount,
    ResetPassword,
    ValidateEmail,
}
