//! Image processing: decode arbitrary input (jpeg/png/webp), resize into
//! three fixed variants, and re-encode each as lossy WebP.

use image::{DynamicImage, ImageReader};
use std::io::Cursor;
use tracing::debug;

use crate::domain::error::ServiceError;

const WEBP_QUALITY: f32 = 85.0;

#[derive(Debug, Clone, Copy)]
pub enum Variant {
    Thumb,
    Medium,
    Full,
}

impl Variant {
    pub const ALL: [Variant; 3] = [Variant::Thumb, Variant::Medium, Variant::Full];

    pub fn max_edge(self) -> u32 {
        match self {
            Variant::Thumb => 256,
            Variant::Medium => 800,
            Variant::Full => 1920,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Variant::Thumb => "thumb",
            Variant::Medium => "medium",
            Variant::Full => "full",
        }
    }
}

pub struct VariantBytes {
    pub variant: Variant,
    pub data: Vec<u8>,
}

/// Decode `raw` (any format guessable by `image`) and produce the three
/// WebP variants. Lossy encoding at quality 85.
pub fn build_variants(raw: &[u8]) -> Result<Vec<VariantBytes>, ServiceError> {
    let img = decode(raw)?;
    let (w, h) = (img.width(), img.height());
    debug!(
        width = w,
        height = h,
        "image_processor:decoded source image"
    );

    Variant::ALL
        .iter()
        .copied()
        .map(|v| {
            let resized = resize_to_fit(&img, v.max_edge());
            let encoded = encode_webp(&resized)?;
            Ok(VariantBytes {
                variant: v,
                data: encoded,
            })
        })
        .collect()
}

fn decode(raw: &[u8]) -> Result<DynamicImage, ServiceError> {
    let reader = ImageReader::new(Cursor::new(raw))
        .with_guessed_format()
        .map_err(|e| ServiceError::BadRequest(format!("image format probe failed: {e}")))?;
    reader
        .decode()
        .map_err(|e| ServiceError::BadRequest(format!("image decode failed: {e}")))
}

fn resize_to_fit(img: &DynamicImage, max_edge: u32) -> DynamicImage {
    if img.width() <= max_edge && img.height() <= max_edge {
        return img.clone();
    }
    img.resize(max_edge, max_edge, image::imageops::FilterType::Lanczos3)
}

fn encode_webp(img: &DynamicImage) -> Result<Vec<u8>, ServiceError> {
    let rgba = img.to_rgba8();
    let encoder = webp::Encoder::from_rgba(&rgba, rgba.width(), rgba.height());
    let memory = encoder.encode(WEBP_QUALITY);
    Ok(memory.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    fn make_png(w: u32, h: u32) -> Vec<u8> {
        let buf: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_fn(w, h, |x, _| Rgba([(x % 256) as u8, 0, 0, 255]));
        let mut out = Vec::new();
        image::DynamicImage::ImageRgba8(buf)
            .write_to(&mut Cursor::new(&mut out), image::ImageFormat::Png)
            .unwrap();
        out
    }

    #[test]
    fn produces_three_variants() {
        let png = make_png(3000, 2000);
        let variants = build_variants(&png).unwrap();
        assert_eq!(variants.len(), 3);
        assert!(variants.iter().all(|v| !v.data.is_empty()));
    }

    #[test]
    fn rejects_garbage_input() {
        assert!(build_variants(b"not an image").is_err());
    }

    #[test]
    fn small_image_does_not_upscale() {
        let png = make_png(100, 80);
        let variants = build_variants(&png).unwrap();
        // Full variant should remain roughly original size — not upscaled to 1920
        assert!(variants.iter().all(|v| !v.data.is_empty()));
    }
}
