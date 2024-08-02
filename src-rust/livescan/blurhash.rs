use std::{fs, path::Path, process::Command};

use crate::config::Config;

use image::{imageops::FilterType::Gaussian, DynamicImage, GenericImageView};
use tracing::{error, warn};

#[derive(Debug, Clone)]
pub struct BlurhashResult {
    pub blurhash: String,
    pub ratio: u32,
}

/// Encodes the image at the given path into a blurhash.
///
/// # Arguments
///
/// * `image_path` - A path to the image to encode.
/// * `format` - The format of the image (e.g., "png", "jpg").
///
/// # Returns
///
/// * `Option<BlurhashResult>` - The result of the encoding, or `None` if the encoding failed.
///
/// # Note
///
/// Being called in cover_finder and handle_tite, there're already a
/// extension extraction to check against the supported formats, no need to
/// re-extract the extension again, wasting cpu cycles.
pub fn encode(config: &Config, image_path: &Path) -> Result<BlurhashResult, String> {
    let input_img_path = image_path.to_str().unwrap_or_default();
    let img_format = image_path
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let (img_width, img_height, decoded_img) =
        decode(config, input_img_path, &img_format).map(|decoded| {
            let (width, height) = decoded.dimensions();
            let decoded = decoded.resize_exact(32, 32, Gaussian);
            (width, height, decoded)
        })?;

    let scale = img_width.min(img_height) / 3;
    let (bh_width, bh_height) = (img_width / scale, img_height / scale);

    let encoded = blurhash::encode(
        bh_width,
        bh_height,
        img_width,
        img_height,
        &decoded_img.to_rgba8().into_vec(),
    )
    .map_err(|err| format!("can't encode blurhash: {}", err))?;

    Ok(BlurhashResult {
        blurhash: encoded,
        ratio: (img_width * config.ratio_percision) / img_height,
    })
}

#[tracing::instrument]
fn decode(config: &Config, in_file: &str, format: &str) -> Result<DynamicImage, String> {
    match format {
        format if config.native_img_formats.contains(&format) => {
            image::open(in_file).map_err(|err| err.to_string())
        }
        "jxl" => {
            let djxl = config
                .djxl_path
                .as_ref()
                .ok_or_else(|| "djxl not found in PATH".to_string())?;

            let out_file = format!("{}.png", in_file);

            let stdout = Command::new(djxl)
                .args([in_file, &out_file])
                .output()
                .map_err(|err| err.to_string())?;

            if !stdout.status.success() {
                return Err(format!(
                    "djxl failed with code {}",
                    stdout.status.code().unwrap_or(-1)
                ));
            }

            image::open(&out_file).map_err(|err| {
                format!(
                    "image might be decoded, but failed to load from /tmp dir: {}",
                    err
                )
            })
        }
        _ => {
            let ffmpeg = match config.ffmpeg_path {
                Some(ref path) => {
                    if !Path::new(path).exists() {
                        return Err(format!("FFMPEG_PATH {} does not exist", path));
                    }
                    path
                }
                None => return Err("FFMPEG_PATH is not set".to_string()),
            };

            let stdout = Command::new(ffmpeg)
                .args([
                    "-i",
                    in_file,
                    "-y",
                    "-vf",
                    "scale='min(100,iw)':'min(100,ih)':force_original_aspect_ratio=decrease",
                    "-f",
                    "image2pipe",
                    "-vcodec",
                    "png",
                    "-",
                ])
                .output()
                .map_err(|err| format!("ffmpeg failed: {}", err))?;

            if !stdout.status.success() {
                if let Err(e) = fs::write("/tmp/ffmpeg_decode.log", stdout.stderr) {
                    error!("failed to write ffmpeg decode log to /tmp: {}", e);
                }
            }

            image::load_from_memory(&stdout.stdout).map_err(|err| {
                format!(
                    "might be decoded, but failed to load from ffmpeg stdout: {}",
                    err
                )
            })
        }
    }
}
