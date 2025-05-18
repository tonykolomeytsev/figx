#![allow(unused)]

use image::error::{EncodingError, ImageFormatHint};
use image::{DynamicImage, ImageBuffer, ImageError, ImageResult, Rgb, Rgba};
use std::io::Write;
use std::ops::Deref;

pub fn webp_write<W: Write>(img: &DynamicImage, w: &mut W, q: f32) -> ImageResult<()> {
    match img {
        DynamicImage::ImageRgb8(img) => webp_write_rgb(img, w, q),
        DynamicImage::ImageRgba8(img) => webp_write_rgba(img, w, q),
        DynamicImage::ImageLuma8(_) => webp_write_rgb(&img.to_rgb8(), w, q),
        DynamicImage::ImageLumaA8(_) => webp_write_rgba(&img.to_rgba8(), w, q),
        DynamicImage::ImageRgb16(_) => webp_write_rgb(&img.to_rgb8(), w, q),
        DynamicImage::ImageRgba16(_) => webp_write_rgba(&img.to_rgba8(), w, q),
        DynamicImage::ImageLuma16(_) => webp_write_rgb(&img.to_rgb8(), w, q),
        DynamicImage::ImageLumaA16(_) => webp_write_rgba(&img.to_rgba8(), w, q),
        DynamicImage::ImageRgb32F(_) => webp_write_rgb(&img.to_rgb8(), w, q),
        DynamicImage::ImageRgba32F(_) => webp_write_rgba(&img.to_rgba8(), w, q),
        _ => webp_write_rgba(&img.to_rgba8(), w, q),
    }
}

pub fn webp_write_rgba<W: Write, C>(
    img: &ImageBuffer<Rgba<u8>, C>,
    w: &mut W,
    q: f32,
) -> ImageResult<()>
where
    C: Deref<Target = [u8]>,
{
    let buf = crate::encode::WebPEncodeRGBA(&img, img.width(), img.height(), img.width() * 4, q)
        .map_err(|_| EncodingError::new(ImageFormatHint::Unknown, "Webp Format Error".to_string()))
        .map_err(ImageError::Encoding)?;
    w.write_all(&buf)?;
    Ok(())
}

pub fn webp_write_rgb<W: Write, C>(
    img: &ImageBuffer<Rgb<u8>, C>,
    w: &mut W,
    q: f32,
) -> ImageResult<()>
where
    C: Deref<Target = [u8]>,
{
    let buf = crate::encode::WebPEncodeRGB(&img, img.width(), img.height(), img.width() * 3, q)
        .map_err(|_| EncodingError::new(ImageFormatHint::Unknown, "Webp Format Error".to_string()))
        .map_err(ImageError::Encoding)?;
    w.write_all(&buf)?;
    Ok(())
}
