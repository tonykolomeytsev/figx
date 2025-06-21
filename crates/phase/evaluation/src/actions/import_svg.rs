use super::{GetRemoteImageArgs, get_remote_image};
use crate::{
    EvalContext, Result, Target,
    actions::{
        materialize::{MaterializeArgs, materialize},
        validation::ensure_is_vector_node,
    },
    figma::NodeMetadata,
};
use lib_progress_bar::create_in_progress_item;
use log::{debug, info};
use phase_loading::SvgProfile;

pub fn import_svg(ctx: &EvalContext, args: ImportSvgArgs) -> Result<()> {
    let ImportSvgArgs {
        node,
        target,
        profile,
    } = args;
    let node_name = target.figma_name();
    let variant_name = target.id.clone().unwrap_or_default();

    debug!(target: "Import", "svg: {}", target.attrs.label.name);
    let _guard = create_in_progress_item(target.attrs.label.name.as_ref());

    ensure_is_vector_node(&node, node_name, &target.attrs.label, false);
    let svg = get_remote_image(
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
    if ctx.eval_args.fetch {
        return Ok(());
    }

    let variant = target
        .id
        .as_ref()
        .map(|it| format!(" ({it})"))
        .unwrap_or_default();
    let label = target.attrs.label.fitted(50);
    materialize(
        ctx,
        MaterializeArgs {
            output_dir: &target.attrs.package_dir.join(&profile.output_dir),
            file_name: target.output_name(),
            file_extension: "svg",
            bytes: &svg,
        },
        || info!(target: "Writing", "`{label}`{variant} to file"),
    )?;

    Ok(())
}

pub struct ImportSvgArgs<'a> {
    node: &'a NodeMetadata,
    target: Target<'a>,
    profile: &'a SvgProfile,
}

impl<'a> ImportSvgArgs<'a> {
    pub fn new(node: &'a NodeMetadata, target: Target<'a>, profile: &'a SvgProfile) -> Self {
        Self {
            node,
            target,
            profile,
        }
    }
}
