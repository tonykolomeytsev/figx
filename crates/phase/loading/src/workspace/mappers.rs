use crate::{
    AndroidWebpProfile, CanBeExtendedBy, ComposeProfile, PdfProfile, PngProfile, SvgProfile,
    WebpProfile,
    parser::{
        AndroidDensityDto, AndroidWebpProfileDto, ColorMappingDto, ComposePreviewDto,
        ComposeProfileDto, PdfProfileDto, PngProfileDto, ResourceVariantNamingDto, SvgProfileDto,
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
            variant_naming: another
                .variant_naming
                .clone()
                .map(Into::into)
                .unwrap_or_else(|| self.variant_naming.clone()),
            variants: another.variants.clone().or_else(|| self.variants.clone()),
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

impl From<ResourceVariantNamingDto> for crate::ResourceVariantNaming {
    fn from(value: ResourceVariantNamingDto) -> Self {
        Self {
            local_name: value.local_name.to_owned(),
            figma_name: value.figma_name.to_owned(),
        }
    }
}
