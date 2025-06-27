mod de {
    use toml_span::{Deserialize, ErrorKind};

    use crate::WebpQuality;

    impl<'de> Deserialize<'de> for WebpQuality {
        fn deserialize(value: &mut toml_span::Value<'de>) -> Result<Self, toml_span::DeserError> {
            let quality = match value.take() {
                toml_span::value::ValueInner::Float(value) => value as f32,
                toml_span::value::ValueInner::Integer(value) => value as f32,
                _ => {
                    return Err(toml_span::Error {
                        kind: ErrorKind::Custom(
                            "webp quality mut be a number from 0 to 100".into(),
                        ),
                        span: value.span,
                        line_info: None,
                    }
                    .into());
                }
            };
            match quality {
                0.0..=100.0 => Ok(WebpQuality(quality)),
                _ => Err(toml_span::Error {
                    kind: ErrorKind::Custom("webp quality mut be a number from 0 to 100".into()),
                    span: value.span,
                    line_info: None,
                }
                .into()),
            }
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod test {

    use toml_span::de_helpers::TableHelper;

    use crate::WebpQuality;

    #[test]
    fn WebpQuality__valid_toml__EXPECT__valid_value() {
        // Given
        let toml = r#"
        quality1 = 0
        quality2 = 75.0
        quality3 = 100
        quality4 = -1
        quality5 = 101
        quality6 = "text?"
        "#;
        let quality1 = WebpQuality(0.0);
        let quality2 = WebpQuality(75.0);
        let quality3 = WebpQuality(100.0);

        // When
        let mut value = toml_span::parse(toml).unwrap();
        let mut th = TableHelper::new(&mut value).unwrap();

        // Then
        assert_eq!(quality1, th.required::<WebpQuality>("quality1").unwrap());
        assert_eq!(quality2, th.required::<WebpQuality>("quality2").unwrap());
        assert_eq!(quality3, th.required::<WebpQuality>("quality3").unwrap());
        assert!(th.required::<WebpQuality>("quality4").is_err());
        assert!(th.required::<WebpQuality>("quality5").is_err());
        assert!(th.required::<WebpQuality>("quality6").is_err());
    }
}
