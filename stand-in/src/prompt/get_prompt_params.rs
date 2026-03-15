//! Parameters for the `prompts/get` JSON-RPC method.

use serde::{Deserialize, Serialize};

/// Parameters for a `prompts/get` request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPromptParams {
    /// Name of the prompt to retrieve.
    pub name: String,

    /// Arguments to pass to the prompt function.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
}
