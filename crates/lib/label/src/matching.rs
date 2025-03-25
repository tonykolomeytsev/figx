use std::{
    ops::Deref,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{Label, Package};

/// A user-supplied pattern used to match one or more [`Label`]s.
///
/// This is the main structure representing parsed patterns like:
/// - `//foo/bar:baz`
/// - `//foo/bar:*`
/// - `//foo/...`
///
/// Patterns can refer to:
/// - specific targets,
/// - wildcard targets within a specific package,
/// - or all targets within a subtree of packages.
///
/// # Example
///
/// ```
/// # use lib_label::PackagePattern;
/// # use lib_label::TargetPattern;
/// # use lib_label::LabelPatternImpl;
/// LabelPatternImpl {
///     package: PackagePattern::Wildcard("foo".into()),
///     target: TargetPattern::All,
///     absolute: true,
///     negative: false,
/// };
/// ```
/// corresponds to the pattern `//foo/...:*`
#[derive(Debug, PartialEq, Clone)]
pub enum LabelPattern {
    Single(LabelPatternImpl),
    Composed(Vec<LabelPatternImpl>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct LabelPatternImpl {
    pub package: PackagePattern,
    pub target: TargetPattern,
    pub absolute: bool,
    pub negative: bool,
}

/// A pattern used to match one or more packages.
///
/// Corresponds to the left-hand side of a label pattern (before the colon),
/// e.g. `foo/bar` or `foo/...`.
#[derive(Debug, PartialEq, Clone)]
pub enum PackagePattern {
    /// Matches all packages in the workspace (e.g. `//...`)
    All,

    /// Matches exactly one package at the specified path (e.g. `//foo/bar`)
    Exact(PathBuf),

    /// Matches a package and all its subpackages recursively (e.g. `//foo/...`)
    Wildcard(PathBuf),
}

/// A pattern used to match one or more targets within a package.
///
/// Corresponds to the right-hand side of a label pattern (after the colon),
/// e.g. `:lib` or `:*`.
#[derive(Debug, PartialEq, Clone)]
pub enum TargetPattern {
    /// Matches all targets in the package (e.g. `:*`, `:all`)
    All,

    /// Matches a single target by exact name (e.g. `:lib`)
    Exact(String),

    /// Matches multiple targets by wildcard (e.g. `:lib*`)
    Wildcard(String),
}

impl FromStr for LabelPattern {
    type Err = PatternError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::Single(parse_pattern(s)?))
    }
}

impl TryFrom<Vec<String>> for LabelPattern {
    type Error = crate::PatternError;

    fn try_from(value: Vec<String>) -> Result<Self, Self::Error> {
        let patterns = value
            .iter()
            .map(|it| parse_pattern(it.as_str()))
            .collect::<Result<_, Self::Error>>()?;
        Ok(Self::Composed(patterns))
    }
}

// region: Error

#[derive(Debug)]
pub enum PatternError {
    BadPackage(String, String),
    BadTarget(String, String),
}

impl std::error::Error for PatternError {}
impl std::fmt::Display for PatternError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

// endregion: Error

fn parse_pattern(pattern: &str) -> Result<LabelPatternImpl, PatternError> {
    let (pattern, negative) = if let Some(stripped) = pattern.strip_prefix('-') {
        (stripped, true)
    } else {
        (pattern, false)
    };

    if pattern == "//..." {
        return Ok(LabelPatternImpl {
            package: PackagePattern::All,
            target: TargetPattern::All,
            absolute: true,
            negative,
        });
    }

    if let Some((package, target)) = pattern.rsplit_once(':') {
        ensure_valid_package(package, pattern)?;
        ensure_valid_target(target, pattern)?;
        let is_absolute_path = package.starts_with("//");
        let package_pattern = match package.trim_start_matches("//") {
            "..." => PackagePattern::All,
            p if p.contains("...") => PackagePattern::Wildcard(PathBuf::from(p)),
            p => PackagePattern::Exact(PathBuf::from(p)),
        };
        let target_pattern = match target {
            "*" | "all" => TargetPattern::All,
            t if t.contains("*") => TargetPattern::Wildcard(t.to_string()),
            t => TargetPattern::Exact(t.to_string()),
        };
        Ok(LabelPatternImpl {
            package: package_pattern,
            target: target_pattern,
            absolute: is_absolute_path,
            negative,
        })
    } else {
        let package = pattern;
        ensure_valid_package(package, pattern)?;
        let is_absolute_path = package.starts_with("//");
        let package_pattern = match package.trim_start_matches("//") {
            "..." => PackagePattern::All,
            p if p.contains("...") => PackagePattern::Wildcard(PathBuf::from(p)),
            p => PackagePattern::Exact(PathBuf::from(p)),
        };
        Ok(LabelPatternImpl {
            package: package_pattern,
            target: TargetPattern::All,
            absolute: is_absolute_path,
            negative,
        })
    }
}

fn ensure_valid_package(package: &str, pattern: &str) -> Result<(), PatternError> {
    let normalized_path = package.trim_start_matches("-").trim_start_matches("//");
    let full_path = PathBuf::from(normalized_path);
    for part in full_path.iter() {
        if part == "..." {
            continue;
        }
        let only_allowed_chars = part
            .to_string_lossy()
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
        if !only_allowed_chars {
            return Err(PatternError::BadPackage(
                pattern.to_string(),
                package.to_string(),
            ));
        }
    }
    Ok(())
}

fn ensure_valid_target(target: &str, pattern: &str) -> Result<(), PatternError> {
    let only_allowed_chars = target
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '*');
    if !only_allowed_chars || target.is_empty() {
        return Err(PatternError::BadTarget(
            pattern.to_string(),
            target.to_string(),
        ));
    }
    Ok(())
}

