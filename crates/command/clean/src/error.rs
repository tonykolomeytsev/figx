use std::fmt::{Debug, Display};

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    WorkspaceError(phase_loading::Error),
    IO(std::io::Error),
    Evaluation(phase_evaluation::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}
impl std::error::Error for Error {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            Self::WorkspaceError(err) => Some(err),
            Self::IO(err) => Some(err),
            Self::Evaluation(err) => Some(err),
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
        Self::IO(value)
    }
}

impl From<phase_evaluation::Error> for Error {
    fn from(value: phase_evaluation::Error) -> Self {
        Self::Evaluation(value)
    }
}
