//! Feishu (Lark) webhook notification channel
//!
//! This module implements the NotificationChannel trait for Feishu/Lark webhooks,
//! supporting text, post, and other message types.

use async_trait::async_trait;
use serde::Serialize;
use std::collections::HashMap;

use crate::channels::r#trait::NotificationChannel;
use crate::channels::webhook::WebhookClient;
use crate::config::{ChannelConfig, MessageTemplate, TemplateEngine};
use crate::error::{ChannelError, NotificationError};
use crate::hooks::HookInput;

/// Feishu/Lark webhook notification channel
pub struct FeishuChannel {
    client: WebhookClient,
}

impl FeishuChannel {
    pub fn new() -> Self {
        Self {
            client: WebhookClient::new().expect("Failed to create webhook client"),
        }
    }

    /// Build Feishu message from hook input and configuration
    fn build_message(&self, input: &HookInput, config: &ChannelConfig) -> Result<FeishuMessage, ChannelError> {
        // Use template engine to render message
        let template_engine = TemplateEngine::new(HashMap::new());
        let template = template_engine.get_template(&input.hook_type, config.message_template.as_ref());
        let rendered = template_engine.render(&template, input);

        Ok(FeishuMessage {
            msg_type: "text".to_string(),
            content: FeishuContent {
                text: rendered.body,
            },
        })
    }
}

impl Default for FeishuChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NotificationChannel for FeishuChannel {
    fn channel_type(&self) -> &'static str {
        "feishu"
    }

    fn display_name(&self) -> &'static str {
        "Feishu/Lark"
    }

    fn validate_config(&self, config: &ChannelConfig) -> Result<(), ChannelError> {
        if !config.enabled {
            return Err(ChannelError::DisabledError);
        }

        if config.webhook_url.is_none() || config.webhook_url.as_ref().unwrap().is_empty() {
            return Err(ChannelError::InvalidConfig(
                "webhook_url is required for Feishu".to_string(),
            ));
        }

        Ok(())
    }

    async fn send(&self, input: &HookInput, config: &ChannelConfig) -> Result<(), ChannelError> {
        let url = config
            .webhook_url
            .as_ref()
            .ok_or_else(|| ChannelError::InvalidConfig("webhook_url not configured".to_string()))?;

        let message = self.build_message(input, config)?;

        self.client.send(url, &message).await?;

        Ok(())
    }

    async fn test(&self, config: &ChannelConfig) -> Result<String, ChannelError> {
        self.validate_config(config)?;

        let test_input = HookInput::notification(
            "test-session".to_string(),
            None,
            "Feishu webhook test successful! 🚀".to_string(),
            Some("Feishu Test".to_string()),
        );

        let url = config.webhook_url.as_ref().unwrap();
        let message = self.build_message(&test_input, config)?;

        let response: crate::channels::webhook::WebhookResponse = self.client.send(url, &message).await?;
        if response.is_success() {
            Ok("Feishu webhook test successful".to_string())
        } else {
            let (code, body) = response.error_info().unwrap();
            Err(ChannelError::WebhookResponseError(format!(
                "HTTP {}: {}",
                code, body
            )))
        }
    }
}

/// Feishu message format
#[derive(Debug, Serialize)]
struct FeishuMessage {
    msg_type: String,
    content: FeishuContent,
}

/// Feishu text message content
#[derive(Debug, Serialize)]
struct FeishuContent {
    text: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_type() {
        let channel = FeishuChannel::new();
        assert_eq!(channel.channel_type(), "feishu");
        assert_eq!(channel.display_name(), "Feishu/Lark");
    }

    #[test]
    fn test_validate_config() {
        let channel = FeishuChannel::new();

        let config_valid = ChannelConfig {
            enabled: true,
            webhook_url: Some("https://open.feishu.cn/open-apis/bot/v2/hook/test".to_string()),
            ..Default::default()
        };
        assert!(channel.validate_config(&config_valid).is_ok());

        let config_no_url = ChannelConfig {
            enabled: true,
            webhook_url: None,
            ..Default::default()
        };
        assert!(channel.validate_config(&config_no_url).is_err());

        let config_disabled = ChannelConfig {
            enabled: false,
            ..Default::default()
        };
        assert!(matches!(
            channel.validate_config(&config_disabled),
            Err(ChannelError::DisabledError)
        ));
    }

    #[test]
    fn test_build_message() {
        let channel = FeishuChannel::new();

        let config = ChannelConfig {
            enabled: true,
            webhook_url: Some("https://test.com".to_string()),
            message_template: Some(MessageTemplate {
                title: Some("{{hook_type}}".to_string()),
                body: Some("Test: {{message}}".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };

        let input = HookInput::notification(
            "test".to_string(),
            None,
            "Hello Feishu".to_string(),
            Some("Test".to_string()),
        );

        let message = channel.build_message(&input, &config).unwrap();
        assert_eq!(message.msg_type, "text");
        assert_eq!(message.content.text, "Test: Hello Feishu");
    }
}
