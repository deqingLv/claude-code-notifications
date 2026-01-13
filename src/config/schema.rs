//! Configuration schema for claude-code-notifications
//!
//! This module defines the data structures for the application configuration,
//! including channels, routing rules, and message templates.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Main application configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    /// Configuration format version
    pub version: String,

    /// Default channels to use when no routing rules match
    #[serde(default = "default_channels")]
    pub default_channels: Vec<String>,

    /// Channel configurations keyed by channel type
    #[serde(default)]
    pub channels: HashMap<String, ChannelConfig>,

    /// Routing rules for intelligent channel selection
    #[serde(default)]
    pub routing_rules: Vec<RoutingRule>,

    /// Global message templates
    #[serde(default)]
    pub global_templates: HashMap<String, MessageTemplate>,
}

fn default_channels() -> Vec<String> {
    vec!["system".to_string()]
}

/// Configuration for a specific notification channel
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
#[derive(Default)]
pub struct ChannelConfig {
    /// Display name for this channel instance (e.g., "个人钉钉", "协作群")
    #[serde(default)]
    pub name: Option<String>,

    /// Channel type (system, dingtalk, feishu, wechat)
    /// This determines which channel implementation to use
    #[serde(default)]
    pub channel_type: String,

    /// Whether this channel is enabled
    pub enabled: bool,

    /// Webhook URL (for webhook-based channels)
    pub webhook_url: Option<String>,

    /// Secret for webhook signing (DingTalk)
    pub secret: Option<String>,

    /// Icon URL or path
    pub icon: Option<String>,

    /// Message template for this channel
    pub message_template: Option<MessageTemplate>,

    /// System-specific settings (sound, timeout)
    pub sound: Option<String>,

    pub timeout_ms: Option<u64>,

    /// Additional channel-specific settings
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Message template with variable substitution
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
#[derive(Default)]
pub struct MessageTemplate {
    /// Template for notification title
    pub title: Option<String>,

    /// Template for notification body
    pub body: Option<String>,

    /// Mentioned users list (WeChat)
    pub mentioned_list: Option<Vec<String>>,

    /// Additional template-specific fields
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Routing rule for intelligent channel selection
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RoutingRule {
    /// Human-readable name for this rule
    pub name: String,

    /// Matching conditions
    #[serde(default, rename = "match")]
    pub match_conditions: RuleMatch,

    /// Channels to use when this rule matches
    pub channels: Vec<String>,

    /// Whether this rule is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

/// Matching conditions for routing rules
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
#[derive(Default)]
pub struct RuleMatch {
    /// Hook types to match (e.g., "Notification", "PreToolUse")
    pub hook_types: Vec<String>,

    /// Regex pattern to match against message content
    pub message_pattern: Option<String>,

    /// Regex pattern to match against tool name (PreToolUse only)
    pub tool_pattern: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_default_app_config() {
        let config: AppConfig = serde_json::from_value(json!({
            "version": "1.0",
            "channels": {}
        }))
        .unwrap();

        assert_eq!(config.version, "1.0");
        assert_eq!(config.default_channels, vec!["system"]);
        assert!(config.channels.is_empty());
        assert!(config.routing_rules.is_empty());
    }

    #[test]
    fn test_channel_config_with_webhook() {
        let config: ChannelConfig = serde_json::from_value(json!({
            "enabled": true,
            "webhook_url": "https://example.com/webhook",
            "icon": "https://example.com/icon.png"
        }))
        .unwrap();

        assert!(config.enabled);
        assert_eq!(
            config.webhook_url,
            Some("https://example.com/webhook".to_string())
        );
        assert_eq!(
            config.icon,
            Some("https://example.com/icon.png".to_string())
        );
    }

    #[test]
    fn test_routing_rule() {
        let rule: RoutingRule = serde_json::from_value(json!({
            "name": "Test rule",
            "match": {
                "hook_types": ["Notification", "Stop"],
                "message_pattern": ".*error.*"
            },
            "channels": ["system", "wechat"],
            "enabled": true
        }))
        .unwrap();

        assert_eq!(rule.name, "Test rule");
        assert_eq!(rule.match_conditions.hook_types.len(), 2);
        assert_eq!(
            rule.match_conditions.message_pattern,
            Some(".*error.*".to_string())
        );
        assert_eq!(rule.channels, vec!["system", "wechat"]);
        assert!(rule.enabled);
    }
}
