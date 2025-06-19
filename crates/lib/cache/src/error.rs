pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Internal(String),
    Initialization(String),
    SurrealKV(Option<String>, surrealkv::Error),
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

    pub fn with_context(self, ctx: impl std::fmt::Display) -> Self {
        match self {
            Self::Internal(e) => Self::Internal(format!("{ctx}: {e}")),
            Self::Initialization(e) => Self::Initialization(format!("{ctx}: {e}")),
            Self::SurrealKV(_, e) => Self::SurrealKV(Some(format!("{ctx}")), e),
            Self::Serialization(e) => Self::Serialization(format!("{ctx}: {e}")),
            Self::Deserialization(e) => Self::Deserialization(format!("{ctx}: {e}")),
            Self::MissingRequiredValue(e) => Self::MissingRequiredValue(format!("{ctx}: {e}")),
        }
    }
}

impl From<surrealkv::Error> for Error {
    fn from(value: surrealkv::Error) -> Self {
        Self::SurrealKV(None, value)
    }
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            Internal(msg) => write!(f, "{msg}"),
            Initialization(msg) => write!(f, "{msg}"),
            SurrealKV(ctx, msg) => write!(
                f,
                "{}surrealkv error: {msg}",
                ctx.as_ref().map(|it| format!("{it}: ")).unwrap_or_default()
            ),
            Serialization(msg) => write!(f, "serialization error: {msg}"),
            Deserialization(msg) => write!(f, "deserialization error: {msg}"),
            MissingRequiredValue(key) => write!(f, "missing required value: key={key}"),
        }
    }
}
