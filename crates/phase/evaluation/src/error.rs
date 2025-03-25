use std::fmt::{Debug, Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    Cache(lib_cache::Error),
    GraphConfiguration(lib_graph_exec::unconfigured::Error),
    WebpCreate,
    ImageDecode(image::ImageError),
    FigmaApiNetwork(lib_figma::Error),
    ExportImage(String),
    FindNode { node_name: String },
    ActionSingleInputAbsent,
    ActionTaggedInputAbsent,
    SvgToCompose(lib_svg2compose::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}
impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<lib_cache::Error> for Error {
    fn from(value: lib_cache::Error) -> Self {
        Self::Cache(value)
    }
}

impl From<lib_graph_exec::unconfigured::Error> for Error {
    fn from(value: lib_graph_exec::unconfigured::Error) -> Self {
        Self::GraphConfiguration(value)
    }
}

impl From<image::ImageError> for Error {
    fn from(value: image::ImageError) -> Self {
        Self::ImageDecode(value)
    }
}

impl From<lib_figma::Error> for Error {
    fn from(value: lib_figma::Error) -> Self {
        Self::FigmaApiNetwork(value)
    }
}

impl From<lib_svg2compose::Error> for Error {
    fn from(value: lib_svg2compose::Error) -> Self {
        Self::SvgToCompose(value)
    }
}
