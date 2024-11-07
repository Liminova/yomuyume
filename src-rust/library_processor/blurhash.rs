use anyhow::Result;
use image::{imageops::FilterType::Gaussian, DynamicImage, GenericImageView};

/// Not contains the actual width and height, but the
/// `resize_to_fill(32, 32, Gaussian)` result. Decode a blurhash to the original
/// image's dimensions is resource intensive and unnecessary.
#[derive(Debug, Clone)]
pub struct BlurhashResult {
    pub blurhash: String,
    pub width: i32,
    pub height: i32,
}

/// A [`blurhash::encode`] wrapper that handles
/// the image resizing and dimension calculations.
pub fn encode(decoded_img: &DynamicImage) -> Result<BlurhashResult> {
    let (width, height) = decoded_img.dimensions();
    let decoded_img = decoded_img.resize_to_fill(32, 32, Gaussian);
    let (smaller_width, smaller_height) = decoded_img.dimensions();
    let (components_x, components_y) = {
        let scale = smaller_width.min(smaller_height) / 3;
        (smaller_width / scale, smaller_height / scale)
    };

    Ok(BlurhashResult {
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
