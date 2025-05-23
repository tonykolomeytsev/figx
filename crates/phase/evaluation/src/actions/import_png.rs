use log::{debug, info};
use phase_loading::{PngProfile, ResourceAttrs};

use crate::{EvalContext, Result};

use super::{
    GetRemoteImageArgs, get_remote_image,
    materialize::{MaterializeArgs, materialize},
};

pub fn import_png(ctx: &EvalContext, args: ImportPngArgs) -> Result<()> {
    debug!(target: "Import", "png: {}", args.attrs.label.name);
    let png = &get_remote_image(
        ctx,
        GetRemoteImageArgs {
            label: &args.attrs.label,
            remote: &args.attrs.remote,
            node_name: &args.attrs.node_name,
            format: "png",
            scale: args.profile.scale,
        },
    )?;
    materialize(
        ctx,
        MaterializeArgs {
            output_dir: &args.attrs.package_dir.join(&args.profile.output_dir),
            file_name: args.attrs.label.name.as_ref(),
            file_extension: "png",
            bytes: &png,
        },
        || info!(target: "Writing", "`{}` to file", args.attrs.label.truncated_display(60)),
    )?;
    Ok(())
}

pub struct ImportPngArgs<'a> {
    attrs: &'a ResourceAttrs,
    profile: &'a PngProfile,
}

impl<'a> ImportPngArgs<'a> {
    pub fn new(attrs: &'a ResourceAttrs, profile: &'a PngProfile) -> Self {
        Self { attrs, profile }
    }
}
