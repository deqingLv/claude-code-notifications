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
}

/// Result type alias for the notification system
pub type Result<T> = std::result::Result<T, NotificationError>;