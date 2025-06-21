use super::{
    GetRemoteImageArgs, get_remote_image,
    materialize::{MaterializeArgs, materialize},
};
use crate::{
    EvalContext, Result, Target,
    actions::{
        convert_svg_to_compose::{ConvertSvgToComposeArgs, convert_svg_to_compose},
        validation::ensure_is_vector_node,
    },
    figma::NodeMetadata,
};
use lib_progress_bar::create_in_progress_item;
use log::{debug, info, warn};
use phase_loading::ComposeProfile;
use std::path::{Path, PathBuf};

pub fn import_compose(ctx: &EvalContext, args: ImportComposeArgs) -> Result<()> {
    let ImportComposeArgs {
        node,
        target,
        profile,
    } = args;
    let node_name = target.figma_name();
    let variant_name = target.id.clone().unwrap_or_default();

    debug!(target: "Import", "compose: {}", target.attrs.label.name);
    let _guard = create_in_progress_item(target.attrs.label.name.as_ref());

    let output_dir = get_output_dir_for_compose_profile(profile, &target.attrs.package_dir);
    let package = get_kotlin_package(&output_dir).unwrap_or_default();

    if let (None, true) = (&profile.package, package.is_empty()) {
        warn!("Kotlin package for {} was not found", output_dir.display());
    }

    ensure_is_vector_node(&node, node_name, &target.attrs.label, false);
    let svg = &get_remote_image(
        ctx,
        GetRemoteImageArgs {
            label: &target.attrs.label,
            remote: &target.attrs.remote,
            node,
            format: "svg",
            scale: 1.0,
            variant_name: &variant_name,
        },
    )?;
    let compose = convert_svg_to_compose(
        ctx,
        ConvertSvgToComposeArgs {
            label: &target.attrs.label,
            variant_name: &variant_name,
            name: target.output_name(),
            package: match profile.package.as_ref() {
                None => &package,
                Some(package) => package,
            },
            kotlin_explicit_api: profile.kotlin_explicit_api,
            extension_target: &profile.extension_target,
            file_suppress_lint: &profile.file_suppress_lint,
            svg,
            color_mappings: &profile.color_mappings,
            preview: &profile.preview,
            composable_get: profile.composable_get,
        },
    )?;

    let variant = target
        .id
        .as_ref()
        .map(|it| format!(" ({it})"))
        .unwrap_or_default();
    let label = target.attrs.label.fitted(50);
    materialize(
        ctx,
        MaterializeArgs {
            output_dir: &output_dir,
            file_name: target.output_name(),
            file_extension: "kt",
            bytes: &compose,
        },
        || info!(target: "Writing", "`{label}`{variant} to file"),
    )?;

    Ok(())
}

pub struct ImportComposeArgs<'a> {
    node: &'a NodeMetadata,
    target: Target<'a>,
    profile: &'a ComposeProfile,
}

impl<'a> ImportComposeArgs<'a> {
    pub fn new(node: &'a NodeMetadata, target: Target<'a>, profile: &'a ComposeProfile) -> Self {
        Self {
            node,
            target,
            profile,
        }
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
