//! Error types for claude-code-notifications
//!
//! This module defines structured error types using the `thiserror` crate
//! to provide comprehensive error handling for the notification system.

use std::io;
use thiserror::Error;

/// Main error type for the claude-code-notifications application
#[derive(Error, Debug)]
pub enum NotificationError {
    /// Error occurred during JSON parsing or validation
    #[error("JSON parsing error: {0}")]
    JsonParseError(#[from] serde_json::Error),

    /// Error occurred while reading from stdin
    #[error("I/O error reading input: {0}")]
    IoError(#[from] io::Error),

    /// Error occurred while displaying desktop notification
    #[error("Notification display error: {0}")]
    NotificationError(#[from] notify_rust::error::Error),

    /// Error occurred while playing sound
    #[error("Sound playback error: {0}")]
    SoundError(String),

    /// Invalid sound parameter provided
    #[error("Invalid sound parameter: {0}")]
    InvalidSoundParameter(String),

    /// Missing required field in JSON input
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Invalid JSON input structure
    #[error("Invalid JSON input: {0}")]
    InvalidInput(String),

    /// Channel-specific error
    #[error("Channel error: {0}")]
    ChannelError(#[from] ChannelError),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Routing rule error
    #[error("Routing error: {0}")]
    RoutingError(String),

    /// Webhook request error
    #[error("Webhook error: {0}")]
    WebhookError(String),

    /// Template rendering error
    #[error("Template error: {0}")]
    TemplateError(String),

    /// Transcript parsing error
    #[error("Transcript parsing error: {0}")]
    TranscriptError(String),

    /// Analysis error
    #[error("Transcript analysis error: {0}")]
    AnalysisError(String),
}

/// Channel-specific errors
#[derive(Error, Debug)]
pub enum ChannelError {
    #[error("HTTP client error: {0}")]
    HttpError(String),

    #[error("Webhook response error: {0}")]
    WebhookResponseError(String),

    #[error("Invalid channel configuration: {0}")]
    InvalidConfig(String),

    #[error("Channel disabled by configuration")]
    DisabledError,

    #[error("Channel not found: {0}")]
    NotFound(String),

    #[error("Channel operation timeout")]
    Timeout,
}

/// Result type alias for the notification system
pub type Result<T> = std::result::Result<T, NotificationError>;
