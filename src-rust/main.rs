#![forbid(unsafe_code)]
#![warn(clippy::perf)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::unwrap_used)]
#![allow(clippy::use_self)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::unused_self)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::inefficient_to_string)]

mod library_processor;
mod routes;
mod traits;
mod types;
mod utils;

use std::time::Duration;

use anyhow::Result;
use app_state::AppState;
use axum::{
    middleware::from_fn_with_state as apply,
    routing::{get, post, put},
    Router,
};
use tokio::{net::TcpListener, time::sleep};
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info};
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    routes::{
        auth, delete_bookmark, delete_favorite, get_categories, get_check, get_cover,
        get_delete_account, get_logout, get_page, get_reset_password, get_scanning_progress,
        get_status, get_tags, get_title, get_validate_email, post_delete_account, post_filter,
        post_login, post_modify_info, post_register, post_reset_password, post_status,
        post_validate_email, put_bookmark, put_favorite, put_progress, ApiDoc,
    },
    utils::app_state,
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_ansi(true)
        .with_max_level(tracing::Level::DEBUG)
        .with_env_filter("sqlx=info,axum=info,yomuyume=debug")
        .init();

    let app_state = AppState::new().await;
    let addr = format!(
        "{}:{}",
        app_state.config.listen_address, app_state.config.server_port
    );

    let app = Router::new()
        .nest(
            "/api/auth",
            Router::new()
                .route("/register", post(post_register))
                .route("/login", post(post_login))
                .route("/logout", get(get_logout)),
        )
        .nest(
            "/api/index",
            Router::new()
                .route("/filter", post(post_filter))
                .route("/categories", get(get_categories))
                .route("/title/:title_id", get(get_title))
                .layer(apply(app_state.clone(), auth)),
        )
        .nest(
            "/api/user",
            Router::new()
                .route("/check", get(get_check))
                .route("/delete", get(get_delete_account).post(post_delete_account))
                .route("/verify", get(get_validate_email).post(post_validate_email))
                .route("/modify", post(post_modify_info))
                .route(
                    "/bookmark/:title_id",
                    put(put_bookmark).delete(delete_bookmark),
                )
                .route(
                    "/favorite/:title_id",
                    put(put_favorite).delete(delete_favorite),
                )
                .route("/progress/:title_id/:page", put(put_progress))
                .layer(apply(app_state.clone(), auth)),
        )
        .nest(
            "/api/utils",
            Router::new()
                .route("/tags", get(get_tags))
                .route("/scanning_progress", get(get_scanning_progress))
                .layer(apply(app_state.clone(), auth)),
        )
        .nest(
            "/api/file",
            Router::new()
                .route("/page/:page_id", get(get_page))
                .route("/cover/:title_id", get(get_cover))
                .layer(apply(app_state.clone(), auth)),
        )
        .nest(
            "/api",
            Router::new()
                .route("/user/reset/:email", get(get_reset_password))
                .route("/user/reset", post(post_reset_password))
                .route("/utils/status", get(get_status).post(post_status)),
        )
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state.clone());

    let server_handle = tokio::spawn(async move {
        info!("listening on: {addr}");

        if let Err(e) = axum::serve(
            TcpListener::bind(&addr)
                .await
                .expect("can't start tcp listener"),
            app.into_make_service(),
        )
        .await
        {
            error!("server error: {e:?}");
        };
    });

    let library_processor_handle = tokio::spawn(async move {
        library_processor::full_scan(app_state.clone()).await;
        loop {
            let (enabled, interval) = {
                let cfg = app_state.live_config.read().await;
                let interval = cfg.rescan_interval_in_minutes.max(5);
                (cfg.rescan_enabled, interval)
            };
            sleep(Duration::from_secs(interval as u64 * 60)).await;
            if enabled {
                library_processor::full_scan(app_state.clone()).await;
            }
        }
    });

    let _ = tokio::join!(server_handle, library_processor_handle);

    debug!("server stopped gracefully");
    Ok(())
}
