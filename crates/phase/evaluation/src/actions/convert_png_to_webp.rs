use crate::{Error, EvalContext, Result};
use lib_cache::CacheKey;
use lib_label::Label;
use log::info;

const WEBP_TRANSFORM_TAG: u8 = 0x02;

pub fn convert_png_to_webp(ctx: &EvalContext, args: ConvertPngToWebpArgs) -> Result<Vec<u8>> {
    // construct unique cache key
    let cache_key = CacheKey::builder()
        .set_tag(WEBP_TRANSFORM_TAG)
        .write(args.bytes)
        .write_str(&args.quality.to_string())
        .build();

    // return cached value if it exists
    if let Some(webp) = ctx.cache.get_bytes(&cache_key)? {
        return Ok(webp);
    }

    // otherwise, do transform
    info!(target: "Converting", "PNG to WEBP: {}", args.label.truncated_display(50));
    let png = image::load_from_memory_with_format(args.bytes, image::ImageFormat::Png)?;
    let encoder = webp::Encoder::from_image(&png).map_err(|_| Error::WebpCreate)?; // fails if img is not RBG8 or RBGA8
    let webp = if args.quality == 100.0 {
        encoder.encode_lossless()
    } else {
        encoder.encode(args.quality)
    };

    // remember result to cache
    ctx.cache.put_slice(&cache_key, &webp)?;
    Ok(webp.to_vec())
}

pub struct ConvertPngToWebpArgs<'a> {
    pub quality: f32,
    pub bytes: &'a [u8],
    pub label: &'a Label,
}
