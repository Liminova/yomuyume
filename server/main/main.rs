#![forbid(unsafe_code)]
#![warn(clippy::nursery, clippy::pedantic, clippy::perf, clippy::unwrap_used)]
#![allow(clippy::doc_markdown, clippy::too_many_lines, clippy::use_self, unused)] // TODO: remove unused

mod config;
mod indexer;
mod routes;
mod utils;

use std::{net::SocketAddr, sync::Arc};

use axum::{
    Router,
    body::Body,
    extract::Request,
    http::{StatusCode, header},
    middleware::from_fn_with_state as apply,
    response::IntoResponse,
    routing::{get, post, put},
};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};

use crate::{
    config::Config,
    indexer::{Indexer, start_index::start_index},
    routes::{
        ApiDoc,
        auth::{get_logout, post_forgot, post_login, post_register},
        file::{get_cover_file, get_page_file},
        middlewares::auth::auth,
        user::{get_whoami, post_modify, put_progress},
        utils::{get_status, post_status},
    },
    utils::constants::{
        FORGOT_PATH, GET_COVER_FILE_PATH, GET_PAGE_FILE_PATH, GET_STATUS_PATH, GET_WHOAMI_PATH,
        LOGIN_PATH, LOGOUT_PATH, POST_USER_MODIFY_PATH, PUT_READ_PROGRESS_PATH, REGISTER_PATH,
    },
};
use frontend_spa::{Content, get_file};

#[derive(Debug)]
pub struct AppState {
    pub pool: sqlx::SqlitePool,
    pub config: Config,
    pub indexer: Indexer,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    tracing_subscriber::fmt()
        .with_ansi(true)
        .with_max_level(tracing::Level::DEBUG)
        .with_env_filter("sqlx=info,axum=info,yomuyume=debug")
        .with_file(true)
        .with_line_number(true)
        .init();

    let config = Config::init();
    let app_state = Arc::new(AppState {
        pool: sqlx::SqlitePool::connect(&config.database_url)
            .await
            .expect("can't connect to database"),
        indexer: Indexer::new(config.tantivy_dir.as_ref(), config.tantivy_memory)
            .expect("can't initialize tantivy"),
        config,
    });

    #[cfg(not(debug_assertions))]
    {
        let migrator = sqlx::migrate!("../../migrations");
        info!("applying {} migrations", migrator.iter().len());
        migrator
            .run(&app_state.pool)
            .await
            .expect("database migration failed");
    }

    let app = Router::new()
        .route(REGISTER_PATH, post(post_register))
        .route(LOGIN_PATH, post(post_login))
        .route(LOGOUT_PATH, get(get_logout))
        .route(FORGOT_PATH, post(post_forgot))
        .merge(
            Router::new()
                // TODO: add OPDS routes here
                // user
                .route(GET_WHOAMI_PATH, get(get_whoami))
                .route(POST_USER_MODIFY_PATH, post(post_modify))
                .route(PUT_READ_PROGRESS_PATH, put(put_progress))
                // file
                .route(GET_PAGE_FILE_PATH, get(get_page_file))
                .route(GET_COVER_FILE_PATH, get(get_cover_file))
                // middleware
                .layer(apply(app_state.clone(), auth)),
        )
        .route(GET_STATUS_PATH, get(get_status).post(post_status))
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
        .fallback(|req: Request<Body>| async move {
            let Content { content, mime_type } = get_file(
                req.uri()
                    .path()
                    .trim_start_matches('/')
                    .trim_end_matches('/'),
            );

            (StatusCode::OK, [(header::CONTENT_TYPE, mime_type)], content).into_response()
        })
        .layer(TraceLayer::new_for_http())
        .with_state(app_state.clone());

    start_index(app_state.clone()).await;

    let listen_address = app_state.config.listen_address.clone();
    let server_handle = tokio::spawn(async move {
        info!("listening on: {}", &listen_address);

        if let Err(e) = axum::serve(
            TcpListener::bind(&listen_address)
                .await
                .expect("can't start tcp listener"),
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        {
            error!("server error: {e:?}");
        }
    });

    server_handle.await.expect("uh oh");

    Ok(())
}
