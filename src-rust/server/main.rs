use crate::{
    config::Config,
    migrator::Migrator,
    routes::{auth, ApiDoc},
};
use axum::{
    middleware::from_fn_with_state as apply,
    routing::{get, post, put},
    Router,
};
use routes::*;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr};
use sea_orm_migration::prelude::*;
use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpListener, sync::Mutex};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

mod config;
mod constants;
mod livescan;
mod migrator;
mod models;
mod routes;

#[derive(Debug)]
pub struct AppState {
    db: DatabaseConnection,
    env: Config,
    scanning_complete: Mutex<bool>,
    scanning_progress: Mutex<f64>,
}

#[tokio::main]
async fn main() -> Result<(), DbErr> {
    dotenvy::dotenv().ok();
    let config = Config::init();

    let addr_str = format!("{}:{}", config.server_address, config.server_port);

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let db = Database::connect(&config.database_url).await?;
    let db = match db.get_database_backend() {
        DbBackend::Sqlite => db,
        _ => {
            tracing::error!("we don't support other databases outside of sqlite. exiting.");
            std::process::exit(1)
        }
    };

    let schema_manager = SchemaManager::new(&db);
    Migrator::up(&db, None).await?;
    assert!(schema_manager.has_table("users").await?);
    assert!(schema_manager.has_table("categories").await?);
    assert!(schema_manager.has_table("titles").await?);
    assert!(schema_manager.has_table("pages").await?);
    assert!(schema_manager.has_table("tags").await?);
    assert!(schema_manager.has_table("titles_tags").await?);
    assert!(schema_manager.has_table("bookmarks").await?);
    assert!(schema_manager.has_table("thumbnails").await?);
    assert!(schema_manager.has_table("favorites").await?);
    assert!(schema_manager.has_table("progresses").await?);

    info!("database migrations complete!");

    let app_state = Arc::new(AppState {
        db,
        env: config.clone(),
        scanning_complete: Mutex::new(false),
        scanning_progress: Mutex::new(0.0),
    });

    let auth_routes = Router::new()
        .route("/register", post(post_register))
        .route("/login", post(post_login))
        .route(
            "/logout",
            get(get_logout).route_layer(apply(app_state.clone(), auth)),
        );

    let utils_routes = Router::new()
        .route("/tags", get(get_tags))
        .route("/scanning_progress", get(get_scanning_progress))
        .route("/ssim_eval", get(get_ssim_eval))
        .layer(apply(app_state.clone(), auth));

    let user_routes = Router::new()
        .route("/check", get(get_check))
        .route("/reset", post(post_reset))
        .route("/delete", get(get_delete).post(post_delete))
        .route("/verify", get(get_verify).post(post_verify))
        .route("/modify", post(post_modify))
        .route("/bookmark/:id", put(put_bookmark).delete(delete_bookmark))
        .route("/favorite/:id", put(put_favorite).delete(delete_favorite))
        .route("/progress/:title_id/:page", put(put_progress))
        .layer(apply(app_state.clone(), auth));

    let index_routes = Router::new()
        .route("/filter", post(post_filter))
        .route("/categories", get(get_categories))
        .route("/title/:title_id", get(get_title))
        .layer(apply(app_state.clone(), auth));

    let file_routes = Router::new()
        .route("/page/:page_id", get(get_page))
        .route("/thumbnail/:thumbnail_id", get(get_thumbnail))
        .layer(apply(app_state.clone(), auth));

    let open_routes = Router::new()
        .route("/user/reset/:email", get(get_reset))
        .route("/utils/status", get(get_status).post(post_status));

    let app = Router::new()
        .nest("/api/auth", auth_routes)
        .nest("/api/index", index_routes)
        .nest("/api/user", user_routes)
        .nest("/api/utils", utils_routes)
        .nest("/api/file", file_routes)
        .nest("/api", open_routes)
        .merge(SwaggerUi::new("/swagger").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(app_state.clone());

    let addr = addr_str.parse::<SocketAddr>().unwrap();
    let listener = TcpListener::bind(&addr).await.unwrap();

    let server_handle = tokio::spawn(async move {
        tracing::debug!("listening on: {}", addr);
        if let Err(e) = axum::serve(listener, app.into_make_service()).await {
            tracing::error!("server error: {}", e);
        };
    });

    let scanner_handle = tokio::spawn(async move {
        let instance = livescan::Scanner::new(app_state.clone());
        instance.await.run().await.unwrap();
    });

    let _ = server_handle.await;
    let _ = scanner_handle.await;

    Ok(())
}
