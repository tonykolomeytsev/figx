pub type Result<T> = ::std::result::Result<T, Error>;

pub enum Error {
    PatternError(lib_label::PatternError),
    WorkspaceError(phase_loading::Error),
    IO(std::io::Error),
}

impl From<lib_label::PatternError> for Error {
    fn from(value: lib_label::PatternError) -> Self {
        Self::PatternError(value)
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