use std::path::PathBuf;

use image::{imageops::FilterType::Gaussian, DynamicImage, GenericImageView};

use crate::utils::{archive_file::ArchiveFile, macros::bail_if_empty};

/// Not contains the actual width and height, but the
/// `resize_to_fill(32, 32, Gaussian)` result. Decode a blurhash to the original
/// image's dimensions is resource intensive and unnecessary.
#[derive(Debug, Clone)]
pub struct ToBlurhashOk {
    pub blurhash: String,
    pub width: i32,
    pub height: i32,
}

/// A [`blurhash::encode`] wrapper that handles
/// the image resizing and dimension calculations.
fn encode(decoded_img: &DynamicImage) -> Result<ToBlurhashOk, blurhash::Error> {
    let (width, height) = decoded_img.dimensions();
    let decoded_img = decoded_img.resize_to_fill(32, 32, Gaussian);
    // TODO: somehow these dimensions are always 32x32
    let (smaller_width, smaller_height) = decoded_img.dimensions();
    let (components_x, components_y) = {
        let scale = smaller_width.min(smaller_height) / 3;
        (smaller_width / scale, smaller_height / scale)
    };

    Ok(ToBlurhashOk {
        blurhash: blurhash::encode(
            components_x,
            components_y,
            smaller_width,
            smaller_height,
            &decoded_img.to_rgba8().into_vec(),
        )?,
        width: width as i32,
        height: height as i32,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum ToBlurhashFromFileErr {
    #[error("can't decode image: {0:?}")]
    DecodeImage(#[from] image::ImageError),
    #[error("can't encode to blurhash: {0:?}")]
    EncodeBlurhash(blurhash::Error),
    #[error("can't read file: {0:?}")]
    ReadFile(#[from] std::io::Error),
}

pub trait ToBlurhashFromFile {
    fn to_blurhash_from_file(&self) -> Result<ToBlurhashOk, ToBlurhashFromFileErr>;
}

impl ToBlurhashFromFile for PathBuf {
    /// Assume the path is an image file and try to encode it to blurhash.
    fn to_blurhash_from_file(&self) -> Result<ToBlurhashOk, ToBlurhashFromFileErr> {
        let buf = std::fs::read(self)?;
        let img = image::load_from_memory(&buf)?;
        encode(&img).map_err(ToBlurhashFromFileErr::EncodeBlurhash)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ToBlurhashFromArchiveErr {
    #[error("file not found in archive")]
    FileNotFound,
    #[error("can't decode image: {0:?}")]
    DecodeImage(#[from] image::ImageError),
    #[error("can't encode to blurhash: {0:?}")]
    EncodeBlurhash(#[from] blurhash::Error),
    #[error("can't read archive: {0:?}")]
    ArchiveFile(#[from] crate::utils::archive_file::ArchiveFileError),
}

pub trait ToBlurhashFromArchive {
    fn to_blurhash_from_archive(
        &self,
        filename: &str,
    ) -> Result<ToBlurhashOk, ToBlurhashFromArchiveErr>;
}

impl ToBlurhashFromArchive for PathBuf {
    /// Assume the path is an archive file and
    /// try to encode a file from it to blurhash.
    fn to_blurhash_from_archive(
        &self,
        filename: &str,
    ) -> Result<ToBlurhashOk, ToBlurhashFromArchiveErr> {
        let buf = self.read_file_from_archive(filename)?;
        bail_if_empty!(buf, Err(ToBlurhashFromArchiveErr::FileNotFound));

        let img = image::load_from_memory(&buf)?;
        encode(&img).map_err(ToBlurhashFromArchiveErr::EncodeBlurhash)
    }
}
