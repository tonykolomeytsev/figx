use std::fmt::{Debug, Display};

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(pub keyring::Error);

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "auth error: {}", self.0)
    }
}
impl std::error::Error for Error {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        Some(&self.0)
    }
}

impl From<keyring::Error> for Error {
    fn from(value: keyring::Error) -> Self {
        Self(value)
    }
}