use std::fmt::{Debug, Display};

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    ServerCreation(String),
    Io(std::io::Error),
    Auth(lib_auth::Error),
    Custom(String),
}

impl Error {
    pub fn server_creation(err: impl Display) -> Self {
        Self::ServerCreation(err.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ServerCreation(s) => write!(f, "unable to create server: {s}"),
            Self::Io(e) => write!(f, "{e}"),
            Self::Auth(e) => write!(f, "{e}"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}
impl std::error::Error for Error {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            _ => todo!(),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<lib_auth::Error> for Error {
    fn from(value: lib_auth::Error) -> Self {
        Self::Auth(value)
    }
}
