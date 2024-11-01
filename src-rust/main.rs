mod library_scanner;
mod routes;
mod types;
mod utils;

use std::sync::Arc;

use anyhow::{Context, Result};
use axum::{
    middleware::from_fn_with_state as apply,
    routing::{get, post, put},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use tokio::{net::TcpListener, sync::Mutex};
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info};
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    routes::{auth, ApiDoc},
    utils::*,
};
use routes::*;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let config = Config::init();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_env_filter("sqlx=warn,axum=info,yomuyume=debug")
        .init();

    let app_state = Arc::new(AppState {
        pool: PgPoolOptions::new()
            .max_connections(100)
            .connect(&config.database_url)
            .await
            .context("can't connect to database")?,
        config: config.clone(),
        scanning_complete: Mutex::new(false),
        scanning_progress: Mutex::new(0.0),
    });

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
                .route("/reset", post(post_reset_password))
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
                .route("/utils/status", get(get_status).post(post_status)),
        )
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http())
        .with_state(app_state.clone());

    let server_handle = tokio::spawn(async move {
        let addr = format!("{}:{}", config.listen_address, config.server_port);
        debug!("listening on: {addr}");

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

    let library_scanner_handle = tokio::spawn(async move {
        if let Err(e) = library_scanner::Scanner::new(app_state.clone())
            .await
            .run()
            .await
        {
            error!("scanner error: {e:?}");
        };
    });

    let _ = server_handle.await;
    let _ = library_scanner_handle.await;

    Ok(())
}
