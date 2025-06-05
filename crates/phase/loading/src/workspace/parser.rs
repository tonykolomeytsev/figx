use super::fig::parse_fig;
use crate::parser::WorkspaceDto;
use crate::workspace::profiles::parse_profiles;
use crate::workspace::remotes::parse_remotes;
use crate::{Error, RemoteSource};
use crate::{InvocationContext, Workspace};
use crate::{Package, Profile};
use crate::{ParseWithContext, Result};
use lib_label::LabelPattern;
use log::debug;
use ordermap::OrderMap;
use std::path::Path;
use std::sync::Arc;

impl WorkspaceDto {
    pub fn from_file(file: &Path) -> Result<Self> {
        let string = std::fs::read_to_string(file).map_err(Error::WorkspaceRead)?;
        Self::from_str(&string).map_err(|e| Error::WorkspaceParse(e, file.to_owned()))
    }

    pub fn from_str(string: &str) -> std::result::Result<Self, toml_span::DeserError> {
        WorkspaceDto::parse_with_ctx(&mut toml_span::parse(&string)?, ())
    }
}

pub(crate) fn parse_workspace(
    context: InvocationContext,
    pattern: LabelPattern,
) -> Result<Workspace> {
    debug!("Parsing workspace config...");
    let ws_dto = WorkspaceDto::from_file(&context.workspace_file)?;
    let remotes = parse_remotes(ws_dto.remotes)?;
    let profiles = parse_profiles(ws_dto.profiles)?;
    let packages = parse_packages(&context, pattern, &remotes, &profiles)?;

    Ok(Workspace {
        context,
        remotes: remotes.into_values().collect(),
        profiles: profiles.into_values().collect(),
        packages,
    })
}

fn parse_packages(
    context: &InvocationContext,
    pattern: LabelPattern,
    remotes: &OrderMap<String, Arc<RemoteSource>>,
    profiles: &OrderMap<String, Arc<Profile>>,
) -> Result<Vec<Package>> {
    context
        .fig_files
        .iter()
        // do not load irrelevant packages
        .filter(|f| lib_label::package_matches(&pattern, &f.package, &context.current_dir))
        .map(|f| {
            parse_fig(f, remotes, profiles, &pattern, &context.current_dir).map_err(|e| match e {
                Error::FigParse(e, _) => Error::FigParse(e, f.fig_file.to_owned()),
                e => e,
            })
        })
        .collect()
}
