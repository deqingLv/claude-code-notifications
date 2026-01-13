//! WeChat Work webhook notification channel
//!
//! This module implements the NotificationChannel trait for WeChat Work (企业微信)
//! webhooks, supporting text messages and mentioned users.

use async_trait::async_trait;
use serde::Serialize;
use std::collections::HashMap;

use crate::channels::r#trait::NotificationChannel;
use crate::channels::webhook::WebhookClient;
use crate::config::{ChannelConfig, MessageTemplate, TemplateEngine};
use crate::error::{ChannelError, NotificationError};
use crate::hooks::HookInput;

/// WeChat Work webhook notification channel
pub struct WeChatChannel {
    client: WebhookClient,
}

impl WeChatChannel {
    pub fn new() -> Self {
        Self {
            client: WebhookClient::new().expect("Failed to create webhook client"),
        }
    }

    /// Build WeChat message from hook input and configuration
    fn build_message(&self, input: &HookInput, config: &ChannelConfig) -> Result<WeChatMessage, ChannelError> {
        // Use template engine to render message
        let template_engine = TemplateEngine::new(HashMap::new());
        let template = template_engine.get_template(&input.hook_type, config.message_template.as_ref());
        let rendered = template_engine.render(&template, input);

        // Get mentioned_list from template if available
        let mentioned_list = config
            .message_template
            .as_ref()
            .and_then(|t| t.mentioned_list.clone())
            .unwrap_or_default();

        Ok(WeChatMessage {
            msgtype: "text".to_string(),
            text: WeChatText {
                content: rendered.body,
                mentioned_list,
            },
        })
    }
}

impl Default for WeChatChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NotificationChannel for WeChatChannel {
    fn channel_type(&self) -> &'static str {
        "wechat"
    }

    fn display_name(&self) -> &'static str {
        "WeChat Work"
    }

    fn validate_config(&self, config: &ChannelConfig) -> Result<(), ChannelError> {
        if !config.enabled {
            return Err(ChannelError::DisabledError);
        }

        if config.webhook_url.is_none() || config.webhook_url.as_ref().unwrap().is_empty() {
            return Err(ChannelError::InvalidConfig(
                "webhook_url is required for WeChat Work".to_string(),
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
            "WeChat Work webhook test successful! 🎉".to_string(),
            Some("WeChat Work Test".to_string()),
        );

        let url = config.webhook_url.as_ref().unwrap();
        let message = self.build_message(&test_input, config)?;

        let response: crate::channels::webhook::WebhookResponse = self.client.send(url, &message).await?;
        if response.is_success() {
            Ok("WeChat Work webhook test successful".to_string())
        } else {
            let (code, body) = response.error_info().unwrap();
            Err(ChannelError::WebhookResponseError(format!(
                "HTTP {}: {}",
                code, body
            )))
        }
    }
}

/// WeChat Work message format
#[derive(Debug, Serialize)]
struct WeChatMessage {
    msgtype: String,
    #[serde(flatten)]
    text: WeChatText,
}

/// WeChat Work text message content
#[derive(Debug, Serialize)]
struct WeChatText {
    content: String,
    mentioned_list: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_type() {
        let channel = WeChatChannel::new();
        assert_eq!(channel.channel_type(), "wechat");
        assert_eq!(channel.display_name(), "WeChat Work");
    }

    #[test]
    fn test_validate_config() {
        let channel = WeChatChannel::new();

        let config_valid = ChannelConfig {
            enabled: true,
            webhook_url: Some("https://qyapi.weixin.qq.com/cgi-bin/webhook/send?key=test".to_string()),
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
        let channel = WeChatChannel::new();

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
            "Hello WeChat".to_string(),
            Some("Test".to_string()),
        );

        let message = channel.build_message(&input, &config).unwrap();
        assert_eq!(message.msgtype, "text");
        assert_eq!(message.text.content, "Test: Hello WeChat");
    }
}
