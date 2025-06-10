mod de {
    use crate::ExportScale;
    use toml_span::{Deserialize, ErrorKind};

    impl<'de> Deserialize<'de> for ExportScale {
        fn deserialize(value: &mut toml_span::Value<'de>) -> Result<Self, toml_span::DeserError> {
            let scale = match value.take() {
                toml_span::value::ValueInner::Float(value) => value as f32,
                toml_span::value::ValueInner::Integer(value) => value as f32,
                _ => {
                    return Err(toml_span::Error {
                        kind: ErrorKind::Custom("scale mut be a number from 0.1 to 4".into()),
                        span: value.span,
                        line_info: None,
                    }
                    .into());
                }
            };
            match scale {
                0.1..=4.0 => Ok(ExportScale(scale)),
                _ => Err(toml_span::Error {
                    kind: ErrorKind::Custom("scale mut be a number from 0.1 to 4".into()),
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
    use crate::ExportScale;

    #[test]
    fn ExportScale__valid_toml__EXPECT__valid_value() {
        // Given
        let toml = r#"
        s1 = 0.1
        s2 = 2
        s3 = 4.0
        s4 = 0
        s5 = 4.1
        "#;
        let scale1 = ExportScale(0.1);
        let scale2 = ExportScale(2.0);
        let scale3 = ExportScale(4.0);

        // When
        let mut value = toml_span::parse(toml).unwrap();
        let mut th = TableHelper::new(&mut value).unwrap();

        // Then
        assert_eq!(scale1, th.required::<ExportScale>("s1").unwrap());
        assert_eq!(scale2, th.required::<ExportScale>("s2").unwrap());
        assert_eq!(scale3, th.required::<ExportScale>("s3").unwrap());
        assert!(th.required::<ExportScale>("s4").is_err());
        assert!(th.required::<ExportScale>("s5").is_err());
    }
}
