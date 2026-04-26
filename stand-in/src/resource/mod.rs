//! MCP resource types — trait, registry, factory, and protocol types.

mod list_resource_templates_result;
mod list_resources_result;
mod read_resource_params;
mod read_resource_result;
mod resource_annotations;
mod resource_contents;
mod resource_definition;
mod resource_factory;
mod resource_registry;
mod resource_template;
mod resource_trait;
mod subscribe_params;
mod unsubscribe_params;

pub use list_resource_templates_result::ListResourceTemplatesResult;
pub use list_resources_result::ListResourcesResult;
pub use read_resource_params::ReadResourceParams;
pub use read_resource_result::ReadResourceResult;
pub use resource_annotations::ResourceAnnotations;
pub use resource_contents::ResourceContents;
pub use resource_definition::Resource;
pub use resource_factory::ResourceFactory;
pub use resource_registry::{ResourceRegistry, match_template_params};
pub use resource_template::ResourceTemplate;
pub use resource_trait::McpResource;
pub use subscribe_params::SubscribeParams;
pub use unsubscribe_params::UnsubscribeParams;
