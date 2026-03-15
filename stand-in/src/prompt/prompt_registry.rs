//! Registry that holds all registered `#[mcp_prompt]` implementations.

use tracing::{debug, warn};

use crate::error::{Error, Result};

use super::{GetPromptResult, McpPrompt, PromptDefinition};

/// Holds all registered prompts and dispatches `prompts/list` and `prompts/get`.
#[derive(Debug, Default)]
pub struct PromptRegistry {
    prompts: Vec<Box<dyn McpPrompt>>,
}

impl PromptRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a prompt implementation.
    pub fn register(&mut self, prompt: Box<dyn McpPrompt>) {
        debug!(prompt = %prompt.name(), "Registered prompt");
        self.prompts.push(prompt);
    }

    /// Return definitions of all registered prompts.
    pub fn list_definitions(&self) -> Vec<PromptDefinition> {
        self.prompts.iter().map(|p| p.to_definition()).collect()
    }

    /// Execute a prompt by name and return the result.
    pub async fn get_prompt(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<GetPromptResult> {
        let prompt = self.prompts.iter().find(|p| p.name() == name);

        match prompt {
            Some(p) => {
                debug!(prompt = %name, "Executing prompt");
                let result = p.execute(arguments).await?;
                Ok(GetPromptResult {
                    description: Some(p.description().to_string()),
                    messages: result.messages,
                })
            }
            None => {
                warn!(prompt = %name, "Unknown prompt requested");
                Err(Error::PromptError(format!("Unknown prompt: {name}")))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt::{Prompt, PromptArgument};
    use async_trait::async_trait;

    #[derive(Debug)]
    struct HelloPrompt;

    #[async_trait]
    impl McpPrompt for HelloPrompt {
        fn name(&self) -> &str {
            "hello"
        }

        fn description(&self) -> &str {
            "Say hello"
        }

        fn arguments(&self) -> Vec<PromptArgument> {
            vec![]
        }

        async fn execute(&self, _arguments: serde_json::Value) -> Result<Prompt> {
            Ok(Prompt::user("Hello!"))
        }
    }

    #[tokio::test]
    async fn test_list_definitions() {
        let mut registry = PromptRegistry::new();
        registry.register(Box::new(HelloPrompt));
        let defs = registry.list_definitions();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].name, "hello");
    }

    #[tokio::test]
    async fn test_get_prompt_success() {
        let mut registry = PromptRegistry::new();
        registry.register(Box::new(HelloPrompt));
        let result = registry
            .get_prompt("hello", serde_json::Value::Null)
            .await
            .unwrap();
        assert_eq!(result.messages.len(), 1);
        assert_eq!(result.description, Some("Say hello".to_string()));
    }

    #[tokio::test]
    async fn test_get_prompt_unknown() {
        let registry = PromptRegistry::new();
        let result = registry
            .get_prompt("nonexistent", serde_json::Value::Null)
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown prompt"));
    }
}
