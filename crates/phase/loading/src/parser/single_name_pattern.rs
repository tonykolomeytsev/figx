mod de {
    use toml_span::{Deserialize, ErrorKind};

    use crate::SingleNamePattern;

    impl<'de> Deserialize<'de> for SingleNamePattern {
        fn deserialize(value: &mut toml_span::Value<'de>) -> Result<Self, toml_span::DeserError> {
            match value.as_str() {
                Some(n) if !n.contains("{base}") => {
                    return Err(toml_span::Error::from((
                        ErrorKind::Custom("expected string pattern with `{base}` marker".into()),
                        value.span,
                    ))
                    .into());
                }
                Some(string) => Ok(SingleNamePattern(string.to_owned())),
                None => {
                    return Err(toml_span::Error::from((
                        ErrorKind::Custom("expected string pattern with `{base}` marker".into()),
                        value.span,
                    ))
                    .into());
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {

    use toml_span::de_helpers::TableHelper;
    use crate::SingleNamePattern;

    #[test]
    fn SingleNamePattern__valid_toml__EXPECT__valid_value() {
        // Given
        let toml = r#"
        s1 = "{base}-big"
        s2 = "prefix / {base} / suffix"
        s3 = "doubled: {base}X{base}"
        s4 = "smth"
        s5 = "no base? :("
        "#;
        let s1 = SingleNamePattern("{base}-big".to_string());
        let s2 = SingleNamePattern("prefix / {base} / suffix".to_string());
        let s3 = SingleNamePattern("doubled: {base}X{base}".to_string());

        // When
        let mut value = toml_span::parse(toml).unwrap();
        let mut th = TableHelper::new(&mut value).unwrap();

        // Then
        assert_eq!(s1, th.required::<SingleNamePattern>("s1").unwrap());
        assert_eq!(s2, th.required::<SingleNamePattern>("s2").unwrap());
        assert_eq!(s3, th.required::<SingleNamePattern>("s3").unwrap());
        assert!(th.required::<SingleNamePattern>("s4").is_err());
        assert!(th.required::<SingleNamePattern>("s5").is_err());
    }
}
