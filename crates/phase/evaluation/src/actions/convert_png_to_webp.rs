use crate::{Error, EvalContext, Result};
use log::debug;

pub fn convert_png_to_webp(_ctx: &EvalContext, args: ConvertPngToWebpArgs) -> Result<Vec<u8>> {
    debug!("transforming: png to webp (q={})", args.quality);

    let png = image::load_from_memory_with_format(&args.bytes, image::ImageFormat::Png)?;
    let encoder = webp::Encoder::from_image(&png).map_err(|_| Error::WebpCreate)?; // fails if img is not RBG8 or RBGA8
    let webp = if args.quality == 100.0 {
        encoder.encode_lossless()
    } else {
        encoder.encode(args.quality)
    };
    Ok(webp.to_vec())
}

pub struct ConvertPngToWebpArgs<'a> {
    pub quality: f32,
    pub bytes: &'a [u8],
}
