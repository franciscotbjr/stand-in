//! JSON Schema describing the expected input for a tool.

use serde::{Deserialize, Serialize};

/// JSON Schema describing the expected input for a tool.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputSchema {
    /// Schema type (always "object" for tool inputs).
    #[serde(rename = "type")]
    pub schema_type: String,

    /// Property definitions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<serde_json::Value>,

    /// Required property names.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

impl InputSchema {
    /// Create a new object-type input schema.
    pub fn object() -> Self {
        Self {
            schema_type: "object".to_string(),
            properties: None,
            required: None,
        }
    }

    /// Set the properties for this schema.
    pub fn with_properties(mut self, properties: serde_json::Value) -> Self {
        self.properties = Some(properties);
        self
    }

    /// Set the required fields for this schema.
    pub fn with_required(mut self, required: Vec<String>) -> Self {
        self.required = Some(required);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_schema() {
        let schema = InputSchema::object();
        assert_eq!(schema.schema_type, "object");
        assert!(schema.properties.is_none());
        assert!(schema.required.is_none());
    }

    #[test]
    fn test_with_properties() {
        let schema = InputSchema::object().with_properties(serde_json::json!({
            "name": {"type": "string"}
        }));
        assert!(schema.properties.is_some());
        let props = schema.properties.unwrap();
        assert_eq!(props["name"]["type"], "string");
    }

    #[test]
    fn test_with_required() {
        let schema =
            InputSchema::object().with_required(vec!["name".to_string(), "age".to_string()]);
        assert_eq!(
            schema.required,
            Some(vec!["name".to_string(), "age".to_string()])
        );
    }
}
