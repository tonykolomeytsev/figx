use crate::{
    AndroidWebpProfile, ComposeProfile, Error, LoadedFigFile, PdfProfile, PngProfile, Profile,
    RemoteSource, Resource, ResourceAttrs, Result, SvgProfile, WebpProfile,
    workspace::{
        AndroidWebpProfileDto, ComposeProfileDto, PdfProfileDto, SvgProfileDto, WebpProfileDto,
        profiles::{CanBeExtendedBy, PngProfileDto},
    },
};
use lib_label::{Label, ResourceName};
use ordermap::OrderMap;
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf, str::FromStr, sync::Arc};
use toml::Table;

#[derive(Deserialize, Default)]
pub(super) struct ResourcesDto(HashMap<String, Table>);

#[derive(Deserialize)]
#[serde(untagged)]
enum ResDefinition<T> {
    Short(String),
    Long {
        name: String,
        #[serde(flatten)]
        profile: T,
    },
}

pub(super) fn parse_resources(
    fig_file: &LoadedFigFile,
    resources_dto: ResourcesDto,
    remotes: &OrderMap<String, Arc<RemoteSource>>,
    profiles: &OrderMap<String, Arc<Profile>>,
) -> Result<Vec<Resource>> {
    let ResourcesDto(sections) = resources_dto;
    let mut resources = Vec::new();
    for (profile_name, definitions) in sections {
        match profiles.get(&profile_name) {
            Some(profile) => {
                parse_definitions(&mut resources, profile, definitions, fig_file, remotes)?
            }
            None => {
                return Err(Error::FigInvalidProfileName(profile_name));
            }
        }
    }
    Ok(resources)
}

fn parse_definitions(
    resources: &mut Vec<Resource>,
    profile: &Arc<Profile>,
    definitions: Table,
    fig_file: &LoadedFigFile,
    remotes: &OrderMap<String, Arc<RemoteSource>>,
) -> Result<()> {
    match profile.as_ref() {
        Profile::Png(typed_profile) => parse_png_definitions(
            resources,
            profile,
            typed_profile,
            definitions,
            fig_file,
            remotes,
        )?,
        Profile::Svg(typed_profile) => parse_svg_definitions(
            resources,
            profile,
            typed_profile,
            definitions,
            fig_file,
            remotes,
        )?,
        Profile::Pdf(typed_profile) => parse_pdf_definitions(
            resources,
            profile,
            typed_profile,
            definitions,
            fig_file,
            remotes,
        )?,
        Profile::Webp(typed_profile) => parse_webp_definitions(
            resources,
            profile,
            typed_profile,
            definitions,
            fig_file,
            remotes,
        )?,
        Profile::Compose(typed_profile) => parse_compose_definitions(
            resources,
            profile,
            typed_profile,
            definitions,
            fig_file,
            remotes,
        )?,
        Profile::AndroidWebp(typed_profile) => parse_android_webp_definitions(
            resources,
            profile,
            typed_profile,
            definitions,
            fig_file,
            remotes,
        )?,
    }
    Ok(())
}

macro_rules! definitions_parser {
    ($name:ident => ($domain_type:path, $dto_type:path, $enum_type:path)) => {
        fn $name(
            resources: &mut Vec<Resource>,
            profile: &Arc<Profile>,
            typed_profile: &$domain_type,
            definitions: Table,
            fig_file: &LoadedFigFile,
            remotes: &OrderMap<String, Arc<RemoteSource>>,
        ) -> Result<()> {
            // iterate over each resource definition in [<profile_name>] dictionary
            for (key, def) in definitions {
                // Parse definition (short 'id = "name"' or long 'id = { ... }')
                let def: ResDefinition<$dto_type> = def.try_into().map_err(Error::FigParse)?;
                // Parse local resource name (id)
                let resource_name =
                    ResourceName::from_str(&key).map_err(Error::FigInvalidResourceName)?;
                // Create label for the resource
                let label: Label = (fig_file.package.clone(), resource_name).into();

                let resource = match def {
                    // If current resource definition is short, parsing is easy
                    // just create resource with default attributes and profile
                    ResDefinition::Short(name) => Resource {
                        attrs: ResourceAttrs {
                            label,
                            remote: parse_remote_by_id(remotes, &typed_profile.remote_id)?,
                            node_name: name,
                            package_dir: fig_file.fig_dir.to_path_buf(),
                        },
                        profile: profile.clone(),
                    },
                    // If current resource definition is long, user could override some profile parameters
                    // so we should merge user-profile with global profile first, then
                    // pass new profile to the resource attributes
                    ResDefinition::Long { name, profile } => {
                        let merged_profile = typed_profile.extend(profile);
                        Resource {
                            attrs: ResourceAttrs {
                                label,
                                remote: parse_remote_by_id(remotes, &merged_profile.remote_id)?,
                                node_name: name,
                                package_dir: fig_file.fig_dir.to_path_buf(),
                            },
                            profile: Arc::new($enum_type(merged_profile)),
                        }
                    }
                };
                resources.push(resource);
            }
            Ok(())
        }
    };
}

definitions_parser!(parse_png_definitions => (PngProfile, PngProfileDto, Profile::Png));
definitions_parser!(parse_svg_definitions => (SvgProfile, SvgProfileDto, Profile::Svg));
definitions_parser!(parse_pdf_definitions => (PdfProfile, PdfProfileDto, Profile::Pdf));
definitions_parser!(parse_webp_definitions => (WebpProfile, WebpProfileDto, Profile::Webp));
definitions_parser!(parse_compose_definitions => (ComposeProfile, ComposeProfileDto, Profile::Compose));
definitions_parser!(parse_android_webp_definitions => (AndroidWebpProfile, AndroidWebpProfileDto, Profile::AndroidWebp));

fn parse_remote_by_id(
    remotes: &OrderMap<String, Arc<RemoteSource>>,
    remote_id: &str,
) -> Result<Arc<RemoteSource>> {
    if remote_id.is_empty() {
        let default_remote = remotes
            .first()
            .ok_or_else(|| Error::WorkspaceNoRemotes(PathBuf::new()))?
            .1;
        Ok(default_remote.clone())
    } else {
        remotes
            .get(remote_id)
            .ok_or_else(|| Error::FigInvalidRemoteName(remote_id.to_string()))
            .cloned()
    }
}
