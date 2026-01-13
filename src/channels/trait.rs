//! Notification channel trait definition
//!
//! This module defines the core abstraction for notification channels.
//! All notification channels must implement the NotificationChannel trait.

use async_trait::async_trait;
use crate::config::ChannelConfig;
use crate::error::{ChannelError, NotificationError, Result};
use crate::hooks::HookInput;

/// Result type for channel operations
pub type ChannelResult<T> = std::result::Result<T, ChannelError>;

/// Notification channel trait
///
/// All notification channels must implement this trait to provide
/// a unified interface for sending notifications through different backends.
#[async_trait]
pub trait NotificationChannel: Send + Sync {
    /// Get the unique identifier for this channel type
    /// Examples: "system", "wechat", "feishu", "dingtalk"
    fn channel_type(&self) -> &'static str;

    /// Get the human-readable display name for this channel
    /// Examples: "System Notification", "WeChat Work", "Feishu", "DingTalk"
    fn display_name(&self) -> &'static str;

    /// Check if this channel is enabled and properly configured
    fn is_enabled(&self, config: &ChannelConfig) -> bool {
        config.enabled
    }

    /// Validate the channel configuration
    /// Returns an error if the configuration is invalid or incomplete
    fn validate_config(&self, config: &ChannelConfig) -> ChannelResult<()> {
        if !config.enabled {
            return Err(ChannelError::DisabledError);
        }
        Ok(())
    }

    /// Send a notification through this channel
    /// This method should return immediately after dispatching the notification
    /// and handle errors gracefully without blocking
    async fn send(&self, input: &HookInput, config: &ChannelConfig) -> ChannelResult<()>;

    /// Optional: Test the channel connection
    /// This is useful for verifying configuration in the web UI
    /// Default implementation returns a success message
    async fn test(&self, config: &ChannelConfig) -> ChannelResult<String> {
        self.validate_config(config)?;
        Ok(format!("{} channel test successful", self.display_name()))
    }
}

/// Helper function to convert ChannelError to NotificationError
pub fn map_channel_error(err: ChannelError) -> NotificationError {
    NotificationError::ChannelError(err)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockChannel;

    #[async_trait]
    impl NotificationChannel for MockChannel {
        fn channel_type(&self) -> &'static str {
            "mock"
        }

        fn display_name(&self) -> &'static str {
            "Mock Channel"
        }

        async fn send(&self, _input: &HookInput, _config: &ChannelConfig) -> ChannelResult<()> {
            Ok(())
        }
    }

    #[test]
    fn test_channel_type() {
        let channel = MockChannel;
        assert_eq!(channel.channel_type(), "mock");
        assert_eq!(channel.display_name(), "Mock Channel");
    }

    #[test]
    fn test_is_enabled() {
        let channel = MockChannel;
        let config_enabled = ChannelConfig {
            enabled: true,
            ..Default::default()
        };
        let config_disabled = ChannelConfig {
            enabled: false,
            ..Default::default()
        };

        assert!(channel.is_enabled(&config_enabled));
        assert!(!channel.is_enabled(&config_disabled));
    }

    #[tokio::test]
    async fn test_validate_config() {
        let channel = MockChannel;
        let config_enabled = ChannelConfig {
            enabled: true,
            ..Default::default()
        };
        let config_disabled = ChannelConfig {
            enabled: false,
            ..Default::default()
        };

        assert!(channel.validate_config(&config_enabled).is_ok());
        assert!(matches!(
            channel.validate_config(&config_disabled),
            Err(ChannelError::DisabledError)
        ));
    }
}
