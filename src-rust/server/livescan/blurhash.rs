use crate::constants::{native_img_formats, ratio_percision};
use blurhash::encode;
use image::{imageops::FilterType::Gaussian, DynamicImage, GenericImageView};
use std::path::PathBuf;
use std::{fs, process::Command};
use tracing::{debug, error, warn};

#[derive(Debug)]
pub struct Blurhash {
    pub ffmpeg_path: Option<String>,
    pub djxl_path: Option<String>,
    pub ffmpeg_log_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BlurhashResult {
    pub blurhash: String,
    pub ratio: u32,
    /// This is the file's name, not the full path
    pub file_name: String,
}

impl Blurhash {
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
    /// Being called in thumbnail_finder and handle_tite, there're already a
    /// extension extraction to check against the supported formats, no need to
    /// re-extract the extension again, wasting cpu cycles.
    #[tracing::instrument]
    pub fn encode(&self, image_path: &PathBuf, format: &str) -> Option<BlurhashResult> {
        let input_img_path = image_path.to_str().unwrap_or_default();

        let decoded_image = self
            .transcode(input_img_path, format)?
            .resize(100, 100, Gaussian);
        let (width, height) = decoded_image.dimensions();

        let scale = width.min(height) / 3;
        let (x, y) = (width / scale, height / scale);

        let encoded = encode(x, y, width, height, &decoded_image.to_rgba8().into_vec())
            .map_err(|_| error!("failed to encode to blurhash"))
            .ok()?
            .to_string();

        let file_name = image_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        Some(BlurhashResult {
            blurhash: encoded,
            ratio: (width * ratio_percision()) / height,
            file_name,
        })
    }

    #[tracing::instrument]
    pub fn transcode(&self, in_file: &str, format: &str) -> Option<DynamicImage> {
        match format {
            format if native_img_formats().contains(&format) => {
                debug!("native");
                image::open(in_file)
                    .map_err(|err| {
                        error!("failed to open: {}", err);
                        err
                    })
                    .ok()
            }
            "jxl" => {
                debug!("djxl");
                self.jpegxl(in_file)
            }
            _ => {
                debug!("ffmpeg");
                self.ffmpeg(in_file)
            }
        }
    }

    #[tracing::instrument]
    fn ffmpeg(&self, in_file: &str) -> Option<DynamicImage> {
        let ffmpeg = match self.ffmpeg_path {
            Some(ref path) => path.clone(),
            None => {
                warn!("ffmpeg not found, please set the FFMPEG_PATH environment variable");
                return None;
            }
        };

        let output = Command::new(ffmpeg)
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
            .ok()?;

        if !output.status.success() {
            let decode_log = self.ffmpeg_log_path.as_ref();
            if let Some(decode_log) = decode_log {
                let err_msg = format!(
                    "ffmpeg failed with code {}",
                    output.status.code().unwrap_or(-1)
                );
                let err = String::from_utf8(output.stderr)
                    .map_err(|_| error!("{}", err_msg))
                    .ok();

                let err_msg = format!("failed to write ffmpeg decode log to {}", decode_log);
                fs::write(decode_log, err.unwrap_or_default())
                    .map_err(|_| err_msg)
                    .ok();
            }
        }

        image::load_from_memory(&output.stdout)
            .map_err(|err| {
                let err_msg = format!(
                    "might be decoded, but failed to load from ffmpeg stdout: {}",
                    err
                );
                error!("{}", err_msg);
            })
            .ok()
    }

    #[tracing::instrument]
    fn jpegxl(&self, in_file: &str) -> Option<DynamicImage> {
        let djxl = self
            .djxl_path
            .as_ref()
            .ok_or_else(|| warn!("djxl not found, please set the DJXL_PATH environment variable"))
            .ok()?;

        let output_path = format!("{}.png", in_file);

        let output = Command::new(djxl)
            .args([in_file, &output_path])
            .output()
            .ok()?;

        if !output.status.success() {
            error!(
                "djxl failed with code {}",
                output.status.code().unwrap_or(-1)
            );
            return None;
        }

        image::open(&output_path)
            .map_err(|err| {
                let err_msg = format!(
                    "might be decoded, but failed to load from temp file: {}",
                    err
                );
                error!(err_msg);
            })
            .ok()
    }
}
