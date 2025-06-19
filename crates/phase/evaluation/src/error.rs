use std::{
    fmt::{Debug, Display},
    ops::Range,
    path::PathBuf,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    Cache(lib_cache::Error),
    WebpCreate,
    ImageDecode(image::ImageError),
    FigmaApiNetwork(lib_figma_fluent::Error),
    ExportImage(String),
    IndexingRemote(String),
    FindNode {
        node_name: String,
        file: PathBuf,
        span: Range<usize>,
    },
    SvgToCompose(lib_svg2compose::Error),
    RenderSvg(String),
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

impl From<image::ImageError> for Error {
    fn from(value: image::ImageError) -> Self {
        Self::ImageDecode(value)
    }
}

impl From<lib_figma_fluent::Error> for Error {
    fn from(value: lib_figma_fluent::Error) -> Self {
        Self::FigmaApiNetwork(value)
    }
}

impl From<lib_svg2compose::Error> for Error {
    fn from(value: lib_svg2compose::Error) -> Self {
        Self::SvgToCompose(value)
    }
}

impl From<retry::Error<lib_figma_fluent::Error>> for Error {
    fn from(value: retry::Error<lib_figma_fluent::Error>) -> Self {
        value.error.into()
    }
}

impl From<retry::Error<Error>> for Error {
    fn from(value: retry::Error<Error>) -> Self {
        value.error.into()
    }
}

impl From<lib_figma_fluent::NodeStreamError> for Error {
    fn from(value: lib_figma_fluent::NodeStreamError) -> Self {
        Self::ExportImage(value.0)
    }
}
