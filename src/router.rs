//! Routing rule engine for intelligent channel selection
//!
//! This module implements a routing engine that determines which notification
//! channels should receive a notification based on hook type, message content,
//! and custom routing rules.

use crate::config::{AppConfig, RoutingRule};
use crate::error::{NotificationError, Result};
use crate::hooks::{HookData, HookInput};
use regex::Regex;
use std::collections::HashSet;

/// Channel router for intelligent notification distribution
pub struct ChannelRouter {
    rules: Vec<RoutingRule>,
}

impl ChannelRouter {
    /// Create a new channel router from configuration
    pub fn new(config: &AppConfig) -> Self {
        Self {
            rules: config.routing_rules.clone(),
        }
    }

    /// Find which channels should receive this notification
    ///
    /// Returns a list of channel IDs based on matching routing rules.
    /// If no rules match, returns the default channels from configuration.
    pub fn match_channels(&self, input: &HookInput, config: &AppConfig) -> Result<Vec<String>> {
        let mut matched_channels = Vec::new();

        // Check each routing rule in order
        for rule in &self.rules {
            if !rule.enabled {
                continue;
            }

            if self.matches_rule(input, rule)? {
                // Add channels from this rule (avoid duplicates)
                for channel_id in &rule.channels {
                    if !matched_channels.contains(channel_id) {
                        matched_channels.push(channel_id.clone());
                    }
                }
            }
        }

        // If no rules matched, use default channels
        if matched_channels.is_empty() {
            matched_channels = config.default_channels.clone();
        }

        Ok(matched_channels)
    }

