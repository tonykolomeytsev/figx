use crate::Label;

impl Label {
    /// Try to fit label string to the length of the `n_chars`
    pub fn fitted(&self, n_chars: usize) -> String {
        let name_len = self.name.as_ref().len();
        let pkg_path = self
            .package
            .0
            .to_str()
            .expect("always valid UTF-8")
            .to_string();
        if name_len + pkg_path.len() + 3 <= n_chars {
            return format!("//{pkg_path}:{}", self.name);
        }

        if self.package.as_os_str().is_empty() {
            return format!("//:{}", self.name);
        } else {
            let mut path_len = pkg_path.len();
            let path = self
                .package
                .0
                .iter()
                .skip_while(|p| {
                    let skip = path_len + name_len + 3 + 4 > n_chars;
                    path_len -= p.to_str().expect("always valid UTF-8").len();
                    if path_len > 0 {
                        path_len -= 1;
                    }
                    skip
                })
                .map(|p| p.to_str().expect("always valid UTF-8").to_owned())
                .collect::<Vec<String>>()
                .join("/");

            if path.is_empty() {
                return format!("//...:{}", self.name);
            } else {
                return format!("//.../{path}:{}", self.name);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Name, Package};
    use std::path::PathBuf;

    fn label(path: &str, name: &str) -> Label {
        Label {
            package: Package(PathBuf::from(path)),
            name: Name(name.to_owned()),
        }
    }

    #[test]
    fn fits_without_truncation() {
        let l = label("short", "target");
        let result = l.fitted(20);
        assert_eq!(result, "//short:target");
    }

    #[test]
    fn truncated_fully() {
        let l = label("short", "target");
        let result = l.fitted(1);
        assert_eq!(result, "//...:target");
    }

    #[test]
    fn truncated_fully_on_root_dir() {
        let l = label("", "target");
        let result = l.fitted(1);
        assert_eq!(result, "//:target");
    }

    #[test]
    fn truncates_with_middle_ellipsis() {
        let l = label("path/to/package", "target");
        let result = l.fitted(23);
        assert_eq!(result, "//.../to/package:target");

        let result = l.fitted(20);
        assert_eq!(result, "//.../package:target");
        assert!(result.len() <= 20);
    }

    #[test]
    fn truncates_to_colon_only() {
        let l = label("path/to/package", "target");
        let result = l.fitted(15);
        assert_eq!(result, "//...:target");
        assert!(result.len() <= 15);
    }

    #[test]
    fn truncates_to_shortened_suffix() {
        let l = label("path/to/package", "target");
        let result = l.fitted(10);
        assert_eq!(result, "//...:target");
    }

    #[test]
    fn long_compose_label() {
        let l = label(
            "jason/components/src/main/kotlin/io/cc/photo/jason/component",
            "ChevronLeft",
        );
        let result = l.fitted(40);
        assert_eq!(result, "//.../photo/jason/component:ChevronLeft");
        assert!(result.len() <= 40);
    }
}
