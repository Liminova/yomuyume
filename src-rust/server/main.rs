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
mod structs;
mod traits;
mod utils;

use std::{net::SocketAddr, time::Duration};

use anyhow::Result;
use app_state::AppState;
use axum::{
    middleware::from_fn_with_state as apply,
    routing::{get, post, put},
    Router,
};
use routes::user::post_sensitive;
use tokio::{net::TcpListener, time::sleep};
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info};
use utoipa::OpenApi;
use utoipa_redoc::{Redoc, Servable};

use crate::{
    routes::{
        auth::{get_logout, post_login, post_register},
        content::{get_categories, get_chapter, get_oneshot, get_series, get_tags, post_search},
        file::{get_cover, get_page},
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
        .nest(
            "/api/auth",
            Router::new()
                .route("/register", post(post_register))
                .route("/login", post(post_login))
                .route("/logout", get(get_logout)),
        )
        .nest(
            "/api/content",
            Router::new()
                .route("/search", post(post_search))
                .route("/categories", get(get_categories))
                .route("/series/{title_id}", get(get_series))
                .route("/oneshot/{title_id}", get(get_oneshot))
                .route("/chapter/{chapter_id}", get(get_chapter))
                .route("/tags", get(get_tags))
                .layer(apply(app_state.clone(), auth)),
        )
        .nest(
            "/api/user",
            Router::new()
                .route("/whoami", get(get_whoami))
                .route("/sensitive", post(post_sensitive))
                .route("/modify", post(post_modify))
                .route(
                    "/bookmark/{title_id}",
                    put(put_bookmark).delete(delete_bookmark),
                )
                .route(
                    "/favorite/{title_id}",
                    put(put_favorite).delete(delete_favorite),
                )
                .route("/progress/{title_id}/{page}", put(put_progress))
                .layer(apply(app_state.clone(), auth)),
        )
        .nest(
            "/api/utils",
            Router::new()
                .route("/scanning_progress", get(get_scanning_progress))
                .layer(apply(app_state.clone(), auth)),
        )
        .nest(
            "/api/file",
            Router::new()
                .route("/page/{is_series}/{page_id}", get(get_page))
                .route("/cover/{title_id}", get(get_cover))
                .layer(apply(app_state.clone(), auth)),
        )
        .nest(
            "/api",
            Router::new().route("/utils/status", get(get_status).post(post_status)),
        )
        .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
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
