#![allow(clippy::needless_for_each)]

pub mod auth;
pub mod errors;
pub mod file;
pub mod middlewares;
pub mod opds;
pub mod user;
pub mod utils;

use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use utoipa::{
    Modify, OpenApi,
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
};

use crate::utils::constants::SESSION_SECRET_COOKIE_NAME;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
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
        auth::post_login,
        auth::post_register,
        auth::get_logout,
        auth::post_forgot,

        user::get_whoami,
        user::post_modify,
        user::put_progress,

        utils::get_status,
        utils::post_status,

        file::get_page_file,
        file::get_cover_file,
    ),
    components(schemas(
        auth::LoginRequest,
        auth::RegisterRequest,
        auth::ForgotRequest,

        user::ModifyRequest,
        user::WhoAmIResponse,

        utils::StatusRequest,
        utils::StatusResponse,
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

fn hash_pass(input: impl AsRef<str>) -> Result<String, argon2::password_hash::Error> {
    Argon2::default()
        .hash_password(input.as_ref().as_bytes(), &SaltString::generate(&mut OsRng))
        .map(|hash| hash.to_string())
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

#[derive(Debug, Clone)]
pub struct UserIDExtension(pub Option<String>);
