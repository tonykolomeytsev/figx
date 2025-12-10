use crate::parser::ProfileDto;
use crate::{CanBeExtendedBy, ResourceAttrs, ResourceDiagnostics, Result};
use crate::{LoadedFigFile, Profile, RemoteSource, Resource, parser::ResourcesDto};
use lib_label::Label;
use ordermap::OrderMap;
use std::sync::Arc;

pub(crate) fn parse_resources(
    fig_file: &LoadedFigFile,
    resources_dto: ResourcesDto,
    remotes: &OrderMap<String, Arc<RemoteSource>>,
) -> Result<Vec<Resource>> {
    let ResourcesDto(resources) = resources_dto;
    let mut output = Vec::new();
    let resource_location_file = Arc::new(fig_file.fig_file.to_owned());

    for (_, res_dto_list) in resources {
        for (res_id, res_dto) in res_dto_list {
            // Create label for the resource
            let label: Label = Label::from_package_and_name(&fig_file.package, &res_id)
                .expect("validated on parsing stage");
            let profile = match res_dto.override_profile {
                None => res_dto.profile,
                Some(p) => Arc::new(res_dto.profile.extend(&p)),
            };
            let res = Resource {
                attrs: ResourceAttrs {
                    label,
                    remote: parse_remote_by_id(remotes, profile.remote_id())?,
                    node_name: res_dto.node_name,
                    package_dir: fig_file.fig_dir.clone(),
                    diag: ResourceDiagnostics {
                        file: resource_location_file.clone(),
                        definition_span: res_dto.def_span.start..res_dto.def_span.end,
                    },
                },
                profile,
            };
            output.push(res);
        }
    }
    Ok(output)
}

impl CanBeExtendedBy<ProfileDto> for Profile {
    fn extend(&self, another: &ProfileDto) -> Self {
        use Profile::*;
        match (self, another) {
            (Png(domain), ProfileDto::Png(dto)) => Png(domain.extend(dto)),
            (Svg(domain), ProfileDto::Svg(dto)) => Svg(domain.extend(dto)),
            (Pdf(domain), ProfileDto::Pdf(dto)) => Pdf(domain.extend(dto)),
            (Webp(domain), ProfileDto::Webp(dto)) => Webp(domain.extend(dto)),
            (Compose(domain), ProfileDto::Compose(dto)) => Compose(domain.extend(dto)),
            (AndroidWebp(domain), ProfileDto::AndroidWebp(dto)) => AndroidWebp(domain.extend(dto)),
            (AndroidDrawable(domain), ProfileDto::AndroidDrawable(dto)) => {
                AndroidDrawable(domain.extend(dto))
            }
            _ => panic!(
                "Inconsistent internal parser state. Cannot merge dto and domain profiles of different types"
            ),
        }
    }
}

fn parse_remote_by_id(
    remotes: &OrderMap<String, Arc<RemoteSource>>,
    remote_id: &str,
) -> Result<Arc<RemoteSource>> {
    if remote_id.is_empty() {
        let default_remote = remotes
            .first()
            .expect("already validated at parsing phase")
            .1;
        Ok(default_remote.clone())
    } else {
        Ok(remotes
            .get(remote_id)
            .expect("validated at the previous stage")
            .clone())
    }
}
