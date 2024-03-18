pub mod auth;
pub mod file;
pub mod index;
pub mod middlewares;
pub mod user;
pub mod utils;

use std::fmt::Display;

pub use self::{auth::*, file::*, index::*, user::*, utils::*};
use axum::{
    body::Body,
    http::{header, HeaderMap, HeaderName, Response, StatusCode},
    response::IntoResponse,
};
use bridge::GenericResponseBody;
pub use middlewares::auth::auth;

use crate::{
    constants::{blurhash_dimension_cap, ratio_percision},
    models::categories::Model as Categories,
};
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use bitcode::Encode;
use serde::Serialize;
use utoipa::OpenApi;

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

pub struct MyResponse {
    pub status_code: StatusCode,
    pub response_header: Vec<(HeaderName, String)>,
    pub payload: Vec<u8>,
}

impl MyResponse {
    pub fn add_header(self, header: (HeaderName, String)) -> Self {
        let mut response_header = self.response_header;
        response_header.push(header);
        MyResponse {
            status_code: self.status_code,
            response_header,
            payload: self.payload,
        }
    }
}

impl IntoResponse for MyResponse {
    fn into_response(self) -> Response<Body> {
        let mut headers = HeaderMap::new();
        for (name, value) in self.response_header {
            headers.insert(name, value.parse().unwrap());
        }

        (self.status_code, headers, self.payload).into_response()
    }
}

/// Init inside a route handler with the request header to determine whether
/// to encode the response body as bitcode or json.
#[derive(Debug)]
pub struct MyResponseBuilder {
    request_header: HeaderMap,
}

impl MyResponseBuilder {
    pub fn new(request_header: HeaderMap) -> Self {
        MyResponseBuilder { request_header }
    }

    pub fn build<T: Encode + Serialize>(
        &self,
        status_code: StatusCode,
        response_body: T,
    ) -> MyResponse {
        let accept_bitcode = &self
            .request_header
            .get(header::ACCEPT)
            .map(|v| v.to_str().unwrap_or_default())
            .unwrap_or_default()
            .contains("bitcode");

        let (header, payload) = match accept_bitcode {
            true => (
                "bitcode".to_string(),
                bitcode::encode(&response_body)
                    .map_err(|e| tracing::error!("Failed to encode bitcode: {}", e))
                    .unwrap_or_default(),
            ),

            false => (
                "application/json".to_string(),
                serde_json::to_vec(&response_body)
                    .map_err(|e| tracing::error!("Failed to encode json: {}", e))
                    .unwrap_or_default(),
            ),
        };

        MyResponse {
            status_code,
            response_header: vec![(header::CONTENT_TYPE, header)],
            payload,
        }
    }

    /// For building all success responses
    pub fn success<T: Encode + Serialize>(&self, body: T) -> MyResponse {
        self.build(StatusCode::OK, body)
    }

    /// For building all error responses
    pub fn generic<T: AsRef<str>>(&self, status_code: StatusCode, message: T) -> MyResponse {
        self.build(
            status_code,
            GenericResponseBody {
                message: message.as_ref().to_string(),
            },
        )
    }

    /* Childrens of self.build, shorthand for commonly used error resps */

    pub fn generic_success<T: AsRef<str>>(&self, message: T) -> MyResponse {
        self.build(
            StatusCode::OK,
            GenericResponseBody {
                message: message.as_ref().to_string(),
            },
        )
    }

    pub fn no_content<T: AsRef<str>>(&self, message: T) -> MyResponse {
        self.generic(StatusCode::NO_CONTENT, message)
    }

    pub fn db_error<T: Display>(&self, message: T) -> MyResponse {
        self.generic(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", message),
        )
    }

    pub fn internal<T: AsRef<str>>(&self, message: T) -> MyResponse {
        self.generic(StatusCode::INTERNAL_SERVER_ERROR, message)
    }

    pub fn bad_request<T: AsRef<str>>(&self, message: T) -> MyResponse {
        self.generic(StatusCode::BAD_REQUEST, message)
    }

    pub fn not_found<T: AsRef<str>>(&self, message: T) -> MyResponse {
        self.generic(StatusCode::NOT_FOUND, message)
    }

    pub fn conflict<T: AsRef<str>>(&self, message: T) -> MyResponse {
        self.generic(StatusCode::CONFLICT, message)
    }

    pub fn unauthorized<T: AsRef<str>>(&self, message: T) -> MyResponse {
        self.generic(StatusCode::UNAUTHORIZED, message)
    }
}

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
