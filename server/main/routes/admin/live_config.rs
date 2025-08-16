use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    routes::errors::{InternalErr, RequestErr},
    structs::ids::UserID,
    utils::{app_state::AppState, constants::LIVE_CONFIG_PATH, live_config::LiveConfigItem},
};

#[derive(Debug, ToSchema, Serialize, Deserialize)]
pub struct SetLiveConfigRequest {
    pub nomedia_support: Option<bool>,
    pub komga_oneshot_support: Option<bool>,
    pub komga_recycle_support: Option<bool>,
    pub rescan_enabled: Option<bool>,
    pub rescan_interval_in_minutes: Option<u32>,
}

/// Set live config
#[utoipa::path(post, path = LIVE_CONFIG_PATH,
    responses(
    (status = 200, description = "Config applied successfully", body = Option<GetLiveConfigResponse>),
    (status = 204, description = "Config applied successfully, but nothing changed", body = String),
    (status = 401, description = "Unauthorized", body = String),
    (status = 500, description = "Internal server error", body = String),
), security(("session-id" = [], "session-secret" = [])))]
pub async fn post_live_config(
    State(app_state): State<Arc<AppState>>,
    Extension(_user_id): Extension<UserID>,
    query: Json<SetLiveConfigRequest>,
) -> Result<Response, InternalErr> {
    let mut target = None;

    if let Some(nomedia_support) = query.nomedia_support {
        target = Some((LiveConfigItem::NomediaSupport, nomedia_support.to_string()));
    }
    if let Some(komga_oneshot_support) = query.komga_oneshot_support {
        target = Some((
            LiveConfigItem::KomgaOneshotSupport,
            komga_oneshot_support.to_string(),
        ));
    }
    if let Some(komga_recycle_support) = query.komga_recycle_support {
        target = Some((
            LiveConfigItem::KomgaRecycleSupport,
            komga_recycle_support.to_string(),
        ));
    }
    if let Some(rescan_enabled) = query.rescan_enabled {
        target = Some((LiveConfigItem::RescanEnabled, rescan_enabled.to_string()));
    }
    if let Some(rescan_interval_in_minutes) = query.rescan_interval_in_minutes {
        target = Some((
            LiveConfigItem::RescanIntervalInMinutes,
            rescan_interval_in_minutes.to_string(),
        ));
    }

    if let Some(target) = target {
        app_state
            .live_config
            .set(&app_state.pool, &target.0, &target.1)
            .await?;
        return Ok(StatusCode::OK.into_response());
    }

    Ok((StatusCode::NO_CONTENT, RequestErr::SetLiveConfigDoNothing).into_response())
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, ToSchema, Serialize, Deserialize)]
pub struct GetLiveConfigResponse {
    pub nomedia_support: bool,
    pub komga_oneshot_support: bool,
    pub komga_recycle_support: bool,
    pub rescan_enabled: bool,
    pub rescan_interval_in_minutes: u32,
}

/// Get live config
#[utoipa::path(post, path = LIVE_CONFIG_PATH, responses(
    (status = 204, description = "Config fetched successfully", body = GetLiveConfigResponse),
    (status = 401, description = "Unauthorized", body = String),
    (status = 500, description = "Internal server error", body = String),
))]
pub async fn get_live_config(
    State(app_state): State<Arc<AppState>>,
) -> Result<Response, InternalErr> {
    Ok((
        StatusCode::OK,
        Json(GetLiveConfigResponse {
            nomedia_support: app_state.live_config.get_nomediasupport().await,
            komga_oneshot_support: app_state.live_config.get_komga_oneshot_support().await,
            komga_recycle_support: app_state.live_config.get_komga_recycle_support().await,
            rescan_enabled: app_state.live_config.get_rescan_enabled().await,
            rescan_interval_in_minutes: app_state
                .live_config
                .get_rescan_interval_in_minutes()
                .await,
        }),
    )
        .into_response())
}