    /// Check if a hook input matches a routing rule
    fn matches_rule(&self, input: &HookInput, rule: &RoutingRule) -> Result<bool> {
        // Check hook type match
        if !rule.match_conditions.hook_types.is_empty() {
            let hook_type_str = format!("{:?}", input.hook_event_name);
            if !rule.match_conditions.hook_types.contains(&hook_type_str) {
                return Ok(false);
            }
        }

        // Check message pattern
        if let Some(pattern) = &rule.match_conditions.message_pattern {
            let regex = Regex::new(pattern)
                .map_err(|e| NotificationError::RoutingError(format!("Invalid regex: {}", e)))?;
            let message = self.extract_message(input);
            if !regex.is_match(&message) {
                return Ok(false);
            }
        }

        // Check tool pattern for PreToolUse
        if let Some(pattern) = &rule.match_conditions.tool_pattern {
            if let HookData::PreToolUse(data) = &input.data {
                let regex = Regex::new(pattern).map_err(|e| {
                    NotificationError::RoutingError(format!("Invalid regex: {}", e))
                })?;
                if !regex.is_match(&data.tool_name) {
                    return Ok(false);
                }
            } else {
                // Tool pattern specified but not a PreToolUse hook
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Extract message text from hook input for pattern matching
    fn extract_message(&self, input: &HookInput) -> String {
        match &input.data {
            HookData::Notification(data) => data.message.clone(),
            HookData::PreToolUse(data) => data.tool_name.clone(),
            HookData::Stop(_data) => "Claude stopped".to_string(),
            HookData::SubagentStop(_data) => "Subagent stopped".to_string(),
            HookData::PermissionRequest(data) => {
                if let Some(desc) = &data.description {
                    desc.clone()
                } else if let Some(perm_type) = &data.permission_type {
                    format!("Permission request: {}", perm_type)
                } else {
                    "Permission request".to_string()
                }
            }
        }
    }

    /// Override channels manually (bypasses routing rules)
    ///
    /// This is used when the user specifies channels via CLI parameter
    pub fn override_channels(&self, channel_ids: Vec<String>) -> Vec<String> {
        // Deduplicate while preserving order
        let mut seen = HashSet::new();
        channel_ids
            .into_iter()
            .filter(|x| seen.insert(x.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ChannelConfig, RuleMatch};
    use std::collections::HashMap;

    fn create_test_config() -> AppConfig {
        let mut channels = HashMap::new();
        channels.insert(
            "system".to_string(),
            ChannelConfig {
                enabled: true,
                ..Default::default()
            },
        );

        AppConfig {
            version: "1.0".to_string(),
            default_channels: vec!["system".to_string()],
            channels,
            routing_rules: vec![],
            global_templates: HashMap::new(),
            debug: false,
        }
    }

    #[test]
    fn test_match_default_channels() {
        let config = create_test_config();
        let router = ChannelRouter::new(&config);

        let input = HookInput::notification(
            "test".to_string(),
            None,
            "Test message".to_string(),
            Some("Test".to_string()),
        );

        let channels = router.match_channels(&input, &config).unwrap();
        assert_eq!(channels, vec!["system"]);
    }

    #[test]
    fn test_match_hook_type() {
        let mut config = create_test_config();
        config.routing_rules = vec![RoutingRule {
            name: "Stop notifications".to_string(),
            match_conditions: RuleMatch {
                hook_types: vec!["Stop".to_string()],
                ..Default::default()
            },
            channels: vec!["system".to_string()],
            enabled: true,
        }];

        let router = ChannelRouter::new(&config);

        let stop_input = HookInput::stop("test".to_string(), None, Some("Test stop".to_string()));

        let notification_input = HookInput::notification(
            "test".to_string(),
            None,
            "Test message".to_string(),
            Some("Test".to_string()),
        );

        // Stop hook should match the rule
        let channels = router.match_channels(&stop_input, &config).unwrap();
        assert_eq!(channels, vec!["system"]);

        // Notification hook should not match, use default
        let channels = router.match_channels(&notification_input, &config).unwrap();
        assert_eq!(channels, vec!["system"]); // Falls back to default
    }

    #[test]
    fn test_match_message_pattern() {
        let mut config = create_test_config();
        config.routing_rules = vec![RoutingRule {
            name: "Error messages".to_string(),
            match_conditions: RuleMatch {
                hook_types: vec![],
                message_pattern: Some(".*error.*".to_string()),
                tool_pattern: None,
            },
            channels: vec!["system".to_string()],
            enabled: true,
        }];

        let router = ChannelRouter::new(&config);

        let error_input = HookInput::notification(
            "test".to_string(),
            None,
            "This is an error message".to_string(),
            Some("Error".to_string()),
        );

        let normal_input = HookInput::notification(
            "test".to_string(),
            None,
            "This is a normal message".to_string(),
            Some("Info".to_string()),
        );

        // Error message should match
        let channels = router.match_channels(&error_input, &config).unwrap();
        assert_eq!(channels, vec!["system"]);

        // Normal message should not match, use default
        let channels = router.match_channels(&normal_input, &config).unwrap();
        assert_eq!(channels, vec!["system"]);
    }

    #[test]
    fn test_match_tool_pattern() {
        let mut config = create_test_config();
        config.routing_rules = vec![RoutingRule {
            name: "ExitPlanMode only".to_string(),
            match_conditions: RuleMatch {
                hook_types: vec![],
                message_pattern: None,
                tool_pattern: Some("ExitPlanMode".to_string()),
            },
            channels: vec!["system".to_string()],
            enabled: true,
        }];

        let router = ChannelRouter::new(&config);

        let exit_plan_input = HookInput::pre_tool_use(
            "test".to_string(),
            None,
            "ExitPlanMode".to_string(),
            Some("Exiting plan mode".to_string()),
        );

        let other_tool_input = HookInput::pre_tool_use(
            "test".to_string(),
            None,
            "Task".to_string(),
            Some("Launching task".to_string()),
        );

        // ExitPlanMode should match
        let channels = router.match_channels(&exit_plan_input, &config).unwrap();
        assert_eq!(channels, vec!["system"]);

        // Other tool should not match, use default
        let channels = router.match_channels(&other_tool_input, &config).unwrap();
        assert_eq!(channels, vec!["system"]);
    }

    #[test]
    fn test_override_channels() {
        let config = create_test_config();
        let router = ChannelRouter::new(&config);

        let channels = router.override_channels(vec![
            "system".to_string(),
            "wechat".to_string(),
            "system".to_string(),
        ]);
        assert_eq!(channels, vec!["system", "wechat"]);
    }

    #[test]
    fn test_disabled_rule() {
        let mut config = create_test_config();
        config.routing_rules = vec![RoutingRule {
            name: "Disabled rule".to_string(),
            match_conditions: RuleMatch {
                hook_types: vec!["Stop".to_string()],
                ..Default::default()
            },
            channels: vec!["custom".to_string()],
            enabled: false,
        }];

        let router = ChannelRouter::new(&config);

        let stop_input = HookInput::stop("test".to_string(), None, Some("Test stop".to_string()));

        // Should use default since rule is disabled
        let channels = router.match_channels(&stop_input, &config).unwrap();
        assert_eq!(channels, vec!["system"]);
    }
}
