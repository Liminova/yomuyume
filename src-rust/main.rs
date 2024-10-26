mod library_scanner;
mod migrator;
mod models;
mod routes;
mod utils;

use std::sync::Arc;

use anyhow::Result;
use axum::{
    middleware::from_fn_with_state as apply,
    routing::{get, post, put},
    Router,
};
use sea_orm::{ConnectionTrait, Database, DbBackend, DbErr};
use sea_orm_migration::prelude::*;
use tokio::{net::TcpListener, sync::Mutex};
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info};
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    migrator::Migrator,
    routes::{auth, ApiDoc},
    utils::*,
};
use routes::*;

#[tokio::main]
async fn main() -> Result<(), DbErr> {
    dotenvy::dotenv().ok();
    let config = Config::init();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_env_filter("sqlx=warn,axum=info,yomuyume=debug")
        .init();

    let db = Database::connect(&config.database_url).await?;
    if db.get_database_backend() != DbBackend::Sqlite {
        error!("we don't support other databases outside of sqlite. exiting.");
        std::process::exit(1)
    }

    if let Err(e) = db.execute_unprepared("PRAGMA journal_mode = WAL;").await {
        info!("{}", e);
    }

    let schema_manager = SchemaManager::new(&db);
    Migrator::up(&db, None).await?;
    assert!(schema_manager.has_table("users").await?);
    assert!(schema_manager.has_table("categories").await?);
    assert!(schema_manager.has_table("titles").await?);
    assert!(schema_manager.has_table("pages").await?);
    assert!(schema_manager.has_table("tags").await?);
    assert!(schema_manager.has_table("titles_tags").await?);
    assert!(schema_manager.has_table("bookmarks").await?);
    assert!(schema_manager.has_table("favorites").await?);
    assert!(schema_manager.has_table("progresses").await?);
    assert!(schema_manager.has_table("session_tokens").await?);
    assert!(schema_manager.has_table("temp_codes").await?);

    info!("database migrations complete!");

    let app_state = Arc::new(AppState {
        db,
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
                .route(
                    "/logout",
                    get(get_logout).route_layer(apply(app_state.clone(), auth)),
                ),
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
                .route("/bookmark/:id", put(put_bookmark).delete(delete_bookmark))
                .route("/favorite/:id", put(put_favorite).delete(delete_favorite))
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
