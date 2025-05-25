use derive_more::From;
use lib_label::{NameParsingError, PackageParsingError};

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug, From)]
pub enum Error {
    #[from]
    Internal(String),

    // region: Init
    InitInaccessibleCurrentWorkDir,
    InitNotInWorkspace,

    // endregion: Init

    // region: Workspace
    WorkspaceRead(std::io::Error),
    WorkspaceParse(toml::de::Error),
    WorkspaceNoRemotes,
    WorkspaceRemoteNoAccessToken(String),
    WorkspaceMoreThanOneDefaultRemotes,
    WorkspaceAtLeastOneDefaultRemote,
    WorkspaceRemoteWithEmptyNodeId,
    WorkspaceInvalidProfileToExtend(String, String),

    // endregion: Workspace

    // region: FigFiles
    FigTraversing(String),
    FigRead(std::io::Error),
    FigParse(toml::de::Error),
    FigInvalidResourceName(NameParsingError),
    FigInvalidPackage(PackageParsingError),
    FigInvalidRemoteName(String),
    FigInvalidProfileName(String),
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
