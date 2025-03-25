use std::str::FromStr;

pub type TargetName = Name;
pub type ResourceName = Name;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Name(String);

impl FromStr for Name {
    type Err = NameParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let only_allowed_chars = s
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
        if !only_allowed_chars {
            return Err(NameParsingError(s.to_string()));
        }
        Ok(Name(s.to_string()))
    }
}

impl std::fmt::Display for TargetName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for TargetName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// region: Error

#[derive(Debug)]
pub struct NameParsingError(pub String);
impl std::error::Error for NameParsingError {}
impl std::fmt::Display for NameParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

// endregion: Error

#[cfg(test)]
#[allow(unused_imports, non_snake_case)]
mod test {
    use super::*;

    #[test]
    fn display_name__EXPECT__predictable_result() {
        assert_eq!("ic_star", Name::from_str("ic_star").unwrap().as_ref())
    }

    #[test]
    fn parse_valid_name__EXPECT__ok() {
        assert_eq!("ic_star", Name::from_str("ic_star").unwrap().as_ref());
        assert_eq!("ic_home", Name::from_str("ic_home").unwrap().as_ref());
        assert_eq!(
            "ic-burger-24",
            Name::from_str("ic-burger-24").unwrap().as_ref()
        );
    }

    #[test]
    fn parse_invalid_name__EXPECT__err() {
        assert!(Name::from_str("ic/star").is_err());
        assert!(Name::from_str("ic#star").is_err());
        assert!(Name::from_str(".ic_star").is_err());
    }
}
