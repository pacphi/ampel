use super::{FormatError, TranslationFormat, TranslationMap};

pub struct YamlFormat;

impl TranslationFormat for YamlFormat {
    fn parse(&self, content: &str) -> Result<TranslationMap, FormatError> {
        serde_yaml::from_str(content).map_err(|e| FormatError::ParseError(e.to_string()))
    }

    fn write(&self, map: &TranslationMap) -> Result<String, FormatError> {
        serde_yaml::to_string(map).map_err(|e| FormatError::WriteError(e.to_string()))
    }

    fn validate(&self, content: &str) -> Result<(), FormatError> {
        self.parse(content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::TranslationValue;

    #[test]
    fn test_parse_simple_yaml() {
        let yaml = r#"
hello: "World"
goodbye: "Farewell"
"#;
        let format = YamlFormat;
        let map = format.parse(yaml).unwrap();

        assert_eq!(map.len(), 2);
        assert_eq!(
            map.get("hello"),
            Some(&TranslationValue::String("World".to_string()))
        );
    }

    #[test]
    fn test_parse_plural_forms() {
        let yaml = r#"
pull_requests:
  count:
    one: "1 pull request"
    other: "%{count} pull requests"
"#;
        let format = YamlFormat;
        let map = format.parse(yaml).unwrap();

        if let Some(TranslationValue::Nested(nested)) = map.get("pull_requests") {
            if let Some(TranslationValue::Plural(forms)) = nested.get("count") {
                assert_eq!(forms.one, Some("1 pull request".to_string()));
                assert_eq!(forms.other, "%{count} pull requests");
            } else {
                panic!("Expected plural forms");
            }
        } else {
            panic!("Expected nested structure");
        }
    }

    #[test]
    fn test_write_preserves_structure() {
        let yaml = r#"hello: World
pull_requests:
  count:
    one: 1 pull request
    other: '%{count} pull requests'
"#;
        let format = YamlFormat;
        let map = format.parse(yaml).unwrap();
        let yaml_out = format.write(&map).unwrap();

        assert!(yaml_out.contains("hello:"));
        assert!(yaml_out.contains("pull_requests:"));
        assert!(yaml_out.contains("count:"));
    }
}
