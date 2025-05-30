use crate::EvalContext;
use crate::Result;
use lib_cache::CacheKey;
use lib_svg2compose::SvgToComposeOptions;
use log::info;
use phase_loading::ColorMapping;
use phase_loading::ComposePreview;

const COMPOSE_TRANSFORM_TAG: u8 = 0x03;

pub fn convert_svg_to_compose(ctx: &EvalContext, args: ConvertSvgToComposeArgs) -> Result<Vec<u8>> {
    // construct unique cache key
    let mut cache_key = CacheKey::builder()
        .set_tag(COMPOSE_TRANSFORM_TAG)
        .write(args.svg)
        .write_str(args.package)
        .write_bool(args.kotlin_explicit_api)
        .write_str(args.extension_target.as_deref().unwrap_or_default())
        .write_str(&args.file_suppress_lint.join(",").to_string());

    for mapping in args.color_mappings {
        cache_key = cache_key.write_str(&mapping.from).write_str(&mapping.to)
    }

    if let Some(preview) = args.preview {
        cache_key = cache_key
            .write_str(&preview.imports.join(","))
            .write_str(&preview.code)
    }

    let cache_key = cache_key.build();

    // return cached value if it exists
    if let Some(compose) = ctx.cache.get_bytes(&cache_key)? {
        return Ok(compose);
    }

    // otherwise, do transform
    info!(target: "Converting", "SVG to Compose: {}", args.name);
    let compose = lib_svg2compose::transform_svg_to_compose(
        args.svg,
        SvgToComposeOptions {
            image_name: args.name.to_owned(),
            package: args.package.to_owned(),
            kotlin_explicit_api: args.kotlin_explicit_api,
            extension_target: args.extension_target.to_owned(),
            file_suppress_lint: args.file_suppress_lint.to_owned(),
            color_mappings: args
                .color_mappings
                .iter()
                .map(|domain| lib_svg2compose::ColorMapping {
                    from: domain.from.to_owned(),
                    to: domain.to.to_owned(),
                    imports: domain.imports.to_owned(),
                })
                .collect(),
            preview: args
                .preview
                .as_ref()
                .map(|domain| lib_svg2compose::ComposePreview {
                    imports: domain.imports.to_owned(),
                    code: domain.code.to_owned(),
                }),
            composable_get: args.composable_get,
        },
    )?;

    // remember result to cache
    ctx.cache.put_slice(&cache_key, &compose)?;
    Ok(compose)
}

pub struct ConvertSvgToComposeArgs<'a> {
    pub name: &'a str,
    pub package: &'a str,
    pub kotlin_explicit_api: bool,
    pub extension_target: &'a Option<String>,
    pub file_suppress_lint: &'a [String],
    pub color_mappings: &'a [ColorMapping],
    pub preview: &'a Option<ComposePreview>,
    pub svg: &'a [u8],
    pub composable_get: bool,
}
