use crate::{EvalContext, Result};
use phase_loading::RemoteSource;

pub fn download_image(ctx: &EvalContext, args: DownloadImageArgs) -> Result<Vec<u8>> {
    ctx.figma_repository.download(args.remote, args.url)
}

pub struct DownloadImageArgs<'a> {
    pub remote: &'a RemoteSource,
    pub url: &'a str,
}
