use phase_loading::RemoteSource;

use crate::{EvalContext, Result, figma::RemoteMetadata};

pub fn fetch_remote(
    ctx: &EvalContext,
    args: FetchRemoteArgs,
    on_fetch_start: impl FnOnce(),
) -> Result<RemoteMetadata> {
    ctx.figma_repository
        .get_remote(args.remote, ctx.eval_args.refetch, on_fetch_start)
}

pub struct FetchRemoteArgs<'a> {
    pub remote: &'a RemoteSource,
}
