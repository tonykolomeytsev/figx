use std::fmt::{Debug, Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Pattern(lib_label::PatternError),
    Workspace(phase_loading::Error),
    Evaluation(phase_evaluation::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self, f)
    }
}
impl std::error::Error for Error {}

impl From<lib_label::PatternError> for Error {
    fn from(value: lib_label::PatternError) -> Self {
        Self::Pattern(value)
    }
}

impl From<phase_loading::Error> for Error {
    fn from(value: phase_loading::Error) -> Self {
        Self::Workspace(value)
    }
}

impl From<phase_evaluation::Error> for Error {
    fn from(value: phase_evaluation::Error) -> Self {
        Self::Evaluation(value)
    }
}
