use crate::{Error, EvalContext, Result};
use lib_cache::CacheKey;
use lib_label::Label;
use log::debug;
use resvg::usvg::Transform;
use resvg::usvg::Tree;

const RESVG_TRANSFORM_TAG: u8 = 0x04;

pub fn render_svg_to_png(ctx: &EvalContext, args: RenderSvgToPngArgs) -> Result<Vec<u8>> {
    // construct unique cache key
    let cache_key = CacheKey::builder()
        .set_tag(RESVG_TRANSFORM_TAG)
        .write(args.svg)
        .write_str(&args.zoom.unwrap_or(1.0).to_string())
        .build();

    // return cached value if it exists
    if let Some(png) = ctx.cache.get_bytes(&cache_key)? {
        return Ok(png);
    }

    // otherwise, do transform
    debug!(
        target: "Rendering", "PNG: `{label}`{variant}",
        label = args.label.fitted(50),
        variant = if args.variant_name.is_empty() {
            String::new()
        } else {
            format!(" ({})", args.variant_name)
        }
    );
    let tree = Tree::from_data(args.svg, &Default::default()).map_err(|e| {
        Error::RenderSvg(format!(
            "invalid svg `{}` {}: {e}",
            args.label, args.variant_name
        ))
    })?;
    let png = render_svg(&tree, args.zoom)
        .map_err(|e| {
            Error::RenderSvg(format!(
                "cannot render svg `{}` {}: {e}",
                args.label, args.variant_name
            ))
        })?
        .encode_png()
        .map_err(|e| {
            Error::RenderSvg(format!(
                "cannot encode rendered svg to png `{}` {}: {e}",
                args.label, args.variant_name
            ))
        })?;

    // remember result to cache
    ctx.cache.put_bytes(&cache_key, &png)?;
    Ok(png.to_vec())
}

fn render_svg(
    tree: &Tree,
    zoom: Option<f32>,
) -> std::result::Result<resvg::tiny_skia::Pixmap, String> {
    let img = {
        let size = match zoom {
            None => tree.size().to_int_size(),
            Some(zoom) => tree
                .size()
                .to_int_size()
                .scale_by(zoom)
                .expect("valid zoom factor"),
        };
        let mut pixmap =
            resvg::tiny_skia::Pixmap::new(size.width(), size.height()).expect("valid svg size");
        let ts = match zoom {
            None => Transform::default(),
            Some(zoom) => Transform::from_scale(zoom, zoom),
        };
        resvg::render(tree, ts, &mut pixmap.as_mut());
        pixmap
    };
    Ok(img)
}

pub struct RenderSvgToPngArgs<'a> {
    pub label: &'a Label,
    pub variant_name: &'a str,
    pub svg: &'a [u8],
    pub zoom: Option<f32>,
}
