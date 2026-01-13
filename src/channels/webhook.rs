//! Generic webhook client for sending notifications
//!
//! This module provides a reusable HTTP client for webhook-based notifications
//! with support for custom headers and timeout configuration.

use async_trait::async_trait;
use reqwest::Client;
use serde::Serialize;
use std::time::Duration;

use crate::error::{ChannelError, NotificationError};

/// Generic webhook client for HTTP-based notifications
pub struct WebhookClient {
    client: Client,
    timeout: Duration,
}

impl WebhookClient {
    /// Create a new webhook client with default timeout (3 seconds for faster response)
    pub fn new() -> Result<Self, NotificationError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .map_err(|e| NotificationError::WebhookError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            timeout: Duration::from_secs(3),
        })
    }

    /// Create a new webhook client with custom timeout
    pub fn with_timeout(timeout_secs: u64) -> Result<Self, NotificationError> {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .map_err(|e| NotificationError::WebhookError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            timeout: Duration::from_secs(timeout_secs),
        })
    }

    /// Send POST request to webhook URL
    pub async fn send<T: Serialize>(
        &self,
        url: &str,
        payload: &T,
    ) -> Result<WebhookResponse, ChannelError> {
        let response = self
            .client
            .post(url)
            .json(payload)
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    ChannelError::Timeout
                } else {
                    ChannelError::HttpError(e.to_string())
                }
            })?;

        let status = response.status();
        let body_result: std::result::Result<String, reqwest::Error> = response.text().await;
        let body = body_result.map_err(|e| ChannelError::HttpError(format!("Failed to read response body: {}", e)))?;

        if status.is_success() {
            Ok(WebhookResponse::Success(body))
        } else {
            Ok(WebhookResponse::Error(status.as_u16(), body))
        }
    }

    /// Send POST request with custom headers
    pub async fn send_with_headers<T: Serialize>(
        &self,
        url: &str,
        payload: &T,
        headers: Vec<(&str, &str)>,
    ) -> Result<WebhookResponse, ChannelError> {
        let mut request = self.client.post(url).json(payload);

        for (key, value) in headers {
            request = request.header(key, value);
        }

        let response = request
            .timeout(self.timeout)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    ChannelError::Timeout
                } else {
                    ChannelError::HttpError(e.to_string())
                }
            })?;

        let status = response.status();
        let body_result: std::result::Result<String, reqwest::Error> = response.text().await;
        let body = body_result.map_err(|e| ChannelError::HttpError(format!("Failed to read response body: {}", e)))?;

        if status.is_success() {
            Ok(WebhookResponse::Success(body))
        } else {
            Ok(WebhookResponse::Error(status.as_u16(), body))
        }
    }
}

impl Default for WebhookClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default webhook client")
    }
}

/// Response from webhook request
#[derive(Debug, Clone)]
pub enum WebhookResponse {
    /// Successful response with body
    Success(String),
    /// Error response with status code and body
    Error(u16, String),
}

impl WebhookResponse {
    /// Check if response was successful
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    /// Get response body if successful, None otherwise
    pub fn body_ok(&self) -> Option<&str> {
        match self {
            Self::Success(body) => Some(body),
            _ => None,
        }
    }

    /// Get error info if error, None otherwise
    pub fn error_info(&self) -> Option<(u16, &str)> {
        match self {
            Self::Error(code, body) => Some((*code, body)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_webhook_client_creation() {
        let client = WebhookClient::new();
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_webhook_client_with_custom_timeout() {
        let client = WebhookClient::with_timeout(10);
        assert!(client.is_ok());
    }

    #[test]
    fn test_webhook_response() {
        let success = WebhookResponse::Success("OK".to_string());
        assert!(success.is_success());
        assert_eq!(success.body_ok(), Some("OK"));
        assert_eq!(success.error_info(), None);

        let error = WebhookResponse::Error(404, "Not Found".to_string());
        assert!(!error.is_success());
        assert_eq!(error.body_ok(), None);
        assert_eq!(error.error_info(), Some((404, "Not Found")));
    }
}
