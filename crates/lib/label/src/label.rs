use crate::{Name, NameParsingError, Package, PackageParsingError};
use std::{path::Path, str::FromStr};

/// A fully-qualified identifier of a resource inside a fig-package.
///
/// Similar in concept to labels used in Bazel or Buck2, this struct identifies a target
/// within a workspace by its package path and local name.
///
/// # Examples
///
/// A label like `foo/bar:lib` corresponds to:
/// - `package`: `"foo/bar"`
/// - `name`: `"lib"`
///
/// The label uniquely identifies a resource named `"lib"` inside the `foo/bar` package.
///
/// # Fields
///
/// - `package`: The relative path to the directory containing the fig-package.
/// - `name`: The resource name inside the package.
///
#[derive(PartialEq, Eq, Hash, Clone)]
#[non_exhaustive]
pub struct Label {
    /// Path of directory with fig-file, e.g. "foo/bar"
    pub package: Package,
    /// Name of resource inside fig-file
    pub name: Name,
}

impl Label {
    pub fn from_package_and_name<P, S>(package: P, name: S) -> Result<Self, LabelError>
    where
        P: AsRef<Path>,
        S: AsRef<str>,
    {
        Ok(Label {
            package: Package::with_path(package)?,
            name: Name::from_str(name.as_ref())?,
        })
    }
}

impl From<(Package, Name)> for Label {
    fn from(value: (Package, Name)) -> Self {
        Self {
            package: value.0,
            name: value.1,
        }
    }
}

impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.package, self.name)
    }
}

impl std::fmt::Debug for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self, f)
    }
}

// region: Error

#[derive(Debug)]
pub enum LabelError {
    BadPackage(String),
    BadName(String),
}
impl std::error::Error for LabelError {}
impl std::fmt::Display for LabelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self, f)
    }
}
impl From<NameParsingError> for LabelError {
    fn from(value: NameParsingError) -> Self {
        Self::BadName(value.0)
    }
}
impl From<PackageParsingError> for LabelError {
    fn from(value: PackageParsingError) -> Self {
        Self::BadPackage(value.0)
    }
}

// endregion: Error

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_create_from_package_and_name() {
        // Given
        let package = Package(PathBuf::from("path/to/package"));
        let name = Name("res_name".to_string());

        // When
        let label: Label = (package, name).into();

        // Then
        assert_eq!("//path/to/package:res_name", label.to_string());
    }

    #[test]
    fn test_label_display() {
        // Given
        let package = Package(PathBuf::from("path/to/package"));
        let name = Name("res_name".to_string());

        // When
        let label: Label = (package, name).into();

        // Then
        assert_eq!("//path/to/package:res_name", format!("{label}"));
    }

    #[test]
    fn test_label_debug() {
        // Given
        let package = Package(PathBuf::from("path/to/package"));
        let name = Name("res_name".to_string());

        // When
        let label: Label = (package, name).into();

        // Then
        assert_eq!("//path/to/package:res_name", format!("{label:?}"));
    }

    #[test]
    fn test_invalid_package() {
        // Given
        let package = "/path/to/package";
        let name = "valid_name";

        // When
        let result = Label::from_package_and_name(package, name);

        // Then
        assert!(matches!(result, Err(LabelError::BadPackage(_))));
    }

    #[test]
    fn test_invalid_name() {
        // Given
        let package = "path/to/package";
        let name = "in§valid_name";

        // When
        let result = Label::from_package_and_name(package, name);

        // Then
        assert!(matches!(result, Err(LabelError::BadName(_))));
    }
}