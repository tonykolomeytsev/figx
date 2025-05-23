use log::{debug, info};
use phase_loading::{ResourceAttrs, SvgProfile};

use crate::{EvalContext, Result};

use super::{
    GetRemoteImageArgs, get_remote_image,
    materialize::{MaterializeArgs, materialize},
};

pub fn import_svg(ctx: &EvalContext, args: ImportSvgArgs) -> Result<()> {
    debug!(target: "Import", "svg: {}", args.attrs.label.name);
    let svg = get_remote_image(
        ctx,
        GetRemoteImageArgs {
            label: &args.attrs.label,
            remote: &args.attrs.remote,
            node_name: &args.attrs.node_name,
            format: "svg",
            scale: args.profile.scale,
        },
    )?;
    materialize(
        ctx,
        MaterializeArgs {
            output_dir: &args.attrs.package_dir.join(&args.profile.output_dir),
            file_name: args.attrs.label.name.as_ref(),
            file_extension: "svg",
            bytes: &svg,
        },
        || info!(target: "Writing", "`{}` to file", args.attrs.label.truncated_display(60)),
    )?;
    Ok(())
}

pub struct ImportSvgArgs<'a> {
    attrs: &'a ResourceAttrs,
    profile: &'a SvgProfile,
}

impl<'a> ImportSvgArgs<'a> {
    pub fn new(attrs: &'a ResourceAttrs, profile: &'a SvgProfile) -> Self {
        Self { attrs, profile }
    }
}
