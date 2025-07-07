use lib_label::PackageParsingError;
use std::path::PathBuf;
use toml_span::Span;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Internal(String),

    // region: Init
    InitInaccessibleCurrentWorkDir,
    InitNotInWorkspace,
    // endregion: Init

    // region: Workspace
    WorkspaceRead(std::io::Error),
    WorkspaceParse(toml_span::DeserError, PathBuf),
    WorkspaceRemoteNoAccessToken(String, PathBuf, Span),
    WorkspaceRemoteEmptyKeychain(String, PathBuf, Span),
    WorkspaceRemoteKeychainError(lib_auth::Error),
    // endregion: Workspace

    // region: FigFiles
    FigTraversing(String),
    FigRead(std::io::Error),
    FigParse(toml_span::DeserError, PathBuf),
    FigInvalidPackage(PackageParsingError),
    // endregion: FigFiles
}

// region: Internal

impl Error {
    pub fn internal(val: impl std::fmt::Display) -> Self {
        Self::Internal(val.to_string())
    }
}

impl From<&str> for Error {
    fn from(val: &str) -> Self {
        Self::Internal(val.to_string())
    }
}

impl From<ignore::Error> for Error {
    fn from(value: ignore::Error) -> Self {
        Self::Internal(value.to_string())
    }
}

// endregion: Internal

// region: Error Boilerplate

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

// endregion: Error Boilerplate
