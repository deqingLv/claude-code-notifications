//! Core notification logic for claude-code-notifications
//!
//! This module contains the main functionality for receiving JSON input
//! from Claude Code hooks and displaying desktop notifications with
//! optional sound playback.

mod error;

use std::io::Read;
use std::process::Command;
use std::thread;
use serde::{Deserialize, Serialize};
use notify_rust::{Notification, Timeout};

pub use error::{NotificationError, Result};

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

/// Display a desktop notification with optional sound
///
/// This function displays a desktop notification and optionally plays a sound
/// in parallel. The notification and sound playback happen concurrently.
pub fn send_notification(input: &NotificationInput, sound: Option<&str>) -> Result<()> {
    // Validate required fields
    if input.session_id.is_empty() {
        return Err(NotificationError::MissingField("session_id".to_string()));
    }
    if input.message.is_empty() {
        return Err(NotificationError::MissingField("message".to_string()));
    }

    let title = input.title.as_deref().unwrap_or("Claude Code");

    // Create notification
    let mut notification = Notification::new();
    notification.summary(title);
    notification.body(&input.message);
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

/// Parse JSON input from stdin
pub fn parse_input() -> Result<NotificationInput> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    if input.trim().is_empty() {
        return Err(NotificationError::InvalidInput(
            "Empty input received".to_string()
        ));
    }

    let notification_input: NotificationInput = serde_json::from_str(&input)?;
    Ok(notification_input)
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