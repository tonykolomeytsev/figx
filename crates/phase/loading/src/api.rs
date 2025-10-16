use std::{
    collections::{BTreeMap, HashSet},
    fmt::{Debug, Display},
    ops::{Deref, Range},
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
    pub out_dir: PathBuf,
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
    pub container_node_ids: NodeIdList,
    pub access_token: String,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum NodeIdList {
    Plain(Vec<String>),
    IdToTag(BTreeMap<String, String>),
}

impl NodeIdList {
    pub fn to_string_id_list(&self) -> Vec<String> {
        match self {
            Self::Plain(ids) => ids.to_owned(),
            Self::IdToTag(table) => table.keys().cloned().collect(),
        }
    }
}

pub type RemoteId = String;

impl Display for RemoteSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}/{}", self.id, self.file_key)
    }
}

impl Debug for RemoteSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}/{}/[", self.id, self.file_key)?;
        match &self.container_node_ids {
            NodeIdList::Plain(ids) => write!(f, "{}", ids.join(","))?,
            NodeIdList::IdToTag(table) => {
                for (idx, (id, tag)) in table.iter().enumerate() {
                    if idx > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}=>{}", id, tag)?;
                }
            }
        }
        write!(f, "]")
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
    AndroidDrawable(AndroidDrawableProfile),
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
            AndroidDrawable(p) => p.remote_id.as_str(),
        }
    }

    pub fn vector(&self) -> bool {
        use Profile::*;
        match self {
            Png(_) | Webp(_) | AndroidWebp(_) => false,
            _ => true,
        }
    }
}

// region: PNG Profile

#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct PngProfile {
    pub remote_id: RemoteId,
    pub scale: ExportScale,
    pub output_dir: PathBuf,
    pub variants: Option<ResourceVariants>,
    pub legacy_loader: bool,
}

impl Default for PngProfile {
    fn default() -> Self {
        Self {
            remote_id: String::new(),
            scale: ExportScale::default(),
            output_dir: PathBuf::new(),
            variants: None,
            legacy_loader: false,
        }
    }
}

// endregion: PNG Profile

// region: SVG Profile

#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct SvgProfile {
    pub remote_id: RemoteId,
    pub output_dir: PathBuf,
    pub variants: Option<ResourceVariants>,
}

impl Default for SvgProfile {
    fn default() -> Self {
        Self {
            remote_id: String::new(),
            output_dir: PathBuf::new(),
            variants: None,
        }
    }
}

// endregion: SVG Profile

// region: PDF Profile

#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct PdfProfile {
    pub remote_id: RemoteId,
    pub output_dir: PathBuf,
    pub variants: Option<ResourceVariants>,
}

impl Default for PdfProfile {
    fn default() -> Self {
        Self {
            remote_id: String::new(),
            output_dir: PathBuf::new(),
            variants: None,
        }
    }
}

// endregion: PDF Profile

// region: WEBP Profile

#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct WebpProfile {
    pub remote_id: RemoteId,
    pub scale: ExportScale,
    pub quality: WebpQuality,
    pub output_dir: PathBuf,
    pub variants: Option<ResourceVariants>,
    pub legacy_loader: bool,
}

impl Default for WebpProfile {
    fn default() -> Self {
        Self {
            remote_id: String::new(),
            scale: ExportScale::default(),
            quality: WebpQuality::default(),
            output_dir: PathBuf::new(),
            variants: None,
            legacy_loader: false,
        }
    }
}

// endregion: WEBP Profile

// region: COMPOSE Profile

#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct ComposeProfile {
    pub remote_id: RemoteId,
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

impl Default for ComposeProfile {
    fn default() -> Self {
        Self {
            remote_id: String::new(),
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

// endregion: COMPOSE Profile

// region: ANDROID-WEBP Profile

#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct AndroidWebpProfile {
    pub remote_id: RemoteId,
    pub android_res_dir: PathBuf,
    pub quality: WebpQuality,
    pub scales: Vec<AndroidDensity>,
    pub night: Option<SingleNamePattern>,
    pub legacy_loader: bool,
}

impl Default for AndroidWebpProfile {
    fn default() -> Self {
        use AndroidDensity::*;
        Self {
            remote_id: String::new(),
            android_res_dir: PathBuf::from("src/main/res"),
            quality: WebpQuality::default(),
            scales: vec![MDPI, HDPI, XHDPI, XXHDPI, XXXHDPI],
            night: None,
            legacy_loader: false,
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

// endregion: ANDROID-WEBP Profile

// region: ANDROID-DRAWABLE Profile

#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct AndroidDrawableProfile {
    pub remote_id: RemoteId,
    pub android_res_dir: PathBuf,
    pub night: Option<SingleNamePattern>,
}

impl Default for AndroidDrawableProfile {
    fn default() -> Self {
        Self {
            remote_id: String::new(),
            android_res_dir: PathBuf::from("src/main/res"),
            night: None,
        }
    }
}

// endregion: ANDROID-DRAWABLE Profile

// region VARIANTS-API

#[derive(Clone)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct ResourceVariants {
    pub all_variants: BTreeMap<String, ResourceVariant>,
    pub use_variants: Option<HashSet<String>>,
}

#[derive(Clone)]
#[cfg_attr(test, derive(PartialEq, Debug))]
pub struct ResourceVariant {
    pub output_name: SingleNamePattern,
    pub figma_name: SingleNamePattern,
    pub scale: Option<ExportScale>,
}

// endregion: VARIANTS-API

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
    pub diag: ResourceDiagnostics,
}

pub struct ResourceDiagnostics {
    pub file: Arc<PathBuf>,
    pub definition_span: Range<usize>,
}

// region: Validated primitives

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct ExportScale(pub(crate) f32);

impl Default for ExportScale {
    fn default() -> Self {
        Self(1.0)
    }
}

impl Deref for ExportScale {
    type Target = f32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Debug for ExportScale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Display for ExportScale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct WebpQuality(pub(crate) f32);

impl Default for WebpQuality {
    fn default() -> Self {
        Self(100.0)
    }
}

impl Deref for WebpQuality {
    type Target = f32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Debug for WebpQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Display for WebpQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct SingleNamePattern(pub(crate) String);

impl AsRef<str> for SingleNamePattern {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Debug for SingleNamePattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Display for SingleNamePattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// endregion: Validated primitives
