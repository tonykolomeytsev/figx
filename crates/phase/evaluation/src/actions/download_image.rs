use log::debug;
use phase_loading::RemoteSource;

use crate::{EvalContext, Result};

pub fn download_image(ctx: &EvalContext, args: DownloadImageArgs) -> Result<Vec<u8>> {
    debug!("downloading: {}", args.url);
    ctx.figma_repository.download(args.remote, args.url)
}

pub struct DownloadImageArgs<'a> {
    pub remote: &'a RemoteSource,
    pub url: &'a str,
}
