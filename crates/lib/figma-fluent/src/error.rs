pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error(pub ureq::Error);

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<ureq::Error> for Error {
    fn from(value: ureq::Error) -> Self {
        Self(value)
    }
}
