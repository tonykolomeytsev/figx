use log::debug;
use log::info;
use phase_loading::AndroidDensity;
use phase_loading::AndroidWebpProfile;
use phase_loading::ResourceAttrs;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;

use crate::EvalContext;
use crate::Result;
use crate::actions::GetRemoteImageArgs;
use crate::actions::convert_png_to_webp::ConvertPngToWebpArgs;
use crate::actions::convert_png_to_webp::convert_png_to_webp;
use crate::actions::get_remote_image;
use crate::actions::materialize::MaterializeArgs;
use crate::actions::materialize::materialize;

pub fn import_android_webp(ctx: &EvalContext, args: ImportAndroidWebpArgs) -> Result<()> {
    debug!(
        "importing android-webp: {} ({})",
        args.attrs.label.name,
        args.profile
            .scales
            .iter()
            .map(density_name)
            .collect::<Vec<_>>()
            .join(", ")
    );

    args.profile
        .scales
        .par_iter()
        .map(|density| {
            let factor = scale_factor(density);
            let density_name = density_name(density);

            let png = get_remote_image(
                ctx,
                GetRemoteImageArgs {
                    label: &args.attrs.label,
                    remote: &args.attrs.remote,
                    node_name: &args.attrs.node_name,
                    format: "png",
                    scale: factor,
                },
            )?;
            let webp = convert_png_to_webp(
                ctx,
                ConvertPngToWebpArgs {
                    quality: args.profile.quality,
                    bytes: &png,
                },
            )?;
            drop(png);
            let output_dir = args
                .attrs
                .package_dir
                .join(&args.profile.android_res_dir)
                .join(&format!("drawable-{density_name}"));

            materialize(
                ctx,
                MaterializeArgs {
                    output_dir: &output_dir,
                    file_name: args.attrs.label.name.as_ref(),
                    file_extension: "webp",
                    bytes: &webp,
                },
                || info!(target: "Writing", "`{}` ({density_name}) to file", args.attrs.label.truncated_display(50)),
            )
        })
        .collect::<Result<()>>()?;
    Ok(())
}

pub struct ImportAndroidWebpArgs<'a> {
    attrs: &'a ResourceAttrs,
    profile: &'a AndroidWebpProfile,
}

impl<'a> ImportAndroidWebpArgs<'a> {
    pub fn new(attrs: &'a ResourceAttrs, profile: &'a AndroidWebpProfile) -> Self {
        Self { attrs, profile }
    }
}

pub fn scale_factor(d: &AndroidDensity) -> f32 {
    use AndroidDensity::*;
    let density = match d {
        LDPI => 0.75,
        MDPI => 1.0,
        HDPI => 1.5,
        XHDPI => 2.0,
        XXHDPI => 3.0,
        XXXHDPI => 4.0,
    };
    density
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
