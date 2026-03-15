//! Definition of an MCP prompt template as returned by `prompts/list`.

use serde::{Deserialize, Serialize};

use super::PromptArgument;

/// Metadata descriptor for an MCP prompt, as returned in `prompts/list`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptDefinition {
    /// Unique prompt name.
    pub name: String,

    /// Human-readable description.
    pub description: String,

    /// Arguments accepted by this prompt, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<PromptArgument>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_definition_serialize_no_arguments() {
        let def = PromptDefinition {
            name: "summarize".to_string(),
            description: "Summarize a document".to_string(),
            arguments: None,
        };
        let json = serde_json::to_value(&def).unwrap();
        assert_eq!(json["name"], "summarize");
        assert_eq!(json["description"], "Summarize a document");
        assert!(json.get("arguments").is_none());
    }

    #[test]
    fn test_definition_serialize_with_arguments() {
        let def = PromptDefinition {
            name: "greet".to_string(),
            description: "Greet someone".to_string(),
            arguments: Some(vec![PromptArgument {
                name: "name".to_string(),
                description: None,
                required: Some(true),
            }]),
        };
        let json = serde_json::to_value(&def).unwrap();
        assert_eq!(json["arguments"][0]["name"], "name");
        assert_eq!(json["arguments"][0]["required"], true);
    }
}
