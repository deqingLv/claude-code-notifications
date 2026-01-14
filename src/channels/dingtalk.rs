//! DingTalk webhook notification channel
//!
//! This module implements the NotificationChannel trait for DingTalk webhooks,
//! supporting text messages and optional webhook signing with secret.
#![allow(unused_imports)]

use async_trait::async_trait;
use hmac::{Hmac, Mac};
use serde::Serialize;
use sha2::Sha256;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::channels::r#trait::NotificationChannel;
use crate::channels::webhook::WebhookClient;
use crate::config::{ChannelConfig, MessageTemplate, TemplateEngine};
use crate::error::ChannelError;
use crate::hooks::HookInput;

/// DingTalk webhook notification channel
pub struct DingTalkChannel {
    client: WebhookClient,
}

impl DingTalkChannel {
    pub fn new() -> Self {
        Self {
            client: WebhookClient::new().expect("Failed to create webhook client"),
        }
    }

    /// Generate DingTalk webhook signature
    ///
    /// DingTalk webhooks can be signed with a secret for additional security.
    /// The signature is calculated as: base64(hmac_sha256(secret, timestamp + "\n" + secret))
    fn generate_signature(secret: &str, timestamp: u64) -> String {
        let string_to_sign = format!("{}\n{}", timestamp, secret);

        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(string_to_sign.as_bytes());
        let result = mac.finalize();
        let code = result.into_bytes();

        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(code)
    }

    /// Build DingTalk message from hook input and configuration
    fn build_message(
        &self,
        input: &HookInput,
        config: &ChannelConfig,
    ) -> Result<DingTalkMessage, ChannelError> {
        // Use template engine to render message
        let template_engine = TemplateEngine::new(HashMap::new());
        let template =
            template_engine.get_template(&input.hook_event_name, config.message_template.as_ref());
        let rendered = template_engine.render(&template, input);

        Ok(DingTalkMessage {
            msgtype: "text".to_string(),
            text: DingTalkText {
                content: rendered.body,
            },
        })
    }
}

impl Default for DingTalkChannel {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NotificationChannel for DingTalkChannel {
    fn channel_type(&self) -> &'static str {
        "dingtalk"
    }

    fn display_name(&self) -> &'static str {
        "DingTalk"
    }

    fn validate_config(&self, config: &ChannelConfig) -> Result<(), ChannelError> {
        if !config.enabled {
            return Err(ChannelError::DisabledError);
        }

        if config.webhook_url.is_none() || config.webhook_url.as_ref().unwrap().is_empty() {
            return Err(ChannelError::InvalidConfig(
                "webhook_url is required for DingTalk".to_string(),
            ));
        }

        Ok(())
    }

    async fn send(
        &self,
        input: &HookInput,
        config: &ChannelConfig,
        _template_engine: &TemplateEngine,
    ) -> Result<(), ChannelError> {
        let webhook_url = config
            .webhook_url
            .as_ref()
            .ok_or_else(|| ChannelError::InvalidConfig("webhook_url not configured".to_string()))?;

        let message = self.build_message(input, config)?;

        // Check if secret is configured for signing
        if let Some(secret) = &config.secret {
            if !secret.is_empty() {
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| ChannelError::InvalidConfig(format!("Time error: {}", e)))?
                    .as_millis() as u64;

                let sign = Self::generate_signature(secret, timestamp);

                // Append timestamp and sign to webhook URL
                let signed_url = format!("{}&timestamp={}&sign={}", webhook_url, timestamp, sign);

                self.client.send(&signed_url, &message).await?;
                return Ok(());
            }
        }

        self.client.send(webhook_url, &message).await?;

        Ok(())
    }

    async fn test(&self, config: &ChannelConfig) -> Result<String, ChannelError> {
        self.validate_config(config)?;

        let test_input = HookInput::notification(
            "test-session".to_string(),
            None,
            "DingTalk webhook test successful! 💬".to_string(),
            Some("DingTalk Test".to_string()),
        );

        let webhook_url = config.webhook_url.as_ref().unwrap();
        let message = self.build_message(&test_input, config)?;

        // Check if secret is configured
        let url = if let Some(secret) = &config.secret {
            if !secret.is_empty() {
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_err(|e| ChannelError::InvalidConfig(format!("Time error: {}", e)))?
                    .as_millis() as u64;

                let sign = Self::generate_signature(secret, timestamp);
                format!("{}&timestamp={}&sign={}", webhook_url, timestamp, sign)
            } else {
                webhook_url.clone()
            }
        } else {
            webhook_url.clone()
        };

        let response: crate::channels::webhook::WebhookResponse =
            self.client.send(&url, &message).await?;
        if response.is_success() {
            Ok("DingTalk webhook test successful".to_string())
        } else {
            let (code, body) = response.error_info().unwrap();
            Err(ChannelError::WebhookResponseError(format!(
                "HTTP {}: {}",
                code, body
            )))
        }
    }
}

/// DingTalk message format
#[derive(Debug, Serialize)]
struct DingTalkMessage {
    msgtype: String,
    text: DingTalkText,
}

/// DingTalk text message content
#[derive(Debug, Serialize)]
struct DingTalkText {
    content: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_type() {
        let channel = DingTalkChannel::new();
        assert_eq!(channel.channel_type(), "dingtalk");
        assert_eq!(channel.display_name(), "DingTalk");
    }

    #[test]
    fn test_validate_config() {
        let channel = DingTalkChannel::new();

        let config_valid = ChannelConfig {
            enabled: true,
            webhook_url: Some("https://oapi.dingtalk.com/robot/send?access_token=test".to_string()),
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
    fn test_generate_signature() {
        let secret = "SEC1234567890";
        let timestamp = 1600000000000;
        let signature = DingTalkChannel::generate_signature(secret, timestamp);

        // Signature should be base64 encoded
        assert!(!signature.is_empty());
        assert!(signature.len() > 20);
    }

    #[test]
    fn test_build_message() {
        let channel = DingTalkChannel::new();

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
            "Hello DingTalk".to_string(),
            Some("Test".to_string()),
        );

        let message = channel.build_message(&input, &config).unwrap();
        assert_eq!(message.msgtype, "text");
        assert_eq!(message.text.content, "Test: Hello DingTalk");
    }
}
