use crate::{
    AndroidWebpProfile, CanBeExtendedBy, ComposeProfile, PdfProfile, PngProfile, ResourceVariants,
    SvgProfile, WebpProfile,
    parser::{
        AndroidDensityDto, AndroidWebpProfileDto, ColorMappingDto, ComposePreviewDto,
        ComposeProfileDto, PdfProfileDto, PngProfileDto, SvgProfileDto, VariantDto, VariantsDto,
        WebpProfileDto,
    },
};

impl CanBeExtendedBy<PngProfileDto> for PngProfile {
    fn extend(&self, another: &PngProfileDto) -> Self {
        Self {
            remote_id: another
                .remote_id
                .as_ref()
                .unwrap_or(&self.remote_id)
                .clone(),
            scale: another.scale.unwrap_or(self.scale),
            output_dir: another
                .output_dir
                .as_ref()
                .unwrap_or(&self.output_dir)
                .clone(),
            variants: match (another.variants.as_ref(), self.variants.as_ref()) {
                (Some(dto), Some(domain)) => Some(domain.extend(dto)),
                (Some(dto), None) => Some(dto.clone().into()),
                (None, Some(domain)) => Some(domain.clone()),
                _ => None,
            },
        }
    }
}

impl CanBeExtendedBy<SvgProfileDto> for SvgProfile {
    fn extend(&self, another: &SvgProfileDto) -> Self {
        Self {
            remote_id: another
                .remote_id
                .as_ref()
                .unwrap_or(&self.remote_id)
                .clone(),
            scale: another.scale.unwrap_or(self.scale),
            output_dir: another
                .output_dir
                .as_ref()
                .unwrap_or(&self.output_dir)
                .clone(),
            variants: match (another.variants.as_ref(), self.variants.as_ref()) {
                (Some(dto), Some(domain)) => Some(domain.extend(dto)),
                (Some(dto), None) => Some(dto.clone().into()),
                (None, Some(domain)) => Some(domain.clone()),
                _ => None,
            },
        }
    }
}

impl CanBeExtendedBy<PdfProfileDto> for PdfProfile {
    fn extend(&self, another: &PdfProfileDto) -> Self {
        Self {
            remote_id: another
                .remote_id
                .as_ref()
                .unwrap_or(&self.remote_id)
                .clone(),
            scale: another.scale.unwrap_or(self.scale),
            output_dir: another
                .output_dir
                .as_ref()
                .unwrap_or(&self.output_dir)
                .clone(),
            variants: match (another.variants.as_ref(), self.variants.as_ref()) {
                (Some(dto), Some(domain)) => Some(domain.extend(dto)),
                (Some(dto), None) => Some(dto.clone().into()),
                (None, Some(domain)) => Some(domain.clone()),
                _ => None,
            },
        }
    }
}

impl CanBeExtendedBy<WebpProfileDto> for WebpProfile {
    fn extend(&self, another: &WebpProfileDto) -> Self {
        Self {
            remote_id: another
                .remote_id
                .as_ref()
                .unwrap_or(&self.remote_id)
                .clone(),
            scale: another.scale.unwrap_or(self.scale),
            quality: another.quality.unwrap_or(self.quality),
            output_dir: another
                .output_dir
                .as_ref()
                .unwrap_or(&self.output_dir)
                .clone(),
            variants: match (another.variants.as_ref(), self.variants.as_ref()) {
                (Some(dto), Some(domain)) => Some(domain.extend(dto)),
                (Some(dto), None) => Some(dto.clone().into()),
                (None, Some(domain)) => Some(domain.clone()),
                _ => None,
            },
        }
    }
}

