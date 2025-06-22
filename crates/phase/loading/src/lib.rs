use lib_label::LabelPattern;
use lib_label::Package as PackageLabel;
use log::debug;
use std::path::Path;
use toml_span::Value;
use util::{FileWithParentDir, find_file_in_ancestors, find_files_in_child_dirs};
use workspace::parse_workspace;

mod api;
mod error;
mod parser;
mod util;
mod workspace;

pub use api::*;
pub use error::*;

static WORKSPACE_FILE_NAME: &str = ".figtree.toml";
static RESOURCES_FILE_NAME: &str = ".fig.toml";
static CACHE_DIR: &str = ".figx-out/caches";

pub fn load_invocation_context() -> Result<InvocationContext> {
    debug!("Restoring invocation context...");
    let working_dir = std::env::current_dir().map_err(|_| Error::InitInaccessibleCurrentWorkDir)?;
    // Looking for workspace marker in this dir and it's ancestors
    let ws_file = find_workspace_file(&working_dir)?;
    // Looking recursively for fig files in workspace directory and children directories
    // FIXME: Cannot start traversing from the current directory because, if the user queries
    //        an absolute package like `//path/to:resource`, we need to know about packages
    //        other than our own.
    let fig_files = find_fig_files(&ws_file.parent_dir)?;

    let current_dir = working_dir
        .strip_prefix(&ws_file.parent_dir)
        .expect("`parent_dir` is ALWAYS subdir of `ws_file.parent_dir`")
        .to_path_buf();

    let mut loaded_fig_files: Vec<LoadedFigFile> = Vec::new();
    let mut current_package = None;
    for FileWithParentDir { file, parent_dir } in fig_files {
        let package = PackageLabel::with_path(
            parent_dir
                .strip_prefix(&ws_file.parent_dir)
                .expect("`f.parent_dir` is ALWAYS subdir of `ws_file.parent_dir`"),
        )
        .map_err(Error::FigInvalidPackage)?;

        if *package == current_dir {
            current_package = Some(package.clone())
        }

        loaded_fig_files.push(LoadedFigFile {
            package,
            fig_dir: parent_dir,
            fig_file: file,
        });
    }

    Ok(InvocationContext {
        workspace_dir: ws_file.parent_dir.clone(),
        workspace_file: ws_file.file,
        current_dir,
        current_package,
        fig_files: loaded_fig_files,
        cache_dir: ws_file.parent_dir.join(CACHE_DIR),
    })
}

pub fn load_workspace(
    pattern: LabelPattern,
    ignore_missing_access_token: bool,
) -> Result<Workspace> {
    let invocation_ctx = load_invocation_context()?;
    debug!("Loading workspace...");
    let ws_file = invocation_ctx.workspace_file.clone();
    parse_workspace(invocation_ctx, pattern, ignore_missing_access_token).map_err(|e| match e {
        Error::WorkspaceParse(e, _) => Error::WorkspaceParse(e, ws_file),
        Error::WorkspaceRemoteNoAccessToken(id, _, span) => {
            Error::WorkspaceRemoteNoAccessToken(id, ws_file, span)
        }
        e => e,
    })
}

fn find_workspace_file(start_dir: &Path) -> Result<FileWithParentDir> {
    debug!("Seeking workspace file...");
    find_file_in_ancestors(WORKSPACE_FILE_NAME, start_dir).ok_or(Error::InitNotInWorkspace)
}

fn find_fig_files(start_dir: &Path) -> Result<Vec<FileWithParentDir>> {
    debug!("Seeking fig files...");
    find_files_in_child_dirs(RESOURCES_FILE_NAME, start_dir)
        .map_err(|e| Error::FigTraversing(e.to_string()))
}

pub(crate) trait ParseWithContext<'de>
where
    Self: Sized,
{
    type Context;

    fn parse_with_ctx(
        value: &mut Value<'de>,
        ctx: Self::Context,
    ) -> std::result::Result<Self, toml_span::DeserError>;
}

pub(crate) trait CanBeExtendedBy<T> {
    fn extend(&self, another: &T) -> Self;
}
