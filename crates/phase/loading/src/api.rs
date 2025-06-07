use std::{
    fmt::{Debug, Display},
    path::PathBuf,
    sync::Arc,
};

use lib_label::Label;
use lib_label::Package as PackageLabel;

/// Represents a workspace that contains all the configuration data
/// for importing resources from Figma into the project.
///
/// A workspace must have at least one `RemoteSource` and can contain
/// multiple `Profile`s and `Resource`s.
pub struct Workspace {
    pub context: InvocationContext,
    pub remotes: Vec<Arc<RemoteSource>>,
    pub profiles: Vec<Arc<Profile>>,
    pub packages: Vec<Package>,
}

pub struct InvocationContext {
    pub workspace_dir: PathBuf,
    pub workspace_file: PathBuf,
    pub current_dir: PathBuf,
    pub current_package: Option<PackageLabel>,
    pub fig_files: Vec<LoadedFigFile>,
    pub cache_dir: PathBuf,
}

pub struct LoadedFigFile {
    pub package: PackageLabel,
    pub fig_dir: PathBuf,
    pub fig_file: PathBuf,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct RemoteSource {
    pub id: RemoteId,
    pub file_key: String,
    pub container_node_ids: Vec<String>,
    pub access_token: String,
}

pub type RemoteId = String;

impl Display for RemoteSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}/{}", self.id, self.file_key)
    }
}

impl Debug for RemoteSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "@{}/{}/[{}]",
            self.id,
            self.file_key,
            self.container_node_ids.join(", ")
        )
    }
}

/// Represents the specification of a resource, which varies depending on its type.
///
/// This enum defines the specific properties for each supported resource type.
#[cfg_attr(test, derive(PartialEq, Debug))]
pub enum Profile {
    Png(PngProfile),
    Svg(SvgProfile),
    Pdf(PdfProfile),
    Webp(WebpProfile),
    Compose(ComposeProfile),
    AndroidWebp(AndroidWebpProfile),
}

impl Profile {
    pub fn remote_id(&self) -> &str {
        use Profile::*;
        match self {
            Png(p) => p.remote_id.as_str(),
            Svg(p) => p.remote_id.as_str(),
            Pdf(p) => p.remote_id.as_str(),
            Webp(p) => p.remote_id.as_str(),
            Compose(p) => p.remote_id.as_str(),
            AndroidWebp(p) => p.remote_id.as_str(),
        }
    }
}

#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct PngProfile {
    pub remote_id: RemoteId,
    pub scale: f32,
    pub output_dir: PathBuf,
    pub variants: Option<ResourceVariants>,
}

impl Default for PngProfile {
    fn default() -> Self {
        Self {
            remote_id: String::new(),
            scale: 1.0,
            output_dir: PathBuf::new(),
            variants: None,
        }
    }
}

#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct SvgProfile {
    pub remote_id: RemoteId,
    pub scale: f32,
    pub output_dir: PathBuf,
    pub variants: Option<ResourceVariants>,
}

impl Default for SvgProfile {
    fn default() -> Self {
        Self {
            remote_id: String::new(),
            scale: 1.0,
            output_dir: PathBuf::new(),
            variants: None,
        }
    }
}

#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct PdfProfile {
    pub remote_id: RemoteId,
    pub scale: f32,
    pub output_dir: PathBuf,
    pub variants: Option<ResourceVariants>,
}

impl Default for PdfProfile {
    fn default() -> Self {
        Self {
            remote_id: String::new(),
            scale: 1.0,
            output_dir: PathBuf::new(),
            variants: None,
        }
    }
}

#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct WebpProfile {
    pub remote_id: RemoteId,
    pub scale: f32,
    pub quality: f32,
    pub output_dir: PathBuf,
    pub variants: Option<ResourceVariants>,
}

impl Default for WebpProfile {
    fn default() -> Self {
        Self {
            remote_id: String::new(),
            scale: 1.0,
            quality: 100.0,
            output_dir: PathBuf::new(),
            variants: None,
        }
    }
}

#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct ComposeProfile {
    pub remote_id: RemoteId,
    pub scale: f32,
    pub src_dir: PathBuf,
    pub package: Option<String>,
    pub kotlin_explicit_api: bool,
    pub extension_target: Option<String>,
    pub file_suppress_lint: Vec<String>,
    pub color_mappings: Vec<ColorMapping>,
    pub preview: Option<ComposePreview>,
    pub variants: Option<ResourceVariants>,
    pub composable_get: bool,
}

#[derive(Clone)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct ColorMapping {
    pub from: String,
    pub to: String,
    pub imports: Vec<String>,
}

#[derive(Clone)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct ComposePreview {
    pub imports: Vec<String>,
    pub code: String,
}

#[derive(Clone)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct ResourceVariants {
    pub naming: ResourceVariantNaming,
    pub list: Option<Vec<String>>,
}

#[derive(Clone)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct ResourceVariantNaming {
    pub local_name: String,
    pub figma_name: String,
}

impl Default for ResourceVariantNaming {
    fn default() -> Self {
        Self {
            local_name: "{base}{variant}".to_string(),
            figma_name: "{base}_{variant}".to_string(),
        }
    }
}

impl Default for ComposeProfile {
    fn default() -> Self {
        Self {
            remote_id: String::new(),
            scale: 1.0,
            src_dir: PathBuf::new(),
            package: None,
            kotlin_explicit_api: false,
            extension_target: None,
            file_suppress_lint: Vec::new(),
            color_mappings: Vec::new(),
            preview: None,
            variants: None,
            composable_get: false,
        }
    }
}

#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct AndroidWebpProfile {
    pub remote_id: RemoteId,
    pub android_res_dir: PathBuf,
    pub quality: f32,
    pub scales: Vec<AndroidDensity>,
    pub night: Option<String>,
}

impl Default for AndroidWebpProfile {
    fn default() -> Self {
        use AndroidDensity::*;
        Self {
            remote_id: String::new(),
            android_res_dir: PathBuf::from("src/main/res"),
            quality: 100.0,
            scales: vec![MDPI, HDPI, XHDPI, XXHDPI, XXXHDPI],
            night: None,
        }
    }
}

#[derive(Clone)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub enum AndroidDensity {
    LDPI,
    MDPI,
    HDPI,
    XHDPI,
    XXHDPI,
    XXXHDPI,
}

pub struct Package {
    pub label: PackageLabel,
    pub resources: Vec<Resource>,
    pub source_file: PathBuf,
}

/// Represents a resource to be imported from a remote source.
///
/// A resource corresponds to a single image/document in the remote source but can
/// result in multiple files in the project (e.g., different resolutions for Android).
///
/// Each resource has a `name`, a `package` it belongs to, and a `spec` that defines
/// its specific properties based on the resource type.
pub struct Resource {
    pub attrs: ResourceAttrs,
    pub profile: Arc<Profile>,
}

pub struct ResourceAttrs {
    pub label: Label,
    pub remote: Arc<RemoteSource>,
    pub node_name: String,
    pub package_dir: PathBuf,
}
