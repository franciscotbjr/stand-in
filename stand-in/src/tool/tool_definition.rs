//! Describes a tool that the MCP server exposes to clients.

use serde::{Deserialize, Serialize};

use super::InputSchema;

/// Describes a tool that the MCP server exposes to clients.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// Tool name.
    pub name: String,

    /// Human-readable description.
    pub description: String,

    /// JSON Schema for the tool's input.
    #[serde(rename = "inputSchema")]
    pub input_schema: InputSchema,
}

impl ToolDefinition {
    /// Create a new tool definition.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: InputSchema,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_definition() {
        let def = ToolDefinition::new("greet", "Greet someone", InputSchema::object());
        assert_eq!(def.name, "greet");
        assert_eq!(def.description, "Greet someone");
        assert_eq!(def.input_schema.schema_type, "object");
    }

    #[test]
    fn test_definition_serialize() {
        let def = ToolDefinition::new(
            "greet",
            "Greet someone",
            InputSchema::object().with_properties(serde_json::json!({
                "name": {"type": "string"}
            })),
        );
        let json = serde_json::to_value(&def).unwrap();
        assert_eq!(json["name"], "greet");
        assert_eq!(json["description"], "Greet someone");
        assert_eq!(json["inputSchema"]["type"], "object");
        assert_eq!(json["inputSchema"]["properties"]["name"]["type"], "string");
    }
}
