//! Configuration management module
//!
//! This module handles loading, saving, and rendering configuration
//! for multi-channel notifications.

pub mod loader;
pub mod schema;
pub mod templates;

pub use loader::{default_config, get_config_path, load_config, load_config_from_path, save_config, save_config_to_path};
pub use schema::{AppConfig, ChannelConfig, MessageTemplate, RuleMatch, RoutingRule};
pub use templates::{RenderedMessage, TemplateEngine};