/// Checks whether a given [`Label`] matches the specified [`LabelPattern`].
///
/// This function determines if a `label` should be selected for processing
/// based on a user-specified `pattern`, optionally relative to a `current_dir`.
///
/// # Examples
///
/// ```
/// # use std::path::PathBuf;
/// # use lib_label::matches;
/// # use lib_label::Label;
/// # use lib_label::LabelPattern;
/// # use std::str::FromStr;
/// let pattern = LabelPattern::from_str("//foo/...:bar").unwrap();
/// let label = Label::from_package_and_name("foo/xyz/zyx", "bar").unwrap();
/// assert!(matches(&pattern, &label, &PathBuf::new()));
/// ```
pub fn matches(pattern: &LabelPattern, label: &Label, current_dir: &Path) -> bool {
    match pattern {
        LabelPattern::Single(pattern) => {
            let result = matches_impl(pattern, label, current_dir);
            if pattern.negative { !result } else { result }
        }
        LabelPattern::Composed(patterns) => {
            // For composed patterns, we need to check if any positive patterns match
            // and none of the negative patterns match
            let mut positive_match = false;
            let mut negative_match = false;
            for pattern in patterns {
                let matches = matches_impl(pattern, label, current_dir);
                if pattern.negative {
                    negative_match |= matches;
                } else {
                    positive_match |= matches;
                }
            }
            positive_match && !negative_match
        }
    }
}

fn matches_impl(pattern: &LabelPatternImpl, label: &Label, current_dir: &Path) -> bool {
    if !package_matches_impl(pattern, &label.package, current_dir) {
        return false;
    }

    match &pattern.target {
        TargetPattern::Exact(target) => label.name.as_ref() == target,
        TargetPattern::All => true,
        TargetPattern::Wildcard(target) => fast_glob::glob_match(target, label.name.as_ref()),
    }
}

pub fn package_matches(pattern: &LabelPattern, package: &Package, current_dir: &Path) -> bool {
    match pattern {
        LabelPattern::Single(pattern) => {
            let result = package_matches_impl(pattern, package, current_dir);
            if pattern.negative { !result } else { result }
        }
        LabelPattern::Composed(patterns) => {
            // For composed patterns, we need to check if any positive patterns match
            // and none of the negative patterns match
            let mut positive_match = false;
            let mut negative_match = false;
            for pattern in patterns {
                let matches = package_matches_impl(pattern, package, current_dir);
                if pattern.negative {
                    negative_match |= matches;
                } else {
                    positive_match |= matches;
                }
            }
            positive_match && !negative_match
        }
    }
}

