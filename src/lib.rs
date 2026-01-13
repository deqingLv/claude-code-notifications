//! Core notification logic for claude-code-notifications
//!
//! This module contains the main functionality for receiving JSON input
//! from Claude Code hooks and displaying desktop notifications with
//! optional sound playback.

mod config;
mod channels;
mod router;
mod error;
mod hooks;
mod web;

use std::io::Read;
use std::time::Duration;
use std::process::Command;
use std::thread;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use notify_rust::{Notification, Timeout};
use tokio::runtime::Runtime;

pub use config::*;
pub use channels::*;
pub use router::ChannelRouter;
pub use error::{NotificationError, Result};
pub use hooks::*;
pub use web::start_web_server;

/// JSON input structure received from Claude Code hooks
#[derive(Debug, Deserialize, Serialize)]
pub struct NotificationInput {
    /// Claude session identifier
    pub session_id: String,
    /// Optional path to session transcript file
    #[serde(default)]
    pub transcript_path: Option<String>,
    /// Notification body text
    pub message: String,
    /// Optional notification title (defaults to "Claude Code")
    pub title: Option<String>,
}

/// Sound system configuration
pub struct SoundSystem;

impl SoundSystem {
    /// Play a sound using the system's audio player
    ///
    /// The sound parameter supports intelligent path resolution:
    /// - System sounds: `{SoundName}` (no path separators) resolves to `/System/Library/Sounds/{SoundName}.aiff`
    /// - Custom audio files: `{/path/to/file}` (contains path separators) supports various audio formats
    ///
    /// # Examples
    /// ```ignore
    /// use claude_code_notifications::SoundSystem;
    ///
    /// // System sound (requires macOS with system sounds)
    /// SoundSystem::play_sound("Glass").unwrap();
    ///
    /// // Custom sound file - example paths (commented out for doc tests)
    /// # // SoundSystem::play_sound("./assets/notification.wav").unwrap();
    /// # // SoundSystem::play_sound("/path/to/custom/sound.wav").unwrap();
    /// ```
    pub fn play_sound(sound: &str) -> Result<()> {
        let sound_path = Self::resolve_sound_path(sound)?;

        // Spawn a thread to play sound asynchronously
        let sound_path_clone = sound_path.clone();
        thread::spawn(move || {
            match Command::new("/usr/bin/afplay")
                .arg(&sound_path_clone)
                .status()
            {
                Ok(status) if !status.success() => {
                    eprintln!("afplay exited with non-zero status: {}", status);
                }
                Err(e) => {
                    eprintln!("Failed to execute afplay: {}", e);
                }
                _ => {} // Success - no output needed
            }
        });

        Ok(())
    }

    /// Resolve sound parameter to actual file path
    fn resolve_sound_path(sound: &str) -> Result<String> {
        // Check if sound contains path separators (custom file)
        if sound.contains('/') || sound.contains('\\') || sound.contains('~') {
            // Custom sound file - expand tilde if present
            let expanded_path = shellexpand::full(sound)
                .map_err(|e| NotificationError::InvalidSoundParameter(e.to_string()))?
                .to_string();

            // Verify file exists
            if !std::path::Path::new(&expanded_path).exists() {
                return Err(NotificationError::InvalidSoundParameter(
                    format!("Sound file not found: {}", expanded_path)
                ));
            }

            Ok(expanded_path)
        } else {
            // System sound - construct path to system sounds
            let system_sound_path = format!("/System/Library/Sounds/{}.aiff", sound);

            // Verify system sound exists
            if !std::path::Path::new(&system_sound_path).exists() {
                return Err(NotificationError::InvalidSoundParameter(
                    format!("System sound not found: {}. Available sounds: Glass, Submarine, Frog, Purr, Basso, Blow, Bottle, Funk, Hero, Morse, Ping, Pop, Sosumi, Tink", sound)
                ));
            }

            Ok(system_sound_path)
        }
    }
}

/// Channel manager for multi-channel notification dispatch
///
/// The ChannelManager coordinates notification delivery across multiple channels,
/// applying routing rules and handling errors gracefully.
pub struct ChannelManager {
    registry: ChannelRegistry,
    config: AppConfig,
    router: ChannelRouter,
}

impl ChannelManager {
    /// Create a new channel manager by loading configuration
    pub fn load() -> Result<Self> {
        let config = load_config()?;
        Self::from_config(config)
    }

    /// Create a new channel manager from a specific configuration
    pub fn from_config(config: AppConfig) -> Result<Self> {
        let registry = ChannelRegistry::new();
        let router = ChannelRouter::new(&config);

        Ok(Self {
            registry,
            config,
            router,
        })
    }

    /// Send notification through appropriate channels
    ///
    /// This method determines which channels should receive the notification
    /// based on routing rules and sends to all matched channels in parallel.
    pub fn send_notification(&self, input: &HookInput) -> Result<()> {
        let runtime = Runtime::new()?;
        runtime.block_on(self.send_notification_async(input))
    }

