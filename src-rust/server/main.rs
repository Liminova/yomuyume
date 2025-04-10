#![forbid(unsafe_code)]
#![warn(clippy::perf, clippy::pedantic, clippy::nursery, clippy::unwrap_used)]
#![allow(
    clippy::use_self,
    clippy::missing_const_for_fn,
    clippy::redundant_closure_for_method_calls,
    clippy::doc_markdown,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::unused_self,
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::trivially_copy_pass_by_ref,
    clippy::inefficient_to_string,
    clippy::unreadable_literal
)]

mod library_processor;
mod routes;
mod structs;
mod traits;
mod utils;

use std::{net::SocketAddr, time::Duration};

use anyhow::Result;
use app_state::AppState;
use axum::{
    body::Body,
    extract::Request,
    http::{header, StatusCode},
    middleware::from_fn_with_state as apply,
    response::IntoResponse,
    routing::{get, post, put},
    Router,
};
use routes::{
    admin::{get_live_config, post_live_config},
    auth::post_forgot,
    user::post_sensitive,
};
use tokio::{net::TcpListener, time::sleep};
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info};
use utils::constants::{
    BOOKMARK_PATH, FAVORITE_PATH, FORGOT_PATH, GET_CATEGORIES_PATH, GET_COVER_FILE_PATH,
    GET_PAGES_PATH, GET_PAGE_FILE_PATH, GET_SCANNING_PROGRESS_PATH, GET_STATUS_PATH, GET_TAGS_PATH,
    GET_TITLE_PATH, LIVE_CONFIG_PATH, LOGIN_PATH, LOGOUT_PATH, REGISTER_PATH, SEARCH_PATH,
    USER_MODIFY_PATH, USER_PROGRESS_PATH, USER_SENSITIVE_PATH, WHOAMI_PATH,
};
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};

use crate::{
    routes::{
        auth::{get_logout, post_login, post_register},
        content::{get_categories, get_pages, get_search, get_tags, get_title},
        file::{get_cover_file, get_page_file},
        middlewares::auth::auth,
        user::{
            delete_bookmark, delete_favorite, get_whoami, post_modify, put_bookmark, put_favorite,
            put_progress,
        },
        utils::{get_scanning_progress, get_status, post_status},
        ApiDoc,
    },
    utils::app_state,
};

include!(concat!(env!("OUT_DIR"), "/spa.rs"));

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_ansi(true)
        .with_max_level(tracing::Level::DEBUG)
        .with_env_filter("sqlx=info,axum=info,yomuyume=debug")
        .with_file(true)
        .with_line_number(true)
        .init();

    let app_state = AppState::new().await;
    let addr = format!(
        "{}:{}",
        app_state.config.listen_address, app_state.config.server_port
    );

    let app = Router::new()
        .route(REGISTER_PATH, post(post_register))
        .route(LOGIN_PATH, post(post_login))
        .route(LOGOUT_PATH, get(get_logout))
        .route(FORGOT_PATH, post(post_forgot))
        .merge(
            Router::new()
                // admin
                .route(
                    LIVE_CONFIG_PATH,
                    post(post_live_config).get(get_live_config),
                )
                // content
                .route(SEARCH_PATH, post(get_search))
                .route(GET_CATEGORIES_PATH, get(get_categories))
                .route(GET_TITLE_PATH, get(get_title))
                .route(GET_PAGES_PATH, get(get_pages))
                .route(GET_TAGS_PATH, get(get_tags))
                // user
                .route(WHOAMI_PATH, get(get_whoami))
                .route(USER_SENSITIVE_PATH, post(post_sensitive))
                .route(USER_MODIFY_PATH, post(post_modify))
                .route(BOOKMARK_PATH, put(put_bookmark).delete(delete_bookmark))
                .route(FAVORITE_PATH, put(put_favorite).delete(delete_favorite))
                .route(USER_PROGRESS_PATH, put(put_progress))
                // file
                .route(GET_PAGE_FILE_PATH, get(get_page_file))
                .route(GET_COVER_FILE_PATH, get(get_cover_file))
                // misc
                .route(GET_SCANNING_PROGRESS_PATH, get(get_scanning_progress))
                // middleware
                .layer(apply(app_state.clone(), auth)),
        )
        .route(GET_STATUS_PATH, get(get_status).post(post_status))
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        .fallback(|req: Request<Body>| async move {
            let request_path = req
                .uri()
                .path()
                .trim_start_matches('/')
                .trim_end_matches('/');

            // index.html
            if request_path.is_empty() {
                return (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, "text/html")],
                    get_spa_asset(b"index.html"),
                )
                    .into_response();
            }

            let body = get_spa_asset(
                {
                    if SPA_IMPLICIT_INDEX_HTML.contains(&request_path) {
                        format!("{request_path}/index.html")
                    } else {
                        request_path.to_string()
                    }
                }
                .as_bytes(),
            );

            if body.is_empty() {
                return (
                    StatusCode::NOT_FOUND,
                    [(header::CONTENT_TYPE, "text/html")],
                    Some(get_spa_asset(b"404.html"))
                        .filter(|b| !b.is_empty())
                        .map_or_else(Body::empty, Body::from),
                )
                    .into_response();
            }

            let header = [(
                header::CONTENT_TYPE,
                request_path
                    .split('.')
                    .next_back()
                    .map_or("text/plain", |ext| get_mime_type(ext.as_bytes()))
                    .to_string(),
            )];

            (StatusCode::OK, header, body).into_response()
        })
        .layer(TraceLayer::new_for_http())
        .with_state(app_state.clone());

    let server_handle = tokio::spawn(async move {
        info!("listening on: {addr}");

        if let Err(e) = axum::serve(
            TcpListener::bind(&addr)
                .await
                .expect("can't start tcp listener"),
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        {
            error!("server error: {e:?}");
        };
    });

    let library_processor_handle = tokio::spawn(async move {
        library_processor::full_scan(app_state.clone()).await;
        loop {
            sleep(Duration::from_secs(
                u64::from(
                    app_state
                        .live_config
                        .get_rescan_interval_in_minutes()
                        .await
                        .max(5),
                ) * 60,
            ))
            .await;

            if app_state.live_config.get_rescan_enabled().await {
                library_processor::full_scan(app_state.clone()).await;
            }
        }
    });

    let _ = tokio::join!(server_handle, library_processor_handle);

    debug!("server stopped gracefully");
    Ok(())
}
