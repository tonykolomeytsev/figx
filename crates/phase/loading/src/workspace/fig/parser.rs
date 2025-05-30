use super::{ResourcesDto, parse_resources};
use crate::{Error, Result};
use crate::{LoadedFigFile, Package};
use crate::{Profile, RemoteSource};
use lib_label::LabelPattern;
use ordermap::OrderMap;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Deserialize)]
struct FigFileDto {
    #[serde(default = "Default::default")]
    #[serde(flatten)]
    res_definitions: ResourcesDto,
}

impl FigFileDto {
    pub fn from_file(file: &Path) -> Result<Self> {
        let string = std::fs::read_to_string(file).map_err(Error::FigRead)?;
        Self::from_str(&string)
    }

    pub fn from_str(string: &str) -> Result<Self> {
        toml::from_str::<FigFileDto>(string).map_err(|e| Error::FigParse(e, PathBuf::new()))
    }
}

pub(crate) fn parse_fig(
    fig_file: &LoadedFigFile,
    remotes: &OrderMap<String, Arc<RemoteSource>>,
    profiles: &OrderMap<String, Arc<Profile>>,
    pattern: &LabelPattern,
    current_dir: &Path,
) -> Result<Package> {
    let fig_dto = FigFileDto::from_file(&fig_file.fig_file)?;
    let mut resources = parse_resources(&fig_file, fig_dto.res_definitions, remotes, profiles)?;

    // filter out irrelevant resources
    resources.retain(|res| lib_label::matches(pattern, &res.attrs.label, current_dir));

    Ok(Package {
        label: fig_file.package.clone(),
        resources,
        source_file: fig_file.fig_file.clone(),
    })
}
