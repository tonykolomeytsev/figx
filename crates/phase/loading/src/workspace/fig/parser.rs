use crate::parser::{ResourcesDto, ResourcesDtoContext};
use crate::workspace::fig::parse_resources;
use crate::{Error, ParseWithContext, Result};
use crate::{LoadedFigFile, Package};
use crate::{Profile, RemoteSource};
use lib_label::LabelPattern;
use log::debug;
use ordermap::OrderMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub(crate) struct FigFileDto(pub ResourcesDto);

impl FigFileDto {
    pub fn from_file(file: &Path, ctx: ResourcesDtoContext<'_>) -> Result<Self> {
        let string = std::fs::read_to_string(file).map_err(Error::FigRead)?;
        Ok(Self::from_str(&string, ctx).map_err(|e| Error::FigParse(e, PathBuf::new()))?)
    }

    pub fn from_str(
        string: &str,
        ctx: ResourcesDtoContext<'_>,
    ) -> std::result::Result<Self, toml_span::DeserError> {
        let resources = ResourcesDto::parse_with_ctx(&mut toml_span::parse(&string)?, ctx)?;
        Ok(FigFileDto(resources))
    }
}

pub(crate) fn parse_fig(
    fig_file: &LoadedFigFile,
    remotes: &OrderMap<String, Arc<RemoteSource>>,
    profiles: &OrderMap<String, Arc<Profile>>,
    pattern: &LabelPattern,
    current_dir: &Path,
) -> Result<Package> {
    debug!("Parsing fig-file {}", fig_file.fig_file.display());
    let fig_dto = FigFileDto::from_file(
        &fig_file.fig_file,
        ResourcesDtoContext {
            declared_remote_ids: &remotes
                .keys()
                .map(|it| it.to_string())
                .collect::<HashSet<_>>(),
            profiles,
        },
    )?;
    let mut resources = parse_resources(&fig_file, fig_dto.0, remotes)?;

    // filter out irrelevant resources
    resources.retain(|res| lib_label::matches(pattern, &res.attrs.label, current_dir));

    Ok(Package {
        label: fig_file.package.clone(),
        resources,
        source_file: fig_file.fig_file.clone(),
    })
}
