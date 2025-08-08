use std::fmt::Display;

pub type Result<T> = ::std::result::Result<T, Error>;

pub enum Error {
    WorkspaceError(phase_loading::Error),
    UserError(String),
    Io(std::io::Error),
    FigmaError(lib_figma_fluent::Error),
    IndexingRemote(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WorkspaceError(err) => write!(f, "scan error: {err}"),
            Self::UserError(err) => write!(f, "scan error: {err}"),
            Self::Io(err) => write!(f, "scan error: {err}"),
            Self::FigmaError(err) => write!(f, "scan error: {err}"),
            Self::IndexingRemote(err) => write!(f, "scan error: {err}"),
        }
    }
}

impl From<phase_loading::Error> for Error {
    fn from(value: phase_loading::Error) -> Self {
        Self::WorkspaceError(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<lib_figma_fluent::Error> for Error {
    fn from(value: lib_figma_fluent::Error) -> Self {
        Self::FigmaError(value)
    }
}
