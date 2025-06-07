use std::{
    collections::{BTreeSet, HashSet},
    path::PathBuf,
};

use crate::CanBeExtendedBy;

use super::VariantsDto;

#[derive(Default)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub(crate) struct ComposeProfileDto {
    pub remote_id: Option<String>,
    pub scale: Option<f32>,
    pub src_dir: Option<PathBuf>,
    pub package: Option<String>,
    pub kotlin_explicit_api: Option<bool>,
    pub extension_target: Option<String>,
    pub file_suppress_lint: Option<BTreeSet<String>>,
    pub color_mappings: Option<Vec<ColorMappingDto>>,
    pub preview: Option<ComposePreviewDto>,
    pub variants: Option<VariantsDto>,
    pub composable_get: Option<bool>,
}

impl CanBeExtendedBy<ComposeProfileDto> for ComposeProfileDto {
    fn extend(&self, another: &ComposeProfileDto) -> Self {
        Self {
            remote_id: another
                .remote_id
                .as_ref()
                .or(self.remote_id.as_ref())
                .cloned(),
            scale: another.scale.or(self.scale),
            src_dir: another.src_dir.as_ref().or(self.src_dir.as_ref()).cloned(),
            package: another.package.as_ref().or(self.package.as_ref()).cloned(),
            kotlin_explicit_api: another.kotlin_explicit_api.or(self.kotlin_explicit_api),
            extension_target: another
                .extension_target
                .as_ref()
                .or(self.extension_target.as_ref())
                .cloned(),
            file_suppress_lint: another
                .file_suppress_lint
                .as_ref()
                .or(self.file_suppress_lint.as_ref())
                .cloned(),
            color_mappings: another
                .color_mappings
                .as_ref()
                .or(self.color_mappings.as_ref())
                .cloned(),
            preview: another.preview.as_ref().or(self.preview.as_ref()).cloned(),
            variants: another
                .variants
                .as_ref()
                .or(self.variants.as_ref())
                .cloned(),
            composable_get: another.composable_get.or(self.composable_get),
        }
    }
}

pub(crate) struct ComposeProfileDtoContext<'a> {
    pub declared_remote_ids: &'a HashSet<String>,
}

#[derive(Clone)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub(crate) struct ColorMappingDto {
    pub from: String,
    pub to: String,
    pub imports: Vec<String>,
}

#[derive(Clone)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub(crate) struct ComposePreviewDto {
    pub imports: Vec<String>,
    pub code: String,
}

mod de {
    use super::*;
    use crate::ParseWithContext;
    use crate::parser::de::parse_variants;
    use crate::parser::util::{validate_figma_scale, validate_remote_id};
    use toml_span::Deserialize;
    use toml_span::de_helpers::TableHelper;

    impl<'de> ParseWithContext<'de> for ComposeProfileDto {
        type Context = ComposeProfileDtoContext<'de>;

        fn parse_with_ctx(
            value: &mut toml_span::Value<'de>,
            ctx: Self::Context,
        ) -> std::result::Result<Self, toml_span::DeserError> {
            // region: extract
            let mut th = TableHelper::new(value)?;
            let remote_id = th.optional_s::<String>("remote");
            let scale = th.optional_s::<f32>("scale");
            let src_dir = th.optional::<String>("src_dir").map(PathBuf::from);
            let package = th.optional("package");
            let kotlin_explicit_api = th.optional("kotlin_explicit_api");
            let file_suppress_lint = th
                .optional::<Vec<String>>("file_suppress_lint")
                .map(|vec| vec.into_iter().collect::<BTreeSet<_>>());
            let extension_target = th.optional("extension_target");
            let color_mappings = th.optional("color_mappings");
            let preview = th.optional("preview");
            let variants = parse_variants(&mut th)?;
            let composable_get = th.optional("composable_get");
            th.finalize(None)?;
            // endregion: extract

            // region: validate
            let remote_id = validate_remote_id(remote_id, ctx.declared_remote_ids)?;
            let scale = validate_figma_scale(scale)?;
            // endregion: validate

            Ok(Self {
                remote_id,
                scale,
                src_dir,
                package,
                kotlin_explicit_api,
                file_suppress_lint,
                extension_target,
                color_mappings,
                preview,
                variants,
                composable_get,
            })
        }
    }

    impl<'de> Deserialize<'de> for ColorMappingDto {
        fn deserialize(value: &mut toml_span::Value<'de>) -> Result<Self, toml_span::DeserError> {
            let mut th = TableHelper::new(value)?;
            let from = th.required("from")?;
            let to = th.required("to")?;
            let imports = th.optional("imports").unwrap_or_default();
            th.finalize(None)?;

            Ok(Self { from, to, imports })
        }
    }

    impl<'de> Deserialize<'de> for ComposePreviewDto {
        fn deserialize(value: &mut toml_span::Value<'de>) -> Result<Self, toml_span::DeserError> {
            let mut th = TableHelper::new(value)?;
            let imports = th.optional("imports").unwrap_or_default();
            let code = th.required("code")?;
            th.finalize(None)?;

            Ok(Self { imports, code })
        }
    }
}
