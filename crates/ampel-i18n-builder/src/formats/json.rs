use super::{FormatError, TranslationFormat, TranslationMap};

pub struct JsonFormat;

impl JsonFormat {
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonFormat {
    fn default() -> Self {
        Self::new()
    }
}

impl TranslationFormat for JsonFormat {
    fn parse(&self, content: &str) -> Result<TranslationMap, FormatError> {
        serde_json::from_str(content).map_err(|e| FormatError::ParseError(e.to_string()))
    }

    fn write(&self, map: &TranslationMap) -> Result<String, FormatError> {
        serde_json::to_string_pretty(map).map_err(|e| FormatError::WriteError(e.to_string()))
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
    fn test_parse_simple_json() {
        let json = r#"{
            "hello": "World",
            "goodbye": "Farewell"
        }"#;
        let format = JsonFormat;
        let map = format.parse(json).unwrap();

        assert_eq!(map.len(), 2);
        assert_eq!(
            map.get("hello"),
            Some(&TranslationValue::String("World".to_string()))
        );
    }

    #[test]
    fn test_parse_nested_json() {
        let json = r#"{
            "pullRequests": {
                "count_one": "{{count}} pull request",
                "count_other": "{{count}} pull requests"
            }
        }"#;
        let format = JsonFormat;
        let map = format.parse(json).unwrap();

        if let Some(TranslationValue::Nested(nested)) = map.get("pullRequests") {
            assert!(nested.contains_key("count_one"));
            assert!(nested.contains_key("count_other"));
        } else {
            panic!("Expected nested structure");
        }
    }

    #[test]
    fn test_write_formats_json() {
        let json = r#"{"hello":"World"}"#;
        let format = JsonFormat;
        let map = format.parse(json).unwrap();
        let json_out = format.write(&map).unwrap();

        // Pretty printed JSON should have newlines
        assert!(json_out.contains('\n'));
        assert!(json_out.contains("\"hello\""));
    }
}
