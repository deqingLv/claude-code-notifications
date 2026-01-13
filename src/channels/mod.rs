//! Notification channels module
//!
//! This module provides implementations of various notification channels
//! including system notifications, WeChat, Feishu, and DingTalk.

pub mod dingtalk;
pub mod feishu;
pub mod system;
pub mod r#trait;
pub mod webhook;
pub mod wechat;

pub use dingtalk::DingTalkChannel;
pub use feishu::FeishuChannel;
pub use r#trait::{map_channel_error, ChannelResult, NotificationChannel};
pub use system::SystemChannel;
pub use webhook::WebhookClient;
pub use wechat::WeChatChannel;

use std::collections::HashMap;

/// Channel factory function type
type ChannelFactory = Box<dyn Fn() -> Box<dyn NotificationChannel + Send + Sync> + Send + Sync>;

/// Channel registry for managing available notification channels
pub struct ChannelRegistry {
    factories: HashMap<String, ChannelFactory>,
}

impl ChannelRegistry {
    /// Create a new channel registry with all built-in channels registered
    pub fn new() -> Self {
        let mut registry = Self {
            factories: HashMap::new(),
        };

        // Register built-in channels
        registry.register_factory(
            "system",
            Box::new(|| {
                Box::new(SystemChannel::new()) as Box<dyn NotificationChannel + Send + Sync>
            }),
        );
        registry.register_factory(
            "wechat",
            Box::new(|| {
                Box::new(WeChatChannel::new()) as Box<dyn NotificationChannel + Send + Sync>
            }),
        );
        registry.register_factory(
            "feishu",
            Box::new(|| {
                Box::new(FeishuChannel::new()) as Box<dyn NotificationChannel + Send + Sync>
            }),
        );
        registry.register_factory(
            "dingtalk",
            Box::new(|| {
                Box::new(DingTalkChannel::new()) as Box<dyn NotificationChannel + Send + Sync>
            }),
        );

        registry
    }

    /// Register a channel factory
    fn register_factory<F>(&mut self, channel_type: &str, factory: F)
    where
        F: Fn() -> Box<dyn NotificationChannel + Send + Sync> + Send + Sync + 'static,
    {
        self.factories
            .insert(channel_type.to_string(), Box::new(factory));
    }

    /// Create a channel instance by type
    pub fn create_channel(
        &self,
        channel_type: &str,
    ) -> Option<Box<dyn NotificationChannel + Send + Sync>> {
        self.factories.get(channel_type).map(|factory| factory())
    }

    /// List all registered channel types
    pub fn list_channels(&self) -> Vec<&str> {
        self.factories.keys().map(|k| k.as_str()).collect()
    }
}

impl Default for ChannelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry() {
        let registry = ChannelRegistry::new();
        let channels = registry.list_channels();

        assert!(channels.contains(&"system"));
        assert!(channels.contains(&"wechat"));
        assert!(channels.contains(&"feishu"));
        assert!(channels.contains(&"dingtalk"));

        assert!(registry.create_channel("system").is_some());
        assert!(registry.create_channel("wechat").is_some());
        assert!(registry.create_channel("feishu").is_some());
        assert!(registry.create_channel("dingtalk").is_some());
        assert!(registry.create_channel("nonexistent").is_none());
    }
}
