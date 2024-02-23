use std::{path::PathBuf, sync::Arc};

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use rand::{thread_rng, Rng};
use sea_orm::{ColumnTrait, Condition, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
    models::prelude::*,
    routes::{calculate_dimension, ErrRsp},
    AppState,
};

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SsimEvalTitle {
    pub id: String,

    pub title: String,
    pub desc: String,
    pub tags: Vec<u32>,

    pub blurhash: String,
    pub width: u32,
    pub height: u32,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct SsimEvalBody {
    pub title_a: SsimEvalTitle,
    pub title_b: SsimEvalTitle,
    pub ssim: f32,
}

async fn random_pair(range: std::ops::Range<u64>) -> (u64, u64) {
    let mut rng = thread_rng();
    let title_a_index = rng.gen_range(range.clone());
    let title_b_index = loop {
        let index = rng.gen_range(range.clone());
        if index != title_a_index {
            break index;
        }
    };
    (title_a_index, title_b_index)
}

async fn get_title(index: u64, data: &Arc<AppState>) -> Result<SsimEvalTitle, ErrRsp> {
    let title = Titles::find()
        .offset(index)
        .one(&data.db)
        .await
        .map_err(ErrRsp::db)?
        .ok_or_else(|| ErrRsp::internal("Can't find title"))?;

    let thumbnail = Thumbnails::find_by_id(&title.id)
        .one(&data.db)
        .await
        .map_err(ErrRsp::db)?
        .ok_or_else(|| ErrRsp::internal("Can't find thumbnail"))?;

    let tags = TitlesTags::find()
        .filter(titles_tags::Column::TitleId.eq(&title.id))
        .all(&data.db)
        .await
        .map_err(ErrRsp::db)?
        .into_iter()
        .map(|title_tag| title_tag.tag_id)
        .collect::<Vec<_>>();

    let (width, height) = calculate_dimension(thumbnail.ratio);

    Ok(SsimEvalTitle {
        id: title.id,
        title: title.title,
        desc: title.description.unwrap_or_default(),
        tags,
        blurhash: thumbnail.blurhash,
        width,
        height,
        format: PathBuf::from(thumbnail.path)
            .extension()
            .map(|s| s.to_str().unwrap_or(""))
            .unwrap_or("")
            .to_ascii_lowercase(),
    })
}

// Return 2 random titles from DB and their SSIM score
#[utoipa::path(get, path = "/api/utils/ssim_eval", responses(
    (status = 200, description = "2 random title", body = SsimEvalBody),
    (status = 401, description = "Unauthorized", body = ErrorResponseBody),
    (status = 500, description = "Internal server error", body = ErrorResponseBody),
))]
pub async fn get_ssim_eval(State(data): State<Arc<AppState>>) -> Result<impl IntoResponse, ErrRsp> {
    let title_count = Titles::find().count(&data.db).await.unwrap();
    let (title_a_index, title_b_index) = random_pair(0..title_count).await;

    let title_a = get_title(title_a_index, &data).await?;
    let title_b = get_title(title_b_index, &data).await?;

    let condition = Condition::any()
        .add(
            Condition::all()
                .add(titles_ssim::Column::TitleIdA.eq(title_a.id.clone()))
                .add(titles_ssim::Column::TitleIdB.eq(title_b.id.clone())),
        )
        .add(
            Condition::all()
                .add(titles_ssim::Column::TitleIdA.eq(title_b.id.clone()))
                .add(titles_ssim::Column::TitleIdB.eq(title_a.id.clone())),
        );

    let ssim_score = TitlesSsim::find()
        .filter(condition)
        .one(&data.db)
        .await
        .map_err(ErrRsp::db)?
        .ok_or_else(|| ErrRsp::internal("Can't find ssim score"))?
        .ssim as f32
        / 1000.0;

    Ok((
        StatusCode::OK,
        Json(SsimEvalBody {
            title_a,
            title_b,
            ssim: ssim_score,
        }),
    ))
}
