pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Http(ureq::Error),
    RateLimit(RateLimitError),
}

#[derive(Debug)]
pub struct RateLimitError {
    pub retry_after_sec: u32,
    pub figma_plan_tier: String,
    pub figma_limit_type: String,
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(e) => write!(f, "{e}"),
            Self::RateLimit(e) => write!(
                f,
                "rate limit: retry after {}s, (tier={}, type={})",
                e.retry_after_sec, e.figma_plan_tier, e.figma_limit_type
            ),
        }
    }
}

impl From<ureq::Error> for Error {
    fn from(value: ureq::Error) -> Self {
        Self::Http(value)
    }
}

impl From<RateLimitError> for Error {
    fn from(value: RateLimitError) -> Self {
        Self::RateLimit(value)
    }
}
