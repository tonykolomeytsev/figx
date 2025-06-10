use super::{
    AndroidWebpProfileDtoContext, ComposeProfileDtoContext, PdfProfileDtoContext,
    PngProfileDtoContext, ProfileDto, SvgProfileDtoContext, WebpProfileDtoContext,
};
use crate::Profile;
use ordermap::OrderMap;
use std::{collections::HashSet, sync::Arc};

#[derive(Default)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub(crate) struct ResourcesDto(pub OrderMap<String, OrderMap<String, ResourceDto>>);

pub(crate) struct ResourcesDtoContext<'de> {
    pub declared_remote_ids: &'de HashSet<String>,
    pub profiles: &'de OrderMap<String, Arc<Profile>>,
}

#[cfg_attr(test, derive(PartialEq, Debug))]
pub(crate) struct ResourceDto {
    pub node_name: String,
    pub profile: Arc<Profile>,
    pub override_profile: Option<ProfileDto>,
}

#[derive(Clone, Copy)]
pub(crate) struct ResourceDtoContext<'de> {
    pub declared_remote_ids: &'de HashSet<String>,
    pub profile: &'de Arc<Profile>,
}

macro_rules! from_ctx_impl {
    ($from:tt, $to:tt) => {
        impl<'de> From<$from<'de>> for $to<'de> {
            fn from(value: $from<'de>) -> Self {
                Self {
                    declared_remote_ids: value.declared_remote_ids,
                }
            }
        }
    };
}

from_ctx_impl!(ResourceDtoContext, PngProfileDtoContext);
from_ctx_impl!(ResourceDtoContext, SvgProfileDtoContext);
from_ctx_impl!(ResourceDtoContext, PdfProfileDtoContext);
from_ctx_impl!(ResourceDtoContext, WebpProfileDtoContext);
from_ctx_impl!(ResourceDtoContext, ComposeProfileDtoContext);
from_ctx_impl!(ResourceDtoContext, AndroidWebpProfileDtoContext);

mod de {
    use toml_span::{ErrorKind, de_helpers::TableHelper};

    use super::*;
    use crate::{
        ParseWithContext,
        parser::{
            AndroidWebpProfileDto, ComposeProfileDto, PdfProfileDto, PngProfileDto, SvgProfileDto,
            WebpProfileDto,
        },
    };

    impl<'de> ParseWithContext<'de> for ResourcesDto {
        type Context = ResourcesDtoContext<'de>;

        fn parse_with_ctx(
            value: &mut toml_span::Value<'de>,
            ctx: Self::Context,
        ) -> std::result::Result<Self, toml_span::DeserError> {
            let mut th = TableHelper::new(value)?;
            let mut sections = OrderMap::new();

            for (profile_key, resources) in th.table.iter_mut() {
                let profile_name = profile_key.to_string();
                let Some(profile) = ctx.profiles.get(&profile_name) else {
                    let expected = ctx
                        .profiles
                        .keys()
                        .map(|it| format!("`{it}`"))
                        .collect::<Vec<_>>()
                        .join(", ");
                    return Err(toml_span::Error::from((
                        ErrorKind::Custom(
                            format!("undeclared profile '{profile_name}' used here, expected values: [{expected}]").into(),
                        ),
                        profile_key.span,
                    ))
                    .into());
                };

                let output: &mut OrderMap<String, ResourceDto> =
                    sections.entry(profile_name.clone()).or_default();

                let mut th = TableHelper::new(resources)?;
                for (res_name, res_value) in th.table.iter_mut() {
                    let res_name = res_name.to_string();
                    output.insert(
                        res_name,
                        ResourceDto::parse_with_ctx(
                            res_value,
                            ResourceDtoContext {
                                declared_remote_ids: ctx.declared_remote_ids,
                                profile: profile,
                            },
                        )?,
                    );
                }
            }

            Ok(Self(sections))
        }
    }

    impl<'de> ParseWithContext<'de> for ResourceDto {
        type Context = ResourceDtoContext<'de>;

        fn parse_with_ctx(
            value: &mut toml_span::Value<'de>,
            ctx: Self::Context,
        ) -> std::result::Result<Self, toml_span::DeserError> {
            // region: extract
            let (node_name, override_profile) = match value.as_str() {
                Some(value) => (value.to_owned(), None),
                None => {
                    let mut th = TableHelper::new(value)?;
                    let name = th.required::<String>("name")?;
                    th.finalize(Some(value))?;

                    use Profile::*;
                    let override_profile = match ctx.profile.as_ref() {
                        Png(_) => {
                            ProfileDto::Png(PngProfileDto::parse_with_ctx(value, ctx.into())?)
                        }
                        Svg(_) => {
                            ProfileDto::Svg(SvgProfileDto::parse_with_ctx(value, ctx.into())?)
                        }
                        Pdf(_) => {
                            ProfileDto::Pdf(PdfProfileDto::parse_with_ctx(value, ctx.into())?)
                        }
                        Webp(_) => {
                            ProfileDto::Webp(WebpProfileDto::parse_with_ctx(value, ctx.into())?)
                        }
                        Compose(_) => ProfileDto::Compose(ComposeProfileDto::parse_with_ctx(
                            value,
                            ctx.into(),
                        )?),
                        AndroidWebp(_) => ProfileDto::AndroidWebp(
                            AndroidWebpProfileDto::parse_with_ctx(value, ctx.into())?,
                        ),
                    };
                    (name, Some(override_profile))
                }
            };
            // endregion: extract
            Ok(Self {
                node_name,
                profile: ctx.profile.clone(),
                override_profile,
            })
        }
    }
}