impl CanBeExtendedBy<ComposeProfileDto> for ComposeProfile {
    fn extend(&self, another: &ComposeProfileDto) -> Self {
        Self {
            remote_id: another
                .remote_id
                .as_ref()
                .unwrap_or(&self.remote_id)
                .clone(),
            scale: another.scale.unwrap_or(self.scale),
            src_dir: another.src_dir.as_ref().unwrap_or(&self.src_dir).clone(),
            package: another.package.clone().or(self.package.clone()),
            kotlin_explicit_api: another
                .kotlin_explicit_api
                .unwrap_or(self.kotlin_explicit_api),
            extension_target: another
                .extension_target
                .clone()
                .or_else(|| self.extension_target.clone()),
            file_suppress_lint: another
                .file_suppress_lint
                .as_ref()
                .map(|it| it.iter().cloned().collect())
                .unwrap_or(self.file_suppress_lint.to_owned()),
            color_mappings: another
                .color_mappings
                .as_ref()
                .map(|it| it.iter().cloned().map(Into::into).collect())
                .unwrap_or(self.color_mappings.clone()),
            preview: another
                .preview
                .clone()
                .map(Into::into)
                .or_else(|| self.preview.clone()),
            variants: match (another.variants.as_ref(), self.variants.as_ref()) {
                (Some(dto), Some(domain)) => Some(domain.extend(dto)),
                (Some(dto), None) => Some(dto.clone().into()),
                (None, Some(domain)) => Some(domain.clone()),
                _ => None,
            },
            composable_get: another.composable_get.unwrap_or(self.composable_get),
        }
    }
}

impl CanBeExtendedBy<AndroidWebpProfileDto> for AndroidWebpProfile {
    fn extend(&self, another: &AndroidWebpProfileDto) -> Self {
        Self {
            remote_id: another
                .remote_id
                .as_ref()
                .unwrap_or(&self.remote_id)
                .clone(),
            android_res_dir: another
                .android_res_dir
                .as_ref()
                .unwrap_or(&self.android_res_dir)
                .clone(),
            quality: another.quality.unwrap_or(self.quality),
            scales: another
                .densities
                .as_ref()
                .map(|set| set.iter().cloned().map(Into::into).collect())
                .unwrap_or_else(|| self.scales.clone()),
            night: another.night.clone().or_else(|| self.night.clone()),
        }
    }
}

impl From<AndroidDensityDto> for crate::AndroidDensity {
    fn from(value: AndroidDensityDto) -> Self {
        use crate::AndroidDensity::*;
        match value {
            AndroidDensityDto::LDPI => LDPI,
            AndroidDensityDto::MDPI => MDPI,
            AndroidDensityDto::HDPI => HDPI,
            AndroidDensityDto::XHDPI => XHDPI,
            AndroidDensityDto::XXHDPI => XXHDPI,
            AndroidDensityDto::XXXHDPI => XXXHDPI,
        }
    }
}

impl From<ColorMappingDto> for crate::ColorMapping {
    fn from(value: ColorMappingDto) -> Self {
        Self {
            from: value.from,
            to: value.to,
            imports: value.imports,
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

impl From<VariantsDto> for crate::ResourceVariants {
    fn from(value: VariantsDto) -> Self {
        Self {
            all_variants: match value.all_variants {
                Some(variants) => variants
                    .into_iter()
                    .map(|(k, v)| (k, v.clone().into()))
                    .collect(),
                None => Default::default(),
            },
            use_variants: value.use_variants.map(|it| it.into_iter().collect()),
        }
    }
}

impl From<VariantDto> for crate::ResourceVariant {
    fn from(value: VariantDto) -> Self {
        Self {
            output_name: value.output_name,
            figma_name: value.figma_name,
            scale: value.scale,
        }
    }
}

impl CanBeExtendedBy<VariantsDto> for ResourceVariants {
    fn extend(&self, another: &VariantsDto) -> Self {
        Self {
            all_variants: match another.all_variants.as_ref() {
                Some(variants) => variants
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone().into()))
                    .collect(),
                None => self.all_variants.clone(),
            },
            use_variants: another
                .use_variants
                .clone()
                .map(|it| it.into_iter().collect()),
        }
    }
}
