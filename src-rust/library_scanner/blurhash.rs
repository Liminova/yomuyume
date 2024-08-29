use image::{imageops::FilterType::Gaussian, DynamicImage, GenericImageView};

/// Not contains the actual width and height, but the
/// `resize_to_fill(32, 32, Gaussian)` result. Decode a blurhash to the original
/// image's dimensions is resource intensive and unnecessary.
#[derive(Debug, Clone)]
pub struct BlurhashResult {
    pub blurhash: String,
    pub small_width: u8,
    pub small_height: u8,
}

/// A [`blurhash::encode`] wrapper that handles
/// the image resizing and dimension calculations.
pub fn encode(decoded_img: &DynamicImage) -> Result<BlurhashResult, String> {
    let decoded_img = decoded_img.resize_to_fill(32, 32, Gaussian);
    let (width, height) = decoded_img.dimensions();
    let (components_x, components_y) = {
        let scale = width.min(height) / 3;
        (width / scale, height / scale)
    };

    let encoded = blurhash::encode(
        components_x,
        components_y,
        width,
        height,
        &decoded_img.to_rgba8().into_vec(),
    )
    .map_err(|err| format!("can't encode blurhash: {}", err))?;

    Ok(BlurhashResult {
        blurhash: encoded,
        small_width: width as u8,
        small_height: height as u8,
    })
}
