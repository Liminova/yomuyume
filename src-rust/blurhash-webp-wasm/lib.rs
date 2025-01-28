#![allow(clippy::excessive_precision)]

use std::{f32::consts::PI, io::Cursor};

use image::{ImageBuffer, Rgba};
use wasm_bindgen::prelude::*;

const DIGIT_LOOKUP: [i32; 128] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 62, 63, 64, 0, 0, 0, 0, 65, 66, 67, 68, 69, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 70, 71,
    0, 72, 0, 73, 74, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28,
    29, 30, 31, 32, 33, 34, 35, 75, 0, 76, 77, 78, 0, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46,
    47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 79, 80, 81, 82, 0,
];

fn decode83(str: &str, start: usize, end: usize) -> i32 {
    let mut value = 0;
    let mut start = start;
    while start < end {
        value *= 83;
        value += DIGIT_LOOKUP[str.as_bytes()[start] as usize];
        start += 1;
    }
    value
}

fn srgb_to_linear(value: f32) -> f32 {
    if value > 10.31475 {
        (value / 269.025 + 0.052132).powf(2.4)
    } else {
        value / 3294.6
    }
}

fn clamp(v: f32, min: f32, max: f32) -> f32 {
    v.max(min).min(max)
}

fn linear_to_srgb(v: f32) -> u8 {
    if v > 0.00001227 {
        // 269.025 * v.powf(0.416666) - 13.025
        clamp(269.025 * v.powf(0.416666) - 13.025, 0.0, 255.0) as u8
    } else {
        // v * 3294.6 + 1.0
        clamp(v * 3294.6 + 1.0, 0.0, 255.0) as u8
    }
}

fn sign_sqr(x: f32) -> f32 {
    if x < 0.0 {
        -1.0 * x * x
    } else {
        x * x
    }
}

/// Fast approximate cosine implementation
/// Based on FTrig https://github.com/netcell/FTrig
fn fast_cos(x: f32) -> f32 {
    let mut x = x + PI / 2.0;
    while x > PI {
        x -= PI * 2.0;
    }
    let cos = 1.273_239_54 * x - 0.405_284_735 * sign_sqr(x);
    0.225 * (sign_sqr(cos) - cos) + cos
}

/// Extracts average color from blurhash
///
/// # Arguments
/// * `blurhash` - Blurhash string
///
/// # Returns
/// `[r, g, b]`
#[wasm_bindgen]
pub fn get_blur_hash_average_color(blurhash: &str) -> Vec<i32> {
    let val = decode83(blurhash, 2, 6);
    vec![val >> 16, (val >> 8) & 255, val & 255]
}

/// Decode blurhash to WebP image
///
/// # Arguments
/// * `blurhash` - Blurhash string
/// * `width` - Width of the image
/// * `height` - Height of the image
/// * `punch` - Punch value
/// # Returns
/// WebP image bytes
#[wasm_bindgen]
pub fn decode(blurhash: &str, width: usize, height: usize, punch: Option<f32>) -> Vec<u8> {
    let size_flag = decode83(blurhash, 0, 1);
    let num_x = ((size_flag % 9) + 1) as usize;
    let num_y = (size_flag / 9 + 1) as usize;
    let size = num_x * num_y;

    let maximum_value = ((decode83(blurhash, 1, 2) + 1) as f32 / 13446.0) * punch.unwrap_or(1.0);

    let mut colors = vec![0.0; size * 3];

    let average_color = get_blur_hash_average_color(blurhash);
    for i in 0..3 {
        colors[i] = srgb_to_linear(average_color[i] as f32);
    }

    for i in 1..size as usize {
        let value = decode83(blurhash, 4 + i * 2, 6 + i * 2) as f32;
        colors[i * 3] = sign_sqr((value / 361.0) - 9.0) * maximum_value;
        colors[i * 3 + 1] = sign_sqr((value / 19.0) % 19.0 - 9.0) * maximum_value;
        colors[i * 3 + 2] = sign_sqr((value % 19.0) - 9.0) * maximum_value;
    }

    let mut cosines_y = vec![0.0; num_y * height];
    let mut cosines_x = vec![0.0; num_x * width];
    for j in 0..num_y {
        for y in 0..height {
            cosines_y[j * height + y] = fast_cos(PI * y as f32 * j as f32 / height as f32);
        }
    }
    for i in 0..num_x {
        for x in 0..width {
            cosines_x[i * width + x] = fast_cos(PI * x as f32 * i as f32 / width as f32);
        }
    }

    let bytes_per_row = width * 4;
    let mut pixels = vec![0; bytes_per_row * height];

    for y in 0..height {
        for x in 0..width {
            let mut r = 0.0;
            let mut g = 0.0;
            let mut b = 0.0;
            for j in 0..num_y {
                let basis_y = cosines_y[j * height + y];
                for i in 0..num_x {
                    let basis = cosines_x[i * width + x] * basis_y;
                    let color_index = (i + j * num_x) * 3;
                    r += colors[color_index] * basis;
                    g += colors[color_index + 1] * basis;
                    b += colors[color_index + 2] * basis;
                }
            }

            let pixel_index = 4 * x + y * bytes_per_row;
            pixels[pixel_index] = linear_to_srgb(r);
            pixels[pixel_index + 1] = linear_to_srgb(g);
            pixels[pixel_index + 2] = linear_to_srgb(b);
            pixels[pixel_index + 3] = 255; // alpha
        }
    }

    let mut cursor = Cursor::new(Vec::new());

    ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(width as u32, height as u32, pixels)
        .unwrap()
        .write_to(&mut cursor, image::ImageFormat::WebP)
        .unwrap();

    cursor.into_inner()
}
