use crate::models::prelude::*;
use rayon::prelude::*;
use rust_bert::pipelines::sentence_embeddings::SentenceEmbeddingsBuilder;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use tracing::{debug, error};

fn cosine_similarity(vec1: &[f32], vec2: &[f32]) -> f32 {
    let dot_product: f32 = vec1.iter().zip(vec2).map(|(&x1, &x2)| x1 * x2).sum();
    let vec1_norm: f32 = vec1.iter().map(|&x| x * x).sum::<f32>().sqrt();
    let vec2_norm: f32 = vec2.iter().map(|&x| x * x).sum::<f32>().sqrt();
    dot_product / (vec1_norm * vec2_norm)
}

#[derive(Clone)]
struct InputData {
    title_id: String,
    input_soup: String,
}

struct SsimCalculationJob {
    title_id_a: String,
    vector_a: Vec<f32>,
    title_id_b: String,
    vector_b: Vec<f32>,
}

struct SsimCalculationResult {
    title_id_a: String,
    title_id_b: String,
    similarity: f32,
}

fn build_ssim_jobs(vectors: Vec<Vec<f32>>, input_data: Vec<InputData>) -> Vec<SsimCalculationJob> {
    let mut ssim_calculation_job: Vec<SsimCalculationJob> = Vec::new();
    for (i, vector_a) in vectors.iter().enumerate() {
        'scoped: for (j, vector_b) in vectors.iter().enumerate().skip(i) {
            if i == j {
                continue 'scoped;
            }
            ssim_calculation_job.push(SsimCalculationJob {
                title_id_a: input_data[i].title_id.clone(),
                vector_a: vector_a.clone(),
                title_id_b: input_data[j].title_id.clone(),
                vector_b: vector_b.clone(),
            });
        }
    }
    ssim_calculation_job
}

async fn embedding(data: &[InputData], model_dir: String) -> Result<Vec<Vec<f32>>, String> {
    let mapped_input_data = data
        .to_owned()
        .clone()
        .into_iter()
        .map(|input| input.input_soup.clone())
        .collect::<Vec<_>>();

    let result = tokio::task::spawn_blocking(move || {
        let model = match SentenceEmbeddingsBuilder::local(model_dir)
            .with_device(tch::Device::cuda_if_available())
            .create_model()
        {
            Ok(model) => model,
            Err(e) => {
                error!("cannot create model: {}", e);
                return Err(e.to_string());
            }
        };
        match model.encode(&mapped_input_data) {
            Ok(vectors) => Ok(vectors),
            Err(e) => {
                error!("cannot encode input: {}", e);
                Err(e.to_string())
            }
        }
    })
    .await;

    match result {
        Ok(Ok(vectors)) => Ok(vectors),
        Ok(Err(e)) => Err(e),
        Err(e) => Err(e.to_string()),
    }
}

pub async fn title_ssim_score(
    db: &DatabaseConnection,
    model_dir: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // Prepare data
    let mut input_data: Vec<InputData> = Vec::new();
    for title in Titles::find().all(db).await?.iter() {
        let soup_tags = 'soup_tags: {
            let tag_ids = match TitlesTags::find()
                .filter(titles_tags::Column::TitleId.eq(title.id.clone()))
                .all(db)
                .await
            {
                Ok(tag_ids) => tag_ids
                    .into_iter()
                    .map(|title_tag| title_tag.tag_id)
                    .collect::<Vec<_>>(),
                Err(e) => {
                    error!("error finding tag ids for title: {}", e);
                    break 'soup_tags String::new();
                }
            };

            let mut tag_names = Vec::new();

            'scoped: for tag_id in tag_ids.into_iter() {
                match Tags::find_by_id(tag_id).one(db).await {
                    Ok(Some(tag)) => tag_names.push(tag.name),
                    Ok(None) => break 'soup_tags String::new(),
                    Err(e) => {
                        error!("error finding tag: {}", e);
                        continue 'scoped;
                    }
                }
            }

            tag_names.join(" ")
        };
        input_data.push(InputData {
            title_id: title.id.clone(),
            input_soup: format!(
                "{} {} {}",
                title.title.clone(),
                title.description.clone().unwrap_or_default(),
                soup_tags
            ),
        });
    }
    debug!("input data prepared");

    let vectors = embedding(&input_data, model_dir).await?;
    debug!("input data encoded");

    // Calculate similarity
    let jobs = build_ssim_jobs(vectors, input_data);
    let results: Vec<SsimCalculationResult> = jobs
        .into_par_iter()
        .map(|job| {
            let similarity = cosine_similarity(&job.vector_a, &job.vector_b);
            SsimCalculationResult {
                title_id_a: job.title_id_a,
                title_id_b: job.title_id_b,
                similarity,
            }
        })
        .collect();
    debug!("similarity calculated");

    // Remove everything from table
    TitlesSsim::delete_many().exec(db).await?;

    let active_models = results
        .into_iter()
        .map(|result| titles_ssim::ActiveModel {
            title_id_a: Set(result.title_id_a),
            title_id_b: Set(result.title_id_b),
            ssim: Set((result.similarity * 1000.0) as u16),
            ..Default::default()
        })
        .collect::<Vec<_>>();
    TitlesSsim::insert_many(active_models).exec(db).await?;

    Ok(())
}
