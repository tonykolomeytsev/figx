use crate::{
    AndroidWebpProfile, ComposeProfile, Error, PdfProfile, PngProfile, Profile, RemoteId, Result,
    SvgProfile, WebpProfile,
};
use ordermap::OrderMap;
use serde::Deserialize;
use std::{
    collections::{BTreeSet, HashMap},
    path::PathBuf,
    sync::Arc,
};
use toml::Table;

#[derive(Deserialize, Default)]
pub(super) struct ProfilesDto {
    #[serde(flatten)]
    builtins: BuiltInProfiles,
    #[serde(flatten)]
    custom: HashMap<String, CustomProfileDto>,
}

#[derive(Deserialize, Default)]
struct BuiltInProfiles {
    #[serde(default = "Default::default")]
    png: PngProfileDto,

    #[serde(default = "Default::default")]
    svg: SvgProfileDto,

    #[serde(default = "Default::default")]
    pdf: PdfProfileDto,

    #[serde(default = "Default::default")]
    webp: WebpProfileDto,

    #[serde(default = "Default::default")]
    compose: ComposeProfileDto,

    #[serde(rename = "android-webp")]
    #[serde(default = "Default::default")]
    android_webp: AndroidWebpProfileDto,
}

#[derive(Deserialize)]
struct CustomProfileDto {
    pub extends: String,
    #[serde(flatten)]
    pub profile: Table,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(super) struct PngProfileDto {
    #[serde(rename = "remote")]
    pub remote_id: Option<RemoteId>,
    pub scale: Option<f32>,
    pub output_dir: Option<PathBuf>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(super) struct SvgProfileDto {
    #[serde(rename = "remote")]
    pub remote_id: Option<RemoteId>,
    pub scale: Option<f32>,
    pub output_dir: Option<PathBuf>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(super) struct PdfProfileDto {
    #[serde(rename = "remote")]
    pub remote_id: Option<RemoteId>,
    pub scale: Option<f32>,
    pub output_dir: Option<PathBuf>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(super) struct WebpProfileDto {
    #[serde(rename = "remote")]
    pub remote_id: Option<RemoteId>,
    pub scale: Option<f32>,
    pub quality: Option<f32>,
    pub output_dir: Option<PathBuf>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(super) struct ComposeProfileDto {
    #[serde(rename = "remote")]
    pub remote_id: Option<RemoteId>,
    pub scale: Option<f32>,
    pub src_dir: Option<PathBuf>,
    pub package: Option<String>,
    pub kotlin_explicit_api: Option<bool>,
    pub extension_target: Option<String>,
    pub file_suppress_lint: Option<BTreeSet<String>>,
    pub color_mappings: Option<Vec<ColorMappingDto>>,
    pub preview: Option<ComposePreviewDto>,
    pub variant_naming: Option<ResourceVariantNamingDto>,
    pub variants: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ColorMappingDto {
    pub from: String,
    pub to: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ComposePreviewDto {
    pub imports: Vec<String>,
    pub code: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ResourceVariantNamingDto {
    pub local_name: String,
    pub figma_name: String,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(super) struct AndroidWebpProfileDto {
    #[serde(rename = "remote")]
    pub remote_id: Option<RemoteId>,
    pub android_res_dir: Option<PathBuf>,
    pub quality: Option<f32>,
    pub scales: Option<BTreeSet<AndroidDensity>>,
    pub night: Option<String>,
}

#[derive(Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub(super) enum AndroidDensity {
    LDPI,
    MDPI,
    HDPI,
    XHDPI,
    XXHDPI,
    XXXHDPI,
}

pub(super) fn parse_profiles(
    ProfilesDto { builtins, custom }: ProfilesDto,
) -> Result<OrderMap<String, Arc<Profile>>> {
    let mut profiles: OrderMap<String, Arc<Profile>> = OrderMap::new();
    parse_builtin_profiles(builtins, &mut profiles);
    parse_custom_profiles(custom, &mut profiles)?;
    Ok(profiles)
}

fn parse_builtin_profiles(
    builtins: BuiltInProfiles,
    profiles: &mut OrderMap<String, Arc<Profile>>,
) {
    profiles.insert(
        "png".to_string(),
        Arc::new(Profile::Png(PngProfile::default().extend(builtins.png))),
    );
    profiles.insert(
        "svg".to_string(),
        Arc::new(Profile::Svg(SvgProfile::default().extend(builtins.svg))),
    );
    profiles.insert(
        "pdf".to_string(),
        Arc::new(Profile::Pdf(PdfProfile::default().extend(builtins.pdf))),
    );
    profiles.insert(
        "webp".to_string(),
        Arc::new(Profile::Webp(WebpProfile::default().extend(builtins.webp))),
    );
    profiles.insert(
        "compose".to_string(),
        Arc::new(Profile::Compose(
            ComposeProfile::default().extend(builtins.compose),
        )),
    );
    profiles.insert(
        "android-webp".to_string(),
        Arc::new(Profile::AndroidWebp(
            AndroidWebpProfile::default().extend(builtins.android_webp),
        )),
    );
}

fn parse_custom_profiles(
    custom: HashMap<String, CustomProfileDto>,
    profiles: &mut OrderMap<String, Arc<Profile>>,
) -> Result<()> {
    for (name, attrs) in custom {
        if let Some(base) = profiles.get(&attrs.extends) {
            match base.as_ref() {
                Profile::Png(base) => {
                    let custom: PngProfileDto =
                        attrs.profile.try_into().map_err(Error::FigParse)?;
                    profiles.insert(name, Arc::new(Profile::Png(base.extend(custom))));
                }

                Profile::Svg(base) => {
                    let custom: SvgProfileDto =
                        attrs.profile.try_into().map_err(Error::FigParse)?;
                    profiles.insert(name, Arc::new(Profile::Svg(base.extend(custom))));
                }

                Profile::Pdf(base) => {
                    let custom: PdfProfileDto =
                        attrs.profile.try_into().map_err(Error::FigParse)?;
                    profiles.insert(name, Arc::new(Profile::Pdf(base.extend(custom))));
                }

                Profile::Webp(base) => {
                    let custom: WebpProfileDto =
                        attrs.profile.try_into().map_err(Error::FigParse)?;
                    profiles.insert(name, Arc::new(Profile::Webp(base.extend(custom))));
                }

                Profile::Compose(base) => {
                    let custom: ComposeProfileDto =
                        attrs.profile.try_into().map_err(Error::FigParse)?;
                    profiles.insert(name, Arc::new(Profile::Compose(base.extend(custom))));
                }

                Profile::AndroidWebp(base) => {
                    let custom: AndroidWebpProfileDto =
                        attrs.profile.try_into().map_err(Error::FigParse)?;
                    profiles.insert(name, Arc::new(Profile::AndroidWebp(base.extend(custom))));
                }
            }
        } else {
            return Err(Error::WorkspaceInvalidProfileToExtend(name, attrs.extends));
        }
    }
    Ok(())
}

pub(crate) trait CanBeExtendedBy<T> {
    fn extend(&self, another: T) -> Self;
}

impl CanBeExtendedBy<PngProfileDto> for PngProfile {
    fn extend(&self, another: PngProfileDto) -> Self {
        Self {
            remote_id: another.remote_id.unwrap_or(self.remote_id.clone()),
            scale: another.scale.unwrap_or(self.scale),
            output_dir: another.output_dir.unwrap_or(self.output_dir.clone()),
        }
    }
}

impl CanBeExtendedBy<SvgProfileDto> for SvgProfile {
    fn extend(&self, another: SvgProfileDto) -> Self {
        Self {
            remote_id: another.remote_id.unwrap_or(self.remote_id.clone()),
            scale: another.scale.unwrap_or(self.scale),
            output_dir: another.output_dir.unwrap_or(self.output_dir.clone()),
        }
    }
}

impl CanBeExtendedBy<PdfProfileDto> for PdfProfile {
    fn extend(&self, another: PdfProfileDto) -> Self {
        Self {
            remote_id: another.remote_id.unwrap_or(self.remote_id.clone()),
            scale: another.scale.unwrap_or(self.scale),
            output_dir: another.output_dir.unwrap_or(self.output_dir.clone()),
        }
    }
}

impl CanBeExtendedBy<WebpProfileDto> for WebpProfile {
    fn extend(&self, another: WebpProfileDto) -> Self {
        Self {
            remote_id: another.remote_id.unwrap_or(self.remote_id.clone()),
            scale: another.scale.unwrap_or(self.scale),
            quality: another.quality.unwrap_or(self.quality),
            output_dir: another.output_dir.unwrap_or(self.output_dir.clone()),
        }
    }
}

impl CanBeExtendedBy<ComposeProfileDto> for ComposeProfile {
    fn extend(&self, another: ComposeProfileDto) -> Self {
        Self {
            remote_id: another.remote_id.unwrap_or(self.remote_id.clone()),
            scale: another.scale.unwrap_or(self.scale),
            src_dir: another.src_dir.unwrap_or(self.src_dir.clone()),
            package: another.package.or(self.package.clone()),
            kotlin_explicit_api: another
                .kotlin_explicit_api
                .unwrap_or(self.kotlin_explicit_api),
            extension_target: another.extension_target.or(self.extension_target.clone()),
            file_suppress_lint: another
                .file_suppress_lint
                .map(|it| it.into_iter().collect())
                .unwrap_or(self.file_suppress_lint.to_owned()),
            color_mappings: another
                .color_mappings
                .map(|it| it.into_iter().map(Into::into).collect())
                .unwrap_or(self.color_mappings.clone()),
            preview: another.preview.map(Into::into).or(self.preview.clone()),
            variant_naming: another
                .variant_naming
                .map(Into::into)
                .unwrap_or_else(|| self.variant_naming.clone()),
            variants: another.variants.or_else(|| self.variants.clone()),
        }
    }
}

impl CanBeExtendedBy<AndroidWebpProfileDto> for AndroidWebpProfile {
    fn extend(&self, another: AndroidWebpProfileDto) -> Self {
        Self {
            remote_id: another.remote_id.unwrap_or(self.remote_id.clone()),
            android_res_dir: another
                .android_res_dir
                .unwrap_or(self.android_res_dir.clone()),
            quality: another.quality.unwrap_or(self.quality),
            scales: another
                .scales
                .map(|set| set.into_iter().map(Into::into).collect())
                .unwrap_or_else(|| self.scales.clone()),
            night: another.night.or(self.night.clone()),
        }
    }
}

impl From<AndroidDensity> for crate::AndroidDensity {
    fn from(value: AndroidDensity) -> Self {
        use crate::AndroidDensity::*;
        match value {
            AndroidDensity::LDPI => LDPI,
            AndroidDensity::MDPI => MDPI,
            AndroidDensity::HDPI => HDPI,
            AndroidDensity::XHDPI => XHDPI,
            AndroidDensity::XXHDPI => XXHDPI,
            AndroidDensity::XXXHDPI => XXXHDPI,
        }
    }
}

impl From<ColorMappingDto> for crate::ColorMapping {
    fn from(value: ColorMappingDto) -> Self {
        Self {
            from: value.from,
            to: value.to,
        }
    }
}

impl From<ComposePreviewDto> for crate::ComposePreview {
    fn from(value: ComposePreviewDto) -> Self {
        Self {
            imports: value.imports,
            code: value.code,
        }
    }
}

impl From<ResourceVariantNamingDto> for crate::ResourceVariantNaming {
    fn from(value: ResourceVariantNamingDto) -> Self {
        Self {
            local_name: value.local_name.to_owned(),
            figma_name: value.figma_name.to_owned(),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {
    use std::str::FromStr;

    use ordermap::ordermap;

    use super::*;

    #[test]
    fn parse_empty_profile_definition__EXPECT__only_builtin_profiles() {
        // Given
        let text = r#"
        "#;
        let model = ordermap! {
            "png".to_string() => Arc::new(Profile::Png(PngProfile::default())),
            "svg".to_string() => Arc::new(Profile::Svg(SvgProfile::default())),
            "pdf".to_string() => Arc::new(Profile::Pdf(PdfProfile::default())),
            "webp".to_string() => Arc::new(Profile::Webp(WebpProfile::default())),
            "compose".to_string() => Arc::new(Profile::Compose(ComposeProfile::default())),
            "android-webp".to_string() => Arc::new(Profile::AndroidWebp(AndroidWebpProfile::default())),
        };

        // When
        let result = parse_profiles(toml::from_str(text).unwrap());

        // Then
        assert_eq!(model, result.unwrap());
    }

    #[test]
    fn parse_overridden_profile_definition__EXPECT__only_overridden_builtin_profiles() {
        // Given
        let text = r#"
        [svg]
        output_dir = "res/img"
        scale = 2
        
        [png]
        output_dir = "res/img"
        scale = 4
        "#;
        let model = ordermap! {
            "png".to_string() => Arc::new(Profile::Png(PngProfile {
                scale: 4.0,
                output_dir: PathBuf::from_str("res/img").unwrap(),
                ..Default::default()
            })),
            "svg".to_string() => Arc::new(Profile::Svg(SvgProfile {
                scale: 2.0,
                output_dir: PathBuf::from_str("res/img").unwrap(),
                ..Default::default()
            })),
            "pdf".to_string() => Arc::new(Profile::Pdf(PdfProfile::default())),
            "webp".to_string() => Arc::new(Profile::Webp(WebpProfile::default())),
            "compose".to_string() => Arc::new(Profile::Compose(ComposeProfile::default())),
            "android-webp".to_string() => Arc::new(Profile::AndroidWebp(AndroidWebpProfile::default())),
        };

        // When
        let result = parse_profiles(toml::from_str(text).unwrap());

        // Then
        assert_eq!(model, result.unwrap());
    }

    #[test]
    fn parse_custom_profile_definition__EXPECT__builtin_and_custom_profiles() {
        // Given
        let text = r#"
        [illustrations]
        extends = "webp"
        remote = "features"
        quality = 90
        "#;
        let model = ordermap! {
            "png".to_string() => Arc::new(Profile::Png(PngProfile::default())),
            "svg".to_string() => Arc::new(Profile::Svg(SvgProfile::default())),
            "pdf".to_string() => Arc::new(Profile::Pdf(PdfProfile::default())),
            "webp".to_string() => Arc::new(Profile::Webp(WebpProfile::default())),
            "compose".to_string() => Arc::new(Profile::Compose(ComposeProfile::default())),
            "android-webp".to_string() => Arc::new(Profile::AndroidWebp(AndroidWebpProfile::default())),
            "illustrations".to_string() => Arc::new(Profile::Webp(WebpProfile {
                remote_id: "features".to_string(),
                quality: 90.0,
                ..Default::default()
            })),
        };

        // When
        let result = parse_profiles(toml::from_str(text).unwrap());

        // Then
        assert_eq!(model, result.unwrap());
    }
}
