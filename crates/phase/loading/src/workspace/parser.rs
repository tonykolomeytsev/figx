use super::fig::parse_fig;
use super::{ProfilesDto, RemotesDto};
use crate::Result;
use crate::workspace::profiles::parse_profiles;
use crate::workspace::remotes::parse_remotes;
use crate::{Error, RemoteSource};
use crate::{InvocationContext, Workspace};
use crate::{Package, Profile};
use lib_label::LabelPattern;
use ordermap::OrderMap;
use serde::Deserialize;
use std::path::Path;
use std::sync::Arc;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WorkspaceDto {
    #[serde(default = "Default::default")]
    pub remotes: RemotesDto,
    #[serde(default = "Default::default")]
    pub profiles: ProfilesDto,
}

impl WorkspaceDto {
    pub fn from_file(file: &Path) -> Result<Self> {
        let string = std::fs::read_to_string(file).map_err(Error::WorkspaceRead)?;
        Self::from_str(&string).map_err(|e| Error::WorkspaceParse(e, file.to_owned()))
    }

    pub fn from_str(string: &str) -> std::result::Result<Self, toml::de::Error> {
        toml::from_str::<WorkspaceDto>(string)
    }
}

pub(crate) fn parse_workspace(
    context: InvocationContext,
    pattern: LabelPattern,
) -> Result<Workspace> {
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
        .map(|f| parse_fig(f, remotes, profiles, &pattern, &context.current_dir))
        .collect()
}
