use super::{
    AndroidWebpProfileDtoContext, ComposeProfileDto, PdfProfileDto, PdfProfileDtoContext,
    PngProfileDto, PngProfileDtoContext, SvgProfileDto, SvgProfileDtoContext, WebpProfileDto,
    WebpProfileDtoContext, android_webp_profile_dto::AndroidWebpProfileDto,
    compose_profile_dto::ComposeProfileDtoContext,
};
use ordermap::OrderMap;
use std::collections::HashSet;

#[derive(Default)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub(crate) struct ProfilesDto(pub OrderMap<String, ProfileDto>);

#[derive(Clone, Copy)]
pub(crate) struct ProfilesDtoContext<'a> {
    pub declared_remote_ids: &'a HashSet<String>,
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

from_ctx_impl!(ProfilesDtoContext, PngProfileDtoContext);
from_ctx_impl!(ProfilesDtoContext, SvgProfileDtoContext);
from_ctx_impl!(ProfilesDtoContext, PdfProfileDtoContext);
from_ctx_impl!(ProfilesDtoContext, WebpProfileDtoContext);
from_ctx_impl!(ProfilesDtoContext, ComposeProfileDtoContext);
from_ctx_impl!(ProfilesDtoContext, AndroidWebpProfileDtoContext);

#[cfg_attr(test, derive(PartialEq, Debug))]
pub(crate) enum ProfileDto {
    Png(PngProfileDto),
    Svg(SvgProfileDto),
    Pdf(PdfProfileDto),
    Webp(WebpProfileDto),
    Compose(ComposeProfileDto),
    AndroidWebp(AndroidWebpProfileDto),
}

mod de {
    use super::*;
    use crate::{
        CanBeExtendedBy, ParseWithContext, parser::android_webp_profile_dto::AndroidWebpProfileDto,
    };
    use ordermap::ordermap;
    use toml_span::{ErrorKind, de_helpers::TableHelper};

    impl<'de> ParseWithContext<'de> for ProfilesDto {
        type Context = ProfilesDtoContext<'de>;

        fn parse_with_ctx(
            value: &mut toml_span::Value<'de>,
            ctx: Self::Context,
        ) -> std::result::Result<Self, toml_span::DeserError> {
            // region: extract
            let mut th = TableHelper::new(value)?;
            let mut profiles = OrderMap::with_capacity(th.table.len());

            // region: built-ins
            let png_profile_dto = match th.take("png") {
                Some((_, mut value)) => PngProfileDto::parse_with_ctx(&mut value, ctx.into())?,
                None => PngProfileDto::default(),
            };
            let svg_profile_dto = match th.take("svg") {
                Some((_, mut value)) => SvgProfileDto::parse_with_ctx(&mut value, ctx.into())?,
                None => SvgProfileDto::default(),
            };
            let pdf_profile_dto = match th.take("pdf") {
                Some((_, mut value)) => PdfProfileDto::parse_with_ctx(&mut value, ctx.into())?,
                None => PdfProfileDto::default(),
            };
            let webp_profile_dto = match th.take("webp") {
                Some((_, mut value)) => WebpProfileDto::parse_with_ctx(&mut value, ctx.into())?,
                None => WebpProfileDto::default(),
            };
            let compose_profile_dto = match th.take("compose") {
                Some((_, mut value)) => ComposeProfileDto::parse_with_ctx(&mut value, ctx.into())?,
                None => ComposeProfileDto::default(),
            };
            let android_webp_profile_dto = match th.take("android-webp") {
                Some((_, mut value)) => {
                    AndroidWebpProfileDto::parse_with_ctx(&mut value, ctx.into())?
                }
                None => AndroidWebpProfileDto::default(),
            };
            // region: built-ins

            for (key, value) in th.table.iter_mut() {
                let profile_id = key.to_string();
                let mut th = TableHelper::new(value)?;
                let extends = th.required_s::<String>("extends")?;
                th.finalize(Some(value))?;

                let profile = match extends.value.as_str() {
                    "png" => ProfileDto::Png(
                        png_profile_dto.extend(&PngProfileDto::parse_with_ctx(value, ctx.into())?),
                    ),
                    "svg" => ProfileDto::Svg(
                        svg_profile_dto.extend(&SvgProfileDto::parse_with_ctx(value, ctx.into())?),
                    ),
                    "pdf" => ProfileDto::Pdf(
                        pdf_profile_dto.extend(&PdfProfileDto::parse_with_ctx(value, ctx.into())?),
                    ),
                    "webp" => ProfileDto::Webp(
                        webp_profile_dto
                            .extend(&WebpProfileDto::parse_with_ctx(value, ctx.into())?),
                    ),
                    "compose" => ProfileDto::Compose(
                        compose_profile_dto
                            .extend(&ComposeProfileDto::parse_with_ctx(value, ctx.into())?),
                    ),
                    "android-webp" => ProfileDto::AndroidWebp(
                        android_webp_profile_dto
                            .extend(&AndroidWebpProfileDto::parse_with_ctx(value, ctx.into())?),
                    ),
                    unknown => {
                        return Err(toml_span::Error::from((
                            ErrorKind::UnexpectedValue {
                                expected: &["png", "svg", "pdf", "webp", "compose", "android-webp"],
                                value: Some(unknown.to_string()),
                            },
                            extends.span,
                        ))
                        .into());
                    }
                };
                profiles.insert(profile_id, profile);
            }
            th.finalize(Some(value))?;
            
            profiles.append(&mut ordermap! {
                "png".to_string() => ProfileDto::Png(png_profile_dto),
                "svg".to_string() => ProfileDto::Svg(svg_profile_dto),
                "pdf".to_string() => ProfileDto::Pdf(pdf_profile_dto),
                "webp".to_string() => ProfileDto::Webp(webp_profile_dto),
                "compose".to_string() => ProfileDto::Compose(compose_profile_dto),
                "android-webp".to_string() => ProfileDto::AndroidWebp(android_webp_profile_dto),
            });
            // endregion: extract

            Ok(Self(profiles))
        }
    }
}
