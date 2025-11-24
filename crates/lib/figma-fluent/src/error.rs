pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Ureq(ureq::Error),
    RateLimit {
        retry_after_sec: u32,
        figma_plan_tier: String,
        figma_limit_type: String,
    },
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ureq(e) => write!(f, "{e}"),
            Self::RateLimit {
                retry_after_sec,
                figma_plan_tier,
                figma_limit_type,
            } => write!(
                f,
                "rate limit: retry after {retry_after_sec}s, (tier={figma_plan_tier}, type={figma_limit_type})"
            ),
        }
    }
}

impl From<ureq::Error> for Error {
    fn from(value: ureq::Error) -> Self {
        Self::Ureq(value)
    }
}
