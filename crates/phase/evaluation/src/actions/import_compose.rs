use super::{
    GetRemoteImageArgs, get_remote_image,
    materialize::{MaterializeArgs, materialize},
};
use crate::{
    EvalContext, Result,
    actions::convert_svg_to_compose::{ConvertSvgToComposeArgs, convert_svg_to_compose},
};
use lib_progress_bar::create_in_progress_item;
use log::{debug, info, warn};
use phase_loading::{ComposeProfile, ResourceAttrs, ResourceVariants};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::path::{Path, PathBuf};

struct ResourceVariant {
    pub res_name: String,
    pub node_name: String,
}

pub fn import_compose(ctx: &EvalContext, args: ImportComposeArgs) -> Result<()> {
    debug!(target: "Import", "compose: {}", args.attrs.label.name);
    let _guard = create_in_progress_item(args.attrs.label.name.as_ref());

    let output_dir = get_output_dir_for_compose_profile(args.profile, &args.attrs.package_dir);
    let package = get_kotlin_package(&output_dir).unwrap_or_default();

    if let (None, true) = (&args.profile.package, package.is_empty()) {
        warn!("Kotlin package for {} was not found", output_dir.display());
    }

    let base_variant = ResourceVariant {
        res_name: args.attrs.label.name.to_string(),
        node_name: args.attrs.node_name.to_owned(),
    };

    // region: generate variants
    let variants = match &args.profile.variants {
        Some(ResourceVariants {
            naming,
            list: Some(list),
        }) => list
            .iter()
            .map(|variant| {
                let naming = &naming;
                let res_name = naming
                    .local_name
                    .replace("{base}", &base_variant.res_name)
                    .replace("{variant}", &variant);
                let node_name = naming
                    .figma_name
                    .replace("{base}", &base_variant.node_name)
                    .replace("{variant}", &variant);

                ResourceVariant {
                    res_name,
                    node_name,
                }
            })
            .collect::<Vec<_>>(),
        _ => vec![base_variant],
    };
    // endregion: generate variants

    variants
        .par_iter()
        .map(|variant| {
            let svg = &get_remote_image(
                ctx,
                GetRemoteImageArgs {
                    label: &args.attrs.label,
                    remote: &args.attrs.remote,
                    node_name: &variant.node_name,
                    format: "svg",
                    scale: args.profile.scale,
                },
            )?;

            materialize(
                ctx,
                MaterializeArgs {
                    output_dir: &output_dir,
                    file_name: &variant.res_name,
                    file_extension: "kt",
                    bytes: &convert_svg_to_compose(
                        ctx,
                        ConvertSvgToComposeArgs {
                            name: &variant.res_name,
                            package: match args.profile.package.as_ref() {
                                None => &package,
                                Some(package) => package,
                            },
                            kotlin_explicit_api: args.profile.kotlin_explicit_api,
                            extension_target: &args.profile.extension_target,
                            file_suppress_lint: &args.profile.file_suppress_lint,
                            svg,
                            color_mappings: &args.profile.color_mappings,
                            preview: &args.profile.preview,
                            composable_get: args.profile.composable_get,
                        },
                    )?,
                },
                || info!(target: "Writing", "`{}` to file", args.attrs.label.truncated_display(60)),
            )
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(())
}

pub struct ImportComposeArgs<'a> {
    attrs: &'a ResourceAttrs,
    profile: &'a ComposeProfile,
}

impl<'a> ImportComposeArgs<'a> {
    pub fn new(attrs: &'a ResourceAttrs, profile: &'a ComposeProfile) -> Self {
        Self { attrs, profile }
    }
}

pub fn get_output_dir_for_compose_profile(p: &ComposeProfile, abs_package_dir: &Path) -> PathBuf {
    let kt_src_dir = &p.src_dir;
    let kt_package = match &p.package {
        Some(package) => package.replace('.', "/"),
        None => String::new(),
    };
    // {abs_package_dir}/{kt_src_dir}/{pkg_dir}
    abs_package_dir.join(kt_src_dir).join(kt_package)
}

pub fn get_kotlin_package(output_dir: &Path) -> Option<String> {
    let mut current_dir = output_dir.to_path_buf();

    // Step 2: Traverse upwards to find source root
    while current_dir.pop() {
        // Moves to parent directory
        if is_source_root(&current_dir) {
            debug!("Found package from sources root: {}", current_dir.display());
            // Reconstruct original path relative to source root
            let rel_path = output_dir
                .strip_prefix(&current_dir)
                .expect("current_dir is always subpath of output_dir");
            let package = dir_to_package(rel_path);
            return Some(package);
        }
    }
    None
}

/// Check if a directory is a known Kotlin source root
fn is_source_root(dir: &Path) -> bool {
    dir.ends_with("src/main/kotlin")
        || dir.ends_with("src/debug/kotlin")
        || dir.ends_with("src/release/kotlin")
        || dir.ends_with("src/main/java")
        || dir.ends_with("src/commonMain/kotlin")
        || dir.ends_with("src/jvmMain/kotlin")
        || dir.ends_with("src/jsMain/kotlin")
        || dir.ends_with("src/iosArm64Main/kotlin")
        || dir.ends_with("src/macosX64Main/kotlin")
}

/// Convert directory path to package name (e.g., "com/example" -> "com.example")
fn dir_to_package(dir: &Path) -> String {
    dir.to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, ".")
}
