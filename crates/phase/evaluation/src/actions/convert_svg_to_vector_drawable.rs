use lib_cache::CacheKey;
use lib_label::Label;
use lib_svg2drawable::SvgToDrawableOptions;
use log::info;

use crate::{EvalContext, Result};

const AVD_TRANSFORM_TAG: u8 = 0x09;

pub fn convert_svg_to_vector_drawable(
    ctx: &EvalContext,
    args: ConvertSvgToVectorDrawableArgs,
) -> Result<Vec<u8>> {
    // construct unique cache key
    let cache_key = CacheKey::builder()
        .set_tag(AVD_TRANSFORM_TAG)
        .write(args.svg)
        .build();

    // return cached value if it exists
    // if let Some(compose) = ctx.cache.get_bytes(&cache_key)? {
    //     return Ok(compose);
    // }

    // otherwise, do transform
    info!(target: "Converting", "SVG to Android Drawable: `{label}`{variant}",
        label = args.label.fitted(40),
        variant = if args.variant_name.is_empty() {
            String::new()
        } else {
            format!(" ({})", args.variant_name)
        }
    );
    let xml = lib_svg2drawable::transform_svg_to_drawable(
        args.svg,
        SvgToDrawableOptions {
            xml_declaration: false,
            auto_mirrored: args.auto_mirrored,
        },
    )
    .map_err(|err| {
        crate::Error::ConversionError(format!(
            "unable to convert SVG to Drawable XML ({}): {err}",
            args.label,
        ))
    })?;

    // remember result to cache
    ctx.cache.put_bytes(&cache_key, &xml)?;
    Ok(xml)
}

pub struct ConvertSvgToVectorDrawableArgs<'a> {
    pub label: &'a Label,
    pub variant_name: &'a str,
    pub auto_mirrored: bool,
    pub svg: &'a [u8],
}
