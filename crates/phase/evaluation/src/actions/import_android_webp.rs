use lib_progress_bar::create_in_progress_item;
use log::debug;
use log::info;
use phase_loading::AndroidDensity;
use phase_loading::AndroidWebpProfile;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;

use crate::EvalContext;
use crate::Result;
use crate::Target;
use crate::actions::GetRemoteImageArgs;
use crate::actions::convert_png_to_webp::ConvertPngToWebpArgs;
use crate::actions::convert_png_to_webp::convert_png_to_webp;
use crate::actions::get_node::GetNodeArgs;
use crate::actions::get_node::ensure_is_vector_node;
use crate::actions::get_node::get_node;
use crate::actions::get_remote_image;
use crate::actions::materialize::MaterializeArgs;
use crate::actions::materialize::materialize;
use crate::actions::render_svg_to_png::RenderSvgToPngArgs;
use crate::actions::render_svg_to_png::render_svg_to_png;

pub fn import_android_webp(ctx: &EvalContext, args: ImportAndroidWebpArgs) -> Result<()> {
    let ImportAndroidWebpArgs { target, profile } = args;

    debug!(target: "Import", "android-webp: {}", target.attrs.label.name);
    let _guard = create_in_progress_item(target.attrs.label.name.as_ref());

    // region: generating all android variants
    let scales = &profile.scales;
    let themes: &[_] = if let Some(night_variant) = &profile.night {
        let light_variant = &target.attrs.node_name;
        let night_variant = expand_night_variant(light_variant, night_variant.as_ref());
        &[(light_variant.to_owned(), false), (night_variant, true)]
    } else {
        let light_variant = &target.attrs.node_name;
        &[(light_variant.to_owned(), false)]
    };
    let all_variants = cartesian_product(scales, themes);
    // endregion: generating all android variants

    all_variants
        .par_iter()
        .map(|(density, (node_name, is_night))| {
            let factor = scale_factor(density);
            let density_name = density_name(density);
            let variant_name = if !is_night {
                format!("{density_name}")
            } else {
                format!("night-{density_name}")
            };

            let node = get_node(
                ctx,
                GetNodeArgs {
                    node_name: &node_name,
                    remote: &target.attrs.remote,
                    diag: &target.attrs.diag,
                },
            )?;
            let png = if profile.legacy_loader {
                get_remote_image(
                    ctx,
                    GetRemoteImageArgs {
                        label: &target.attrs.label,
                        remote: &target.attrs.remote,
                        node: &node,
                        format: "png",
                        scale: factor,
                        variant_name: &variant_name,
                    },
                )?
            } else {
                ensure_is_vector_node(&node, node_name, &target.attrs.label, true);
                let svg = get_remote_image(
                    ctx,
                    GetRemoteImageArgs {
                        label: &target.attrs.label,
                        remote: &target.attrs.remote,
                        node: &node,
                        format: "svg",
                        scale: 1.0,       // always the same yes
                        variant_name: "", // no variant yes
                    },
                )?;
                render_svg_to_png(
                    ctx,
                    RenderSvgToPngArgs {
                        label: &target.attrs.label,
                        variant_name: &variant_name,
                        svg: &svg,
                        zoom: if factor != 1.0 { Some(factor) } else { None },
                    },
                )?
            };

            let webp = convert_png_to_webp(
                ctx,
                ConvertPngToWebpArgs {
                    quality: *profile.quality,
                    bytes: &png,
                    label: &target.attrs.label,
                    variant_name: &variant_name,
                },
            )?;
            let output_dir = target
                .attrs
                .package_dir
                .join(&profile.android_res_dir)
                .join(&format!("drawable-{variant_name}"));

            materialize(
                ctx,
                MaterializeArgs {
                    output_dir: &output_dir,
                    file_name: target.attrs.label.name.as_ref(),
                    file_extension: "webp",
                    bytes: &webp,
                },
                || {
                    info!(target: "Writing", "`{label}` ({variant}) to file",
                        label = target.attrs.label.fitted(50),
                        variant = &variant_name,
                    )
                },
            )
        })
        .collect::<Result<()>>()?;
    Ok(())
}

pub struct ImportAndroidWebpArgs<'a> {
    target: Target<'a>,
    profile: &'a AndroidWebpProfile,
}

impl<'a> ImportAndroidWebpArgs<'a> {
    pub fn new(target: Target<'a>, profile: &'a AndroidWebpProfile) -> Self {
        Self { target, profile }
    }
}

pub fn scale_factor(d: &AndroidDensity) -> f32 {
    use AndroidDensity::*;
    match d {
        LDPI => 0.75,
        MDPI => 1.0,
        HDPI => 1.5,
        XHDPI => 2.0,
        XXHDPI => 3.0,
        XXXHDPI => 4.0,
    }
}

pub fn density_name(d: &AndroidDensity) -> &str {
    use AndroidDensity::*;
    match d {
        LDPI => "ldpi",
        MDPI => "mdpi",
        HDPI => "hdpi",
        XHDPI => "xhdpi",
        XXHDPI => "xxhdpi",
        XXXHDPI => "xxxhdpi",
    }
}

pub fn cartesian_product<'a, A, B>(list_a: &'a [A], list_b: &'a [B]) -> Vec<(&'a A, &'a B)> {
    list_a
        .iter()
        .flat_map(|a| list_b.iter().map(move |b| (a, b)))
        .collect()
}

pub fn expand_night_variant(light_variant: &str, night_variant: &str) -> String {
    night_variant.replacen("{base}", light_variant, 1)
}
