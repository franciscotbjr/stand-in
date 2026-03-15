//! The `McpPrompt` trait implemented by all `#[mcp_prompt]` functions.

use async_trait::async_trait;

use crate::error::Result;

use super::{Prompt, PromptArgument, PromptDefinition};

/// Implemented by every `#[mcp_prompt]`-annotated function.
///
/// The `#[mcp_prompt]` macro generates a struct and this implementation automatically.
#[async_trait]
pub trait McpPrompt: Send + Sync + std::fmt::Debug {
    /// Unique prompt name.
    fn name(&self) -> &str;

    /// Human-readable description.
    fn description(&self) -> &str;

    /// Arguments accepted by this prompt.
    fn arguments(&self) -> Vec<PromptArgument>;

    /// Execute the prompt with the given JSON arguments and return the rendered messages.
    async fn execute(&self, arguments: serde_json::Value) -> Result<Prompt>;

    /// Build the `PromptDefinition` for `prompts/list`.
    fn to_definition(&self) -> PromptDefinition {
        let args = self.arguments();
        PromptDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            arguments: if args.is_empty() { None } else { Some(args) },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;
    use async_trait::async_trait;

    #[derive(Debug)]
    struct DummyPrompt;

    #[async_trait]
    impl McpPrompt for DummyPrompt {
        fn name(&self) -> &str {
            "dummy"
        }

        fn description(&self) -> &str {
            "A dummy prompt"
        }

        fn arguments(&self) -> Vec<PromptArgument> {
            vec![PromptArgument {
                name: "input".to_string(),
                description: None,
                required: Some(true),
            }]
        }

        async fn execute(&self, _arguments: serde_json::Value) -> Result<Prompt> {
            Ok(Prompt::user("dummy output"))
        }
    }

    #[test]
    fn test_to_definition_default_impl() {
        let prompt = DummyPrompt;
        let def = prompt.to_definition();
        assert_eq!(def.name, "dummy");
        assert_eq!(def.description, "A dummy prompt");
        let args = def.arguments.unwrap();
        assert_eq!(args.len(), 1);
        assert_eq!(args[0].name, "input");
        assert_eq!(args[0].required, Some(true));
    }

    #[test]
    fn test_to_definition_no_arguments_returns_none() {
        #[derive(Debug)]
        struct NoArgPrompt;

        #[async_trait]
        impl McpPrompt for NoArgPrompt {
            fn name(&self) -> &str {
                "no_args"
            }
            fn description(&self) -> &str {
                "No args"
            }
            fn arguments(&self) -> Vec<PromptArgument> {
                vec![]
            }
            async fn execute(&self, _: serde_json::Value) -> Result<Prompt> {
                Ok(Prompt::user("ok"))
            }
        }

        let def = NoArgPrompt.to_definition();
        assert!(def.arguments.is_none());
    }
}
