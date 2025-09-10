use image::{DynamicImage, GenericImageView, Pixel};
use serde::{Deserialize, Serialize};

pub trait AverageColor {
    fn average_color(&self) -> Option<HexColor>;
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct HexColor((u8, u8, u8));

impl From<HexColor> for String {
    fn from(hex: HexColor) -> Self {
        format!("#{:02x}{:02x}{:02x}", hex.0.0, hex.0.1, hex.0.2)
    }
}

impl std::fmt::Display for HexColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.0.0, self.0.1, self.0.2)
    }
}

impl HexColor {
    pub fn from_str(s: &str) -> Option<Self> {
        let s = s.strip_prefix('#').unwrap_or(s);
        if s.len() != 6 {
            return None;
        }
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some(HexColor((r, g, b)))
    }
}

impl AverageColor for DynamicImage {
    fn average_color(&self) -> Option<HexColor> {
        let (width, height) = self.dimensions();
        let total_pixels = u64::from(width) * u64::from(height);

        if total_pixels == 0 {
            return None; // Handle empty images
        }

        let mut sum_r: u64 = 0;
        let mut sum_g: u64 = 0;
        let mut sum_b: u64 = 0;

        // Iterate through each pixel and sum its color components.
        for (_x, _y, pixel) in self.pixels() {
            let rgba = pixel.to_rgb(); // Convert to RGBA8 format
            sum_r += u64::from(rgba[0]);
            sum_g += u64::from(rgba[1]);
            sum_b += u64::from(rgba[2]);
        }

        // Calculate the average for each component.
        let avg_r = (sum_r / total_pixels) as u8;
        let avg_g = (sum_g / total_pixels) as u8;
        let avg_b = (sum_b / total_pixels) as u8;

        Some(HexColor((avg_r, avg_g, avg_b)))
    }
}
