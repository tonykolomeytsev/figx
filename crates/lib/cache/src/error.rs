pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Internal(String),
    Initialization(String),
    SurrealKV(String),
    Serialization(String),
    Deserialization(String),
    MissingRequiredValue(String),
}

impl Error {
    pub fn initialization(e: impl std::fmt::Display) -> Self {
        Self::Initialization(e.to_string())
    }

    pub fn serialization(e: impl std::fmt::Display) -> Self {
        Self::Serialization(e.to_string())
    }

    pub fn deserialization(e: impl std::fmt::Display) -> Self {
        Self::Deserialization(e.to_string())
    }
}

impl From<surrealkv::Error> for Error {
    fn from(value: surrealkv::Error) -> Self {
        Self::SurrealKV(value.to_string())
    }
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            Internal(msg) => write!(f, "{msg}"),
            Initialization(msg) => write!(f, "{msg}"),
            SurrealKV(msg) => write!(f, "surrealkv error: {msg}"),
            Serialization(msg) => write!(f, "serialization error: {msg}"),
            Deserialization(msg) => write!(f, "deserialization error: {msg}"),
            MissingRequiredValue(key) => write!(f, "missing required value: key={key}")
        }
    }
}
