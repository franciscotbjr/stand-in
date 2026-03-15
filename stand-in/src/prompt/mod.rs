//! MCP prompt types — trait, registry, factory, and protocol types.

mod get_prompt_params;
mod get_prompt_result;
mod list_prompts_result;
mod prompt_argument;
mod prompt_definition;
mod prompt_factory;
mod prompt_message;
mod prompt_registry;
mod prompt_trait;
mod prompt_type;

pub use get_prompt_params::GetPromptParams;
pub use get_prompt_result::GetPromptResult;
pub use list_prompts_result::ListPromptsResult;
pub use prompt_argument::PromptArgument;
pub use prompt_definition::PromptDefinition;
pub use prompt_factory::PromptFactory;
pub use prompt_message::{PromptContent, PromptMessage, PromptRole};
pub use prompt_registry::PromptRegistry;
pub use prompt_trait::McpPrompt;
pub use prompt_type::Prompt;