fn package_matches_impl(pattern: &LabelPatternImpl, package: &Package, current_dir: &Path) -> bool {
    match (pattern.absolute, &pattern.package) {
        (true, PackagePattern::Exact(pattern)) => pattern == package.deref(),
        (false, PackagePattern::Exact(pattern)) => current_dir.join(pattern) == package.deref(),

        (true, PackagePattern::All) => true,
        (false, PackagePattern::All) => package.starts_with(current_dir),

        (true, PackagePattern::Wildcard(pattern)) => {
            let glob = pattern
                .to_str()
                .expect("always valid unicode here")
                .replace("...", "**");
            let path = package.to_str().expect("always valid unicode here");
            fast_glob::glob_match(glob, path)
        }
        (false, PackagePattern::Wildcard(pattern)) => {
            let absolute_package = current_dir.join(pattern);
            let glob = absolute_package
                .to_str()
                .expect("always valid unicode here")
                .replace("...", "**");
            let path = package.to_str().expect("always valid unicode here");
            fast_glob::glob_match(glob, path)
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    // region: parsing tests

    #[test]
    #[allow(non_snake_case)]
    fn parse_absolute_exact_pattern__EXPECT__ok() {
        assert_eq!(
            LabelPattern::from_str("//foo/bar:wiz").unwrap(),
            LabelPattern::Single(LabelPatternImpl {
                absolute: true,
                negative: false,
                package: PackagePattern::Exact(PathBuf::from("foo/bar")),
                target: TargetPattern::Exact("wiz".to_string()),
            }),
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn parse_absolute_negative_exact_pattern__EXPECT__ok() {
        assert_eq!(
            LabelPattern::from_str("-//foo/bar:wiz").unwrap(),
            LabelPattern::Single(LabelPatternImpl {
                absolute: true,
                negative: true,
                package: PackagePattern::Exact(PathBuf::from("foo/bar")),
                target: TargetPattern::Exact("wiz".to_string()),
            }),
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn parse_relative_exact_pattern__EXPECT__ok() {
        assert_eq!(
            LabelPattern::from_str("foo/bar:wiz").unwrap(),
            LabelPattern::Single(LabelPatternImpl {
                absolute: false,
                negative: false,
                package: PackagePattern::Exact(PathBuf::from("foo/bar")),
                target: TargetPattern::Exact("wiz".to_string()),
            }),
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn parse_relative_negative_exact_pattern__EXPECT__ok() {
        assert_eq!(
            LabelPattern::from_str("-foo/bar:wiz").unwrap(),
            LabelPattern::Single(LabelPatternImpl {
                absolute: false,
                negative: true,
                package: PackagePattern::Exact(PathBuf::from("foo/bar")),
                target: TargetPattern::Exact("wiz".to_string()),
            }),
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn parse_absolute_pattern_with_exact_package_only__EXPECT__ok() {
        assert_eq!(
            LabelPattern::from_str("//foo/bar").unwrap(),
            LabelPattern::Single(LabelPatternImpl {
                absolute: true,
                negative: false,
                package: PackagePattern::Exact(PathBuf::from("foo/bar")),
                target: TargetPattern::All,
            }),
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn parse_absolute_negative_pattern_with_exact_package_only__EXPECT__ok() {
        assert_eq!(
            LabelPattern::from_str("-//foo/bar").unwrap(),
            LabelPattern::Single(LabelPatternImpl {
                absolute: true,
                negative: true,
                package: PackagePattern::Exact(PathBuf::from("foo/bar")),
                target: TargetPattern::All,
            }),
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn parse_relative_pattern_with_exact_package_only__EXPECT__ok() {
        assert_eq!(
            LabelPattern::from_str("foo/bar").unwrap(),
            LabelPattern::Single(LabelPatternImpl {
                absolute: false,
                negative: false,
                package: PackagePattern::Exact(PathBuf::from("foo/bar")),
                target: TargetPattern::All,
            }),
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn parse_relative_negative_pattern_with_exact_package_only__EXPECT__ok() {
        assert_eq!(
            LabelPattern::from_str("-foo/bar").unwrap(),
            LabelPattern::Single(LabelPatternImpl {
                absolute: false,
                negative: true,
                package: PackagePattern::Exact(PathBuf::from("foo/bar")),
                target: TargetPattern::All,
            }),
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn parse_absolute_pattern_with_exact_package_and_all_targets__EXPECT__ok() {
        assert_eq!(
            LabelPattern::from_str("//foo/bar:*").unwrap(),
            LabelPattern::Single(LabelPatternImpl {
                absolute: true,
                negative: false,
                package: PackagePattern::Exact(PathBuf::from("foo/bar")),
                target: TargetPattern::All,
            }),
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn parse_relative_pattern_with_exact_package_and_all_targets__EXPECT__ok() {
        assert_eq!(
            LabelPattern::from_str("foo/bar:*").unwrap(),
            LabelPattern::Single(LabelPatternImpl {
                absolute: false,
                negative: false,
                package: PackagePattern::Exact(PathBuf::from("foo/bar")),
                target: TargetPattern::All,
            }),
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn parse_absolute_pattern_with_exact_package_and_glob_target__EXPECT__ok() {
        assert_eq!(
            LabelPattern::from_str("//foo/bar:ic_*").unwrap(),
            LabelPattern::Single(LabelPatternImpl {
                absolute: true,
                negative: false,
                package: PackagePattern::Exact(PathBuf::from("foo/bar")),
                target: TargetPattern::Wildcard("ic_*".to_string()),
            }),
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn parse_relative_pattern_with_exact_package_and_glob_target__EXPECT__ok() {
        assert_eq!(
            LabelPattern::from_str("foo/bar:ic_*").unwrap(),
            LabelPattern::Single(LabelPatternImpl {
                absolute: false,
                negative: false,
                package: PackagePattern::Exact(PathBuf::from("foo/bar")),
                target: TargetPattern::Wildcard("ic_*".to_string()),
            }),
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn parse_absolute_pattern_with_glob_package_and_glob_target__EXPECT__ok() {
        assert_eq!(
            LabelPattern::from_str("//foo/...:ic_*").unwrap(),
            LabelPattern::Single(LabelPatternImpl {
                absolute: true,
                negative: false,
                package: PackagePattern::Wildcard(PathBuf::from("foo/...")),
                target: TargetPattern::Wildcard("ic_*".to_string()),
            }),
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn parse_relative_pattern_with_glob_package_and_glob_target__EXPECT__ok() {
        assert_eq!(
            LabelPattern::from_str("foo/...:ic_*").unwrap(),
            LabelPattern::Single(LabelPatternImpl {
                absolute: false,
                negative: false,
                package: PackagePattern::Wildcard(PathBuf::from("foo/...")),
                target: TargetPattern::Wildcard("ic_*".to_string()),
            }),
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn parse_absolute_pattern_with_glob_package__EXPECT__ok() {
        assert_eq!(
            LabelPattern::from_str("//foo/...").unwrap(),
            LabelPattern::Single(LabelPatternImpl {
                absolute: true,
                negative: false,
                package: PackagePattern::Wildcard(PathBuf::from("foo/...")),
                target: TargetPattern::All,
            }),
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn parse_relative_pattern_with_glob_package__EXPECT__ok() {
        assert_eq!(
            LabelPattern::from_str("foo/...").unwrap(),
            LabelPattern::Single(LabelPatternImpl {
                absolute: false,
                negative: false,
                package: PackagePattern::Wildcard(PathBuf::from("foo/...")),
                target: TargetPattern::All,
            }),
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn parse_absolute_recursive_pattern__EXPECT__ok() {
        assert_eq!(
            LabelPattern::from_str("//...").unwrap(),
            LabelPattern::Single(LabelPatternImpl {
                absolute: true,
                negative: false,
                package: PackagePattern::All,
                target: TargetPattern::All,
            }),
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn parse_relative_recursive_pattern__EXPECT__ok() {
        assert_eq!(
            LabelPattern::from_str("...").unwrap(),
            LabelPattern::Single(LabelPatternImpl {
                absolute: false,
                negative: false,
                package: PackagePattern::All,
                target: TargetPattern::All,
            }),
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn parse_relative_negative_recursive_pattern__EXPECT__ok() {
        assert_eq!(
            LabelPattern::from_str("-...").unwrap(),
            LabelPattern::Single(LabelPatternImpl {
                absolute: false,
                negative: true,
                package: PackagePattern::All,
                target: TargetPattern::All,
            }),
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn parse_relative_short_package_short_target__EXPECT__ok() {
        assert_eq!(
            LabelPattern::from_str("foo:bar").unwrap(),
            LabelPattern::Single(LabelPatternImpl {
                absolute: false,
                negative: false,
                package: PackagePattern::Exact(PathBuf::from("foo")),
                target: TargetPattern::Exact("bar".to_string()),
            }),
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn parse_relative_negative_short_package_short_target__EXPECT__ok() {
        assert_eq!(
            LabelPattern::from_str("-foo:bar").unwrap(),
            LabelPattern::Single(LabelPatternImpl {
                absolute: false,
                negative: true,
                package: PackagePattern::Exact(PathBuf::from("foo")),
                target: TargetPattern::Exact("bar".to_string()),
            }),
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn parse_relative_target_only__EXPECT__ok() {
        assert_eq!(
            LabelPattern::from_str(":bar").unwrap(),
            LabelPattern::Single(LabelPatternImpl {
                absolute: false,
                negative: false,
                package: PackagePattern::Exact(PathBuf::new()),
                target: TargetPattern::Exact("bar".to_string()),
            }),
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn parse_relative_negative_target_only__EXPECT__ok() {
        assert_eq!(
            LabelPattern::from_str("-:bar").unwrap(),
            LabelPattern::Single(LabelPatternImpl {
                absolute: false,
                negative: true,
                package: PackagePattern::Exact(PathBuf::new()),
                target: TargetPattern::Exact("bar".to_string()),
            }),
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn parse_relative_all_wildcard_targets__EXPECT__ok() {
        assert_eq!(
            LabelPattern::from_str(":*").unwrap(),
            LabelPattern::Single(LabelPatternImpl {
                absolute: false,
                negative: false,
                package: PackagePattern::Exact(PathBuf::new()),
                target: TargetPattern::All,
            }),
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn parse_relative_negative_all_wildcard_targets__EXPECT__ok() {
        assert_eq!(
            LabelPattern::from_str("-:*").unwrap(),
            LabelPattern::Single(LabelPatternImpl {
                absolute: false,
                negative: true,
                package: PackagePattern::Exact(PathBuf::new()),
                target: TargetPattern::All,
            }),
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn parse_relative_all_word_targets__EXPECT__ok() {
        assert_eq!(
            LabelPattern::from_str(":all").unwrap(),
            LabelPattern::Single(LabelPatternImpl {
                absolute: false,
                negative: false,
                package: PackagePattern::Exact(PathBuf::new()),
                target: TargetPattern::All,
            }),
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn parse_relative_glob_target__EXPECT__ok() {
        assert_eq!(
            LabelPattern::from_str(":ill_*").unwrap(),
            LabelPattern::Single(LabelPatternImpl {
                absolute: false,
                negative: false,
                package: PackagePattern::Exact(PathBuf::new()),
                target: TargetPattern::Wildcard("ill_*".to_string()),
            }),
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn parse_absolute_glob_target__EXPECT__ok() {
        assert_eq!(
            LabelPattern::from_str("//:ill_*").unwrap(),
            LabelPattern::Single(LabelPatternImpl {
                absolute: true,
                negative: false,
                package: PackagePattern::Exact(PathBuf::new()),
                target: TargetPattern::Wildcard("ill_*".to_string()),
            }),
        );
    }

    #[test]
    #[allow(non_snake_case)]
    fn parse_invalid_packages__EXPECT__error() {
        assert!(LabelPattern::from_str("//foo/bar*").is_err());
        assert!(LabelPattern::from_str("*foo/bar").is_err());
        assert!(LabelPattern::from_str("../bar:xyz").is_err());
        assert!(LabelPattern::from_str(":...").is_err());
    }

    // endregion: parsing tests

    #[test]
    fn matches_all_targets_in_absolute_package__EXPECT__ok() {
        let p = LabelPattern::from_str("//foo/bar").unwrap();
        assert!(matches(&p, &target("//foo/bar:xyz"), &PathBuf::new()));
        assert!(matches(&p, &target("//foo/bar:abc"), &PathBuf::new()));
        assert!(matches(&p, &target("//foo/bar:baz"), &PathBuf::new()));
        assert!(!matches(&p, &target("//foo/baz:xyz"), &PathBuf::new()));
        assert!(!matches(&p, &target("//foo/baz:abc"), &PathBuf::new()));
        assert!(!matches(&p, &target("//foo/baz:baz"), &PathBuf::new()));
        // Negative
        let p = LabelPattern::from_str("-//foo/bar").unwrap();
        assert!(!matches(&p, &target("//foo/bar:xyz"), &PathBuf::new()));
        assert!(!matches(&p, &target("//foo/bar:abc"), &PathBuf::new()));
        assert!(!matches(&p, &target("//foo/bar:baz"), &PathBuf::new()));
        assert!(matches(&p, &target("//foo/baz:xyz"), &PathBuf::new()));
        assert!(matches(&p, &target("//foo/baz:abc"), &PathBuf::new()));
        assert!(matches(&p, &target("//foo/baz:baz"), &PathBuf::new()));
    }

    #[test]
    fn matches_glob_targets_in_absolute_package__EXPECT__ok() {
        let p = LabelPattern::from_str("//foo/bar:*").unwrap();
        assert!(matches(&p, &target("//foo/bar:xyz"), &PathBuf::new()));
        assert!(matches(&p, &target("//foo/bar:abc"), &PathBuf::new()));
        assert!(matches(&p, &target("//foo/bar:baz"), &PathBuf::new()));
        assert!(!matches(&p, &target("//foo/baz:xyz"), &PathBuf::new()));
        assert!(!matches(&p, &target("//foo/baz:abc"), &PathBuf::new()));
        assert!(!matches(&p, &target("//foo/baz:baz"), &PathBuf::new()));
        // Negative
        let p = LabelPattern::from_str("-//foo/bar:*").unwrap();
        assert!(!matches(&p, &target("//foo/bar:xyz"), &PathBuf::new()));
        assert!(!matches(&p, &target("//foo/bar:abc"), &PathBuf::new()));
        assert!(!matches(&p, &target("//foo/bar:baz"), &PathBuf::new()));
        assert!(matches(&p, &target("//foo/baz:xyz"), &PathBuf::new()));
        assert!(matches(&p, &target("//foo/baz:abc"), &PathBuf::new()));
        assert!(matches(&p, &target("//foo/baz:baz"), &PathBuf::new()));
    }

    #[test]
    fn matches_all_targets_in_absolute_glob_package__EXPECT__ok() {
        let p = LabelPattern::from_str("//foo/...:*").unwrap();
        assert!(matches(&p, &target("//foo/bar:xyz"), &PathBuf::new()));
        assert!(matches(&p, &target("//foo/fuz/baz:abc"), &PathBuf::new()));
        assert!(matches(
            &p,
            &target("//foo/buz/biz/nun:baz"),
            &PathBuf::new()
        ));
        assert!(!matches(&p, &target("//fee/bar:xyz"), &PathBuf::new()));
        assert!(!matches(&p, &target("//fee/fuz:abc"), &PathBuf::new()));
        assert!(!matches(&p, &target("//fee/buz:baz"), &PathBuf::new()));
        // Negative
        let p = LabelPattern::from_str("-//foo/...:*").unwrap();
        assert!(!matches(&p, &target("//foo/bar:xyz"), &PathBuf::new()));
        assert!(!matches(&p, &target("//foo/fuz/baz:abc"), &PathBuf::new()));
        assert!(!matches(
            &p,
            &target("//foo/buz/biz/nun:baz"),
            &PathBuf::new()
        ));
        assert!(matches(&p, &target("//fee/bar:xyz"), &PathBuf::new()));
        assert!(matches(&p, &target("//fee/fuz:abc"), &PathBuf::new()));
        assert!(matches(&p, &target("//fee/buz:baz"), &PathBuf::new()));
    }

    #[test]
    fn matches_all_targets_in_absolute_mid_glob_package__EXPECT__ok() {
        let p = LabelPattern::from_str("//foo/.../bar:*").unwrap();
        assert!(matches(&p, &target("//foo/abc/bar:xyz"), &PathBuf::new()));
        assert!(matches(&p, &target("//foo/fuz/bar:abc"), &PathBuf::new()));
        assert!(matches(&p, &target("//foo/buz/bar:baz"), &PathBuf::new()));
        assert!(!matches(&p, &target("//fe/abc/bar:xyz"), &PathBuf::new()));
        assert!(!matches(&p, &target("//fo/fuz/baz:abc"), &PathBuf::new()));
        assert!(!matches(&p, &target("//fw/buz/baw:baz"), &PathBuf::new()));
        // Negative
        let p = LabelPattern::from_str("-//foo/.../bar:*").unwrap();
        assert!(!matches(&p, &target("//foo/abc/bar:xyz"), &PathBuf::new()));
        assert!(!matches(&p, &target("//foo/fuz/bar:abc"), &PathBuf::new()));
        assert!(!matches(&p, &target("//foo/buz/bar:baz"), &PathBuf::new()));
        assert!(matches(&p, &target("//fe/abc/bar:xyz"), &PathBuf::new()));
        assert!(matches(&p, &target("//fo/fuz/baz:abc"), &PathBuf::new()));
        assert!(matches(&p, &target("//fw/buz/baw:baz"), &PathBuf::new()));
    }

    #[test]
    fn matches_all_targets_in_rel_package_no_cwd__EXPECT__ok() {
        let p = LabelPattern::from_str("foo/bar").unwrap();
        assert!(matches(&p, &target("//foo/bar:xyz"), &PathBuf::new()));
        assert!(matches(&p, &target("//foo/bar:abc"), &PathBuf::new()));
        assert!(matches(&p, &target("//foo/bar:baz"), &PathBuf::new()));
        assert!(!matches(&p, &target("//baz/bar:xyz"), &PathBuf::new()));
        assert!(!matches(&p, &target("//baz/bar:abc"), &PathBuf::new()));
        assert!(!matches(&p, &target("//baz/bar:baz"), &PathBuf::new()));
        // Negative
        let p = LabelPattern::from_str("-foo/bar").unwrap();
        assert!(!matches(&p, &target("//foo/bar:xyz"), &PathBuf::new()));
        assert!(!matches(&p, &target("//foo/bar:abc"), &PathBuf::new()));
        assert!(!matches(&p, &target("//foo/bar:baz"), &PathBuf::new()));
        assert!(matches(&p, &target("//baz/bar:xyz"), &PathBuf::new()));
        assert!(matches(&p, &target("//baz/bar:abc"), &PathBuf::new()));
        assert!(matches(&p, &target("//baz/bar:baz"), &PathBuf::new()));
    }

    #[test]
    fn matches_all_targets_in_rel_package_cwd__EXPECT__ok() {
        let p = LabelPattern::from_str("bar").unwrap();
        assert!(matches(&p, &target("//foo/bar:xyz"), &path("foo")));
        assert!(matches(&p, &target("//foo/bar:abc"), &path("foo")));
        assert!(matches(&p, &target("//foo/bar:baz"), &path("foo")));
        assert!(!matches(&p, &target("//fox/bar:xyz"), &path("foo")));
        assert!(!matches(&p, &target("//foz/bar:abc"), &path("foo")));
        assert!(!matches(&p, &target("//fov/bar:baz"), &path("foo")));
        // Negative
        let p = LabelPattern::from_str("bar").unwrap();
        assert!(matches(&p, &target("//foo/bar:xyz"), &path("foo")));
        assert!(matches(&p, &target("//foo/bar:abc"), &path("foo")));
        assert!(matches(&p, &target("//foo/bar:baz"), &path("foo")));
        assert!(!matches(&p, &target("//fox/bar:xyz"), &path("foo")));
        assert!(!matches(&p, &target("//foz/bar:abc"), &path("foo")));
        assert!(!matches(&p, &target("//fov/bar:baz"), &path("foo")));
    }

    #[test]
    fn matches_glob_targets_in_rel_package_no_cwd__EXPECT__ok() {
        let p = LabelPattern::from_str("foo/bar:*").unwrap();
        assert!(matches(&p, &target("//foo/bar:xyz"), &PathBuf::new()));
        assert!(matches(&p, &target("//foo/bar:abc"), &PathBuf::new()));
        assert!(matches(&p, &target("//foo/bar:baz"), &PathBuf::new()));
        assert!(!matches(&p, &target("//zoo/bar:xyz"), &PathBuf::new()));
        assert!(!matches(&p, &target("//doo/bar:abc"), &PathBuf::new()));
        assert!(!matches(&p, &target("//voo/bar:baz"), &PathBuf::new()));
        // Negative
        let p = LabelPattern::from_str("-foo/bar:*").unwrap();
        assert!(!matches(&p, &target("//foo/bar:xyz"), &PathBuf::new()));
        assert!(!matches(&p, &target("//foo/bar:abc"), &PathBuf::new()));
        assert!(!matches(&p, &target("//foo/bar:baz"), &PathBuf::new()));
        assert!(matches(&p, &target("//zoo/bar:xyz"), &PathBuf::new()));
        assert!(matches(&p, &target("//doo/bar:abc"), &PathBuf::new()));
        assert!(matches(&p, &target("//voo/bar:baz"), &PathBuf::new()));
    }

    #[test]
    fn matches_glob_targets_in_rel_package_cwd__EXPECT__ok() {
        let p = LabelPattern::from_str("bar:*").unwrap();
        assert!(matches(&p, &target("//foo/bar:xyz"), &path("foo")));
        assert!(matches(&p, &target("//foo/bar:abc"), &path("foo")));
        assert!(matches(&p, &target("//foo/bar:baz"), &path("foo")));
        assert!(!matches(&p, &target("//foo/bav:xyz"), &path("foo")));
        assert!(!matches(&p, &target("//foo/ban:abc"), &path("foo")));
        assert!(!matches(&p, &target("//foo/bas:baz"), &path("foo")));
        // Negative
        let p = LabelPattern::from_str("-bar:*").unwrap();
        assert!(!matches(&p, &target("//foo/bar:xyz"), &path("foo")));
        assert!(!matches(&p, &target("//foo/bar:abc"), &path("foo")));
        assert!(!matches(&p, &target("//foo/bar:baz"), &path("foo")));
        assert!(matches(&p, &target("//foo/bav:xyz"), &path("foo")));
        assert!(matches(&p, &target("//foo/ban:abc"), &path("foo")));
        assert!(matches(&p, &target("//foo/bas:baz"), &path("foo")));
    }

    #[test]
    fn matches_all_targets_in_rel_glob_package__EXPECT__ok() {
        let p = LabelPattern::from_str("...:*").unwrap();
        assert!(matches(&p, &target("//foo/a/bar:xyz"), &path("foo")));
        assert!(matches(&p, &target("//foo/b/bar:abc"), &path("foo")));
        assert!(matches(&p, &target("//foo/c/bar:baz"), &path("foo")));
        assert!(!matches(&p, &target("//soo/a/bar:xyz"), &path("foo")));
        assert!(!matches(&p, &target("//doo/b/bar:abc"), &path("foo")));
        assert!(!matches(&p, &target("//xoo/c/bar:baz"), &path("foo")));
        // Negative
        let p = LabelPattern::from_str("-...:*").unwrap();
        assert!(!matches(&p, &target("//foo/a/bar:xyz"), &path("foo")));
        assert!(!matches(&p, &target("//foo/b/bar:abc"), &path("foo")));
        assert!(!matches(&p, &target("//foo/c/bar:baz"), &path("foo")));
        assert!(matches(&p, &target("//soo/a/bar:xyz"), &path("foo")));
        assert!(matches(&p, &target("//doo/b/bar:abc"), &path("foo")));
        assert!(matches(&p, &target("//xoo/c/bar:baz"), &path("foo")));
    }

    #[test]
    fn matches_only_targets_in_root__EXPECT__ok() {
        let p = LabelPattern::from_str(":*").unwrap();
        assert!(matches(&p, &target("//:xyz"), &path("")));
        assert!(matches(&p, &target("//:abc"), &path("")));
        assert!(matches(&p, &target("//:baz"), &path("")));
        assert!(!matches(&p, &target("//foo/bar:xyz"), &path("")));
        assert!(!matches(&p, &target("//foo/bar:abc"), &path("")));
        assert!(!matches(&p, &target("//foo/bar:baz"), &path("")));
        // Negative
        let p = LabelPattern::from_str("-:*").unwrap();
        assert!(!matches(&p, &target("//:xyz"), &path("")));
        assert!(!matches(&p, &target("//:abc"), &path("")));
        assert!(!matches(&p, &target("//:baz"), &path("")));
        assert!(matches(&p, &target("//foo/bar:xyz"), &path("")));
        assert!(matches(&p, &target("//foo/bar:abc"), &path("")));
        assert!(matches(&p, &target("//foo/bar:baz"), &path("")));
    }

    #[test]
    fn matches_composed_pattern__EXPECT__ok() {
        let p = LabelPattern::try_from(vec![":*".to_string(), "-...:foo".to_string()]).unwrap();
        assert!(matches(&p, &target("//:xyz"), &path("")));
        assert!(matches(&p, &target("//:abc"), &path("")));
        assert!(matches(&p, &target("//:baz"), &path("")));
        assert!(!matches(&p, &target("//:foo"), &path("")));
        assert!(!matches(&p, &target("//abc:foo"), &path("")));
        assert!(!matches(&p, &target("//baz:foo"), &path("")));
    }

    #[test]
    fn matches_composed_pattern2__EXPECT__ok() {
        let p = LabelPattern::try_from(vec!["//foo/...".to_string(), "-//foo/bar/...".to_string()])
            .unwrap();
        assert!(matches(&p, &target("//foo/jkl:xyz"), &path("")));
        assert!(matches(&p, &target("//foo/abc:abc"), &path("")));
        assert!(matches(&p, &target("//foo/xyz/pvp:baz"), &path("")));
        assert!(!matches(&p, &target("//foo/bar/qwe:foo"), &path("")));
        assert!(!matches(&p, &target("//foo/bar/abc:foo"), &path("")));
        assert!(!matches(&p, &target("//foo/bar/baz:foo"), &path("")));
    }

    // Util function
    fn target(s: &str) -> Label {
        let (package, name) = s.rsplit_once(':').unwrap();
        Label::from_package_and_name(package.trim_start_matches("//"), name).unwrap()
    }

    fn path(s: &str) -> PathBuf {
        PathBuf::from(s)
    }
}