    /// Send notification through appropriate channels (async version)
    pub async fn send_notification_async(&self, input: &HookInput) -> Result<()> {
        // Match channels based on routing rules
        let matched_channels = self.router.match_channels(input, &self.config)?;

        // Wrap input in Arc for safe sharing across tasks
        let input = Arc::new(input.clone());

        // Send to all matched channels in parallel
        let mut tasks = Vec::new();

        for channel_id in matched_channels {
            // Get channel configuration
            let channel_config = self.config.channels.get(&channel_id)
                .cloned()
                .unwrap_or_default();

            // Get the channel type from config (defaults to channel_id for backward compatibility)
            let channel_type = if channel_config.channel_type.is_empty() {
                channel_id.clone()
            } else {
                channel_config.channel_type.clone()
            };

            // Create a new channel instance based on channel_type
            if let Some(channel) = self.registry.create_channel(&channel_type) {
                // Skip disabled channels
                if !channel.is_enabled(&channel_config) {
                    continue;
                }

                let input = Arc::clone(&input);

                tasks.push(tokio::spawn(async move {
                    let result = channel.send(&*input, &channel_config).await;
                    (channel_id, result)
                }));
            }
        }

        // Wait for all tasks with timeout (2 seconds - system channel is instant, webhooks run in background)
        if tasks.is_empty() {
            eprintln!("Warning: No enabled channels found for notification");
            return Ok(());
        }

        let results = tokio::time::timeout(
            Duration::from_secs(2),
            futures::future::join_all(tasks),
        ).await;

        match results {
            Ok(task_results) => {
                // Log errors but don't fail on partial failures
                for task_result in task_results {
                    if let Ok((channel_type, result)) = task_result {
                        if let Err(e) = result {
                            eprintln!("Channel {} error: {}", channel_type, e);
                        }
                    }
                }
                Ok(())
            }
            Err(e) => {
                eprintln!("Timeout waiting for channels: {}", e);
                Ok(()) // Don't fail on timeout
            }
        }
    }

    /// Send notification to specific channels (bypasses routing rules)
    pub fn send_to_channels(&self, input: &HookInput, channel_ids: Vec<String>) -> Result<()> {
        let runtime = Runtime::new()?;
        runtime.block_on(self.send_to_channels_async(input, channel_ids))
    }

    /// Send notification to specific channels (async version)
    pub async fn send_to_channels_async(&self, input: &HookInput, channel_ids: Vec<String>) -> Result<()> {
        // Deduplicate channels
        let channel_ids = self.router.override_channels(channel_ids);

        // Wrap input in Arc for safe sharing across tasks
        let input = Arc::new(input.clone());

        let mut tasks = Vec::new();

        for channel_id in channel_ids {
            // Get channel configuration
            let channel_config = self.config.channels.get(&channel_id)
                .cloned()
                .unwrap_or_default();

            // Get the channel type from config (defaults to channel_id for backward compatibility)
            let channel_type = if channel_config.channel_type.is_empty() {
                channel_id.clone()
            } else {
                channel_config.channel_type.clone()
            };

            // Create a new channel instance based on channel_type
            if let Some(channel) = self.registry.create_channel(&channel_type) {
                // Skip disabled channels
                if !channel.is_enabled(&channel_config) {
                    eprintln!("Warning: Channel {} is not enabled", channel_id);
                    continue;
                }

                let input = Arc::clone(&input);

                tasks.push(tokio::spawn(async move {
                    let result = channel.send(&*input, &channel_config).await;
                    (channel_id, result)
                }));
            }
        }

        // Wait for all tasks with timeout (2 seconds - system channel is instant, webhooks run in background)
        if tasks.is_empty() {
            eprintln!("Warning: No valid channels specified");
            return Ok(());
        }

        let results = tokio::time::timeout(
            Duration::from_secs(2),
            futures::future::join_all(tasks),
        ).await;

        match results {
            Ok(task_results) => {
                for task_result in task_results {
                    if let Ok((channel_type, result)) = task_result {
                        if let Err(e) = result {
                            eprintln!("Channel {} error: {}", channel_type, e);
                        }
                    }
                }
                Ok(())
            }
            Err(e) => {
                eprintln!("Timeout waiting for channels: {}", e);
                Ok(())
            }
        }
    }
}

