use std::{
    ops::Deref,
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Package(PathBuf);

impl Package {
    pub fn with_path<P>(path: P) -> Result<Self, PackageParsingError>
    where
        P: AsRef<Path>,
    {
        // Guard expr
        let _ = path.as_ref().to_str().ok_or(PackageParsingError(
            path.as_ref().to_string_lossy().to_string(),
        ));

        let full_path = path.as_ref().to_path_buf();
        for part in full_path.iter() {
            let only_allowed_chars = part
                .to_string_lossy()
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
            if !only_allowed_chars {
                return Err(PackageParsingError(
                    path.as_ref().to_string_lossy().to_string(),
                ));
            }
        }
        Ok(Package(full_path))
    }

    pub fn empty() -> Self {
        Self(PathBuf::new())
    }
}

impl std::fmt::Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "//{}", self.0.display())
    }
}

impl AsRef<Path> for Package {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl AsRef<str> for Package {
    fn as_ref(&self) -> &str {
        self.0
            .to_str()
            .expect("Always valid path inside Package value-objects")
    }
}

impl Deref for Package {
    type Target = Path;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


// region: Error

#[derive(Debug)]
pub struct PackageParsingError(pub String);
impl std::error::Error for PackageParsingError {}
impl std::fmt::Display for PackageParsingError {
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
    fn display_package__EXPECT__predictable_result() {
        assert_eq!(
            "//foo/bar",
            Package::with_path(PathBuf::from("foo/bar"))
                .unwrap()
                .to_string(),
        )
    }

    #[test]
    fn parse_valid_package__EXPECT__ok() {
        assert_eq!(
            Package(PathBuf::from("foo/bar")),
            Package::with_path("foo/bar").unwrap(),
        );
        assert_eq!(
            Package(PathBuf::from("foo-123/bar_x")),
            Package::with_path("foo-123/bar_x").unwrap(),
        );
    }

    #[test]
    fn parse_invalid_packages__EXPECT__err() {
        assert!(Package::with_path("/foo/bar").is_err());
        assert!(Package::with_path("foo/.bar").is_err());
        assert!(Package::with_path("#foo/bar").is_err());
        assert!(Package::with_path("foo/+bar").is_err());
    }
}
