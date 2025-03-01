use std::{
    io::Cursor,
    path::{Path, PathBuf},
};

use image::{imageops::FilterType::Gaussian, DynamicImage, GenericImageView};
use jxl_oxide::integration::JxlDecoder;

use crate::utils::archive_file::ArchiveFile;

/// Not contains the actual width and height, but the
/// `resize_to_fill(32, 32, Gaussian)` result. Decode a blurhash to the original
/// image's dimensions is resource intensive and unnecessary.
#[derive(Debug, Clone)]
pub struct ToBlurhashOk {
    pub blurhash: String,
    pub width: i32,
    pub height: i32,
}

trait ToBlurhash {
    fn to_blurhash(&self) -> Result<ToBlurhashOk, blurhash::Error>;
}

impl ToBlurhash for DynamicImage {
    /// A [`blurhash::encode`] wrapper that handles
    /// the image resizing and dimension calculations.
    fn to_blurhash(&self) -> Result<ToBlurhashOk, blurhash::Error> {
        let (width, height) = self.dimensions();
        let decoded_img = self.resize_to_fill(32, 32, Gaussian);
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

        if self
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("jxl"))
        {
            DynamicImage::from_decoder(JxlDecoder::new(Cursor::new(buf))?)?
        } else {
            image::load_from_memory(&buf)?
        }
        .to_blurhash()
        .map_err(ToBlurhashFromFileErr::EncodeBlurhash)
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
        if buf.is_empty() {
            return Err(ToBlurhashFromArchiveErr::FileNotFound);
        }

        if Path::new(filename)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("jxl"))
        {
            DynamicImage::from_decoder(JxlDecoder::new(Cursor::new(buf))?)?
        } else {
            image::load_from_memory(&buf)?
        }
        .to_blurhash()
        .map_err(ToBlurhashFromArchiveErr::EncodeBlurhash)
    }
}