/// Handle a hook input with optional sound (legacy mode for backward compatibility)
///
/// This function displays appropriate notifications based on the hook type
/// and optionally plays a sound in parallel.
pub fn handle_hook(input: &HookInput, sound: Option<&str>) -> Result<()> {
    // Validate required fields
    if input.common.session_id.is_empty() {
        return Err(NotificationError::MissingField("session_id".to_string()));
    }

    // Prepare notification title and body based on hook type
    let (title, body) = match &input.data {
        HookData::Notification(data) => {
            let title = data.title.as_deref().unwrap_or("Claude Code");
            let body = data.message.clone();
            (title, body)
        }
        HookData::PreToolUse(data) => {
            let title = "Claude Code - PreToolUse";
            let body = data.tool_name.clone();
            (title, body)
        }
        HookData::Stop(data) => {
            let title = "Claude Code - Stop";
            let body = data.reason.as_deref().unwrap_or("Claude stopped generating").to_string();
            (title, body)
        }
        HookData::SubagentStop(data) => {
            let title = "Claude Code - SubagentStop";
            let body = match (&data.subagent_id, &data.reason) {
                (Some(id), Some(reason)) => format!("Subagent {} stopped: {}", id, reason),
                (Some(id), None) => format!("Subagent {} stopped", id),
                (None, Some(reason)) => format!("Subagent stopped: {}", reason),
                (None, None) => "Subagent stopped".to_string(),
            };
            (title, body)
        }
    };

    // Create notification
    let mut notification = Notification::new();
    notification.summary(title);
    notification.body(&body);
    notification.timeout(Timeout::Milliseconds(5000)); // 5 second timeout

    // Display notification
    notification.show()?;

    // Play sound if specified
    if let Some(sound_param) = sound {
        // Spawn sound in separate thread - don't block on errors
        let sound_param = sound_param.to_string();
        thread::spawn(move || {
            if let Err(e) = SoundSystem::play_sound(&sound_param) {
                eprintln!("Warning: Sound playback failed: {}", e);
            }
        });
    }

    Ok(())
}

/// Display a desktop notification with optional sound
///
/// This function displays a desktop notification and optionally plays a sound
/// in parallel. The notification and sound playback happen concurrently.
/// Maintains backward compatibility with the old NotificationInput format.
pub fn send_notification(input: &NotificationInput, sound: Option<&str>) -> Result<()> {
    // Convert legacy NotificationInput to HookInput
    let hook_input = HookInput::notification(
        input.session_id.clone(),
        input.transcript_path.clone(),
        input.message.clone(),
        input.title.clone(),
    );
    handle_hook(&hook_input, sound)
}

/// Parse JSON input from stdin
///
/// Supports both the new HookInput format and the legacy NotificationInput format
/// for backward compatibility.
pub fn parse_input() -> Result<HookInput> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    if input.trim().is_empty() {
        return Err(NotificationError::InvalidInput(
            "Empty input received".to_string()
        ));
    }

    // First try to parse as the new HookInput format
    match serde_json::from_str::<HookInput>(&input) {
        Ok(hook_input) => Ok(hook_input),
        Err(_) => {
            // If that fails, try to parse as legacy NotificationInput format
            match serde_json::from_str::<NotificationInput>(&input) {
                Ok(notification_input) => {
                    // Convert legacy format to new format
                    Ok(HookInput::notification(
                        notification_input.session_id,
                        notification_input.transcript_path,
                        notification_input.message,
                        notification_input.title,
                    ))
                }
                Err(e) => {
                    // Return the original HookInput parse error for better diagnostics
                    // but try to parse again to get the actual error
                    Err(NotificationError::JsonParseError(e))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_valid_input() {
        let json = r#"{
            "session_id": "test-session",
            "transcript_path": "/tmp/test.md",
            "message": "Test notification",
            "title": "Test Title"
        }"#;

        let input: NotificationInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.session_id, "test-session");
        assert_eq!(input.transcript_path, Some("/tmp/test.md".to_string()));
        assert_eq!(input.message, "Test notification");
        assert_eq!(input.title, Some("Test Title".to_string()));
    }

    #[test]
    fn test_parse_input_without_title() {
        let json = r#"{
            "session_id": "test-session",
            "transcript_path": "/tmp/test.md",
            "message": "Test notification"
        }"#;

        let input: NotificationInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.session_id, "test-session");
        assert_eq!(input.transcript_path, Some("/tmp/test.md".to_string()));
        assert_eq!(input.message, "Test notification");
        assert_eq!(input.title, None);
    }

    #[test]
    fn test_parse_invalid_json() {
        let json = r#"{"invalid": json}"#;
        let result: std::result::Result<NotificationInput, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_input_without_transcript_path() {
        let json = r#"{
            "session_id": "test-session",
            "message": "Test notification without transcript"
        }"#;

        let input: NotificationInput = serde_json::from_str(json).unwrap();
        assert_eq!(input.session_id, "test-session");
        assert_eq!(input.transcript_path, None);
        assert_eq!(input.message, "Test notification without transcript");
        assert_eq!(input.title, None);
    }

    #[test]
    fn test_sound_path_resolution_system() {
        // This test will only pass on macOS with system sounds available
        #[cfg(target_os = "macos")]
        {
            let result = SoundSystem::resolve_sound_path("Glass");
            assert!(result.is_ok());
            let path = result.unwrap();
            assert!(path.contains("/System/Library/Sounds/Glass.aiff"));
        }
    }

    #[test]
    fn test_sound_path_resolution_custom() {
        // Create a temporary file for testing
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_str().unwrap();

        let result = SoundSystem::resolve_sound_path(temp_path);
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path, temp_path);
    }

    #[test]
    fn test_sound_path_resolution_invalid() {
        let result = SoundSystem::resolve_sound_path("/nonexistent/file.wav");
        assert!(result.is_err());

        let result = SoundSystem::resolve_sound_path("NonExistentSound");
        assert!(result.is_err());
    }
}