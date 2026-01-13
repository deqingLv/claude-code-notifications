//! System/desktop notification channel
//!
//! This module implements the NotificationChannel trait for system desktop notifications
//! using notify-rust, with support for sound playback via afplay on macOS.

use async_trait::async_trait;
use notify_rust::Notification;
use std::thread;

use crate::channels::r#trait::NotificationChannel;
use crate::config::templates::TemplateEngine;
use crate::config::ChannelConfig;
use crate::error::{ChannelError, NotificationError};
use crate::hooks::HookInput;

/// System notification channel implementation
pub struct SystemChannel;

impl Default for SystemChannel {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemChannel {
    pub fn new() -> Self {
        Self
    }

    /// Play a sound using the system's audio player
    fn play_sound(sound: &str) -> Result<(), NotificationError> {
        let sound_path = Self::resolve_sound_path(sound)?;

        // Spawn a thread to play sound asynchronously
        let sound_path_clone = sound_path.clone();
        thread::spawn(move || {
            match std::process::Command::new("/usr/bin/afplay")
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
    fn resolve_sound_path(sound: &str) -> Result<String, NotificationError> {
        // Check if sound contains path separators (custom file)
        if sound.contains('/') || sound.contains('\\') || sound.contains('~') {
            // Custom sound file - expand tilde if present
            let expanded_path = shellexpand::full(sound)
                .map_err(|e| NotificationError::InvalidSoundParameter(e.to_string()))?
                .to_string();

            // Verify file exists
            if !std::path::Path::new(&expanded_path).exists() {
                return Err(NotificationError::InvalidSoundParameter(format!(
                    "Sound file not found: {}",
                    expanded_path
                )));
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

    /// Display a desktop notification
    fn display_notification(
        title: &str,
        body: &str,
        timeout_ms: u64,
        icon: Option<&str>,
    ) -> Result<(), NotificationError> {
        let mut notification = Notification::new();
        notification.summary(title);
        notification.body(body);
        notification.timeout(notify_rust::Timeout::Milliseconds(timeout_ms as u32));

        // Set icon if provided and valid
        if let Some(icon_name) = icon {
            // Try to resolve icon path
            if let Some(resolved_icon) = Self::resolve_icon_path(icon_name) {
                notification.icon(&resolved_icon);
            } else {
                // If icon is "Claude Code" but not found, silently use default system icon
                if icon_name != "Claude Code" {
                    eprintln!("Warning: Icon not found: {}", icon_name);
                }
            }
        }

        notification.show()?;
        Ok(())
    }

    /// Resolve icon name to file path
    fn resolve_icon_path(icon_name: &str) -> Option<String> {
        // If it's already a path (contains / or .), check if file exists
        if icon_name.contains('/') || icon_name.contains('.') {
            let expanded_path = shellexpand::full(icon_name).ok()?.to_string();
            let path = std::path::Path::new(&expanded_path);
            if path.exists() {
                return Some(expanded_path);
            }
            return None;
        }

        // Special case: "Claude Code" icon - check common locations
        if icon_name == "Claude Code" {
            // Check in assets directory relative to executable
            let exe_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()));

            // Possible icon file names and extensions
            let icon_files = [
                "claude-code.png",
                "claude-code.icns",
                "claude-code.ico",
                "claude.png",
                "claude.icns",
                "claude.ico",
            ];

            // Check multiple locations
            let locations = [
                // 1. Assets directory relative to executable
                exe_dir.as_ref().map(|p| p.join("assets")),
                // 2. Current directory
                Some(std::env::current_dir().unwrap_or_default().join("assets")),
                // 3. User's .claude directory
                dirs::home_dir().map(|p| p.join(".claude")),
                // 4. Project root (for development)
                Some(std::path::PathBuf::from(".")),
            ];

            for location in locations.iter().flatten() {
                for icon_file in &icon_files {
                    let icon_path = location.join(icon_file);
                    if icon_path.exists() {
                        return Some(icon_path.to_string_lossy().to_string());
                    }
                }
            }

            // Not found
            return None;
        }

        // For other icon names, treat as application name (macOS)
        // notify-rust will handle application names on macOS
        Some(icon_name.to_string())
    }
}

#[async_trait]
impl NotificationChannel for SystemChannel {
    fn channel_type(&self) -> &'static str {
        "system"
    }

    fn display_name(&self) -> &'static str {
        "System Notification"
    }

    fn validate_config(&self, config: &ChannelConfig) -> Result<(), ChannelError> {
        if !config.enabled {
            return Err(ChannelError::DisabledError);
        }

        // Validate sound if specified
        if let Some(sound) = &config.sound {
            if !sound.is_empty() {
                Self::resolve_sound_path(sound)
                    .map_err(|e| ChannelError::InvalidConfig(e.to_string()))?;
            }
        }

        Ok(())
    }

    async fn send(&self, input: &HookInput, config: &ChannelConfig) -> Result<(), ChannelError> {
        // Get timeout from config or use default
        let timeout_ms = config.timeout_ms.unwrap_or(5000);

        // Use template engine to render message
        let template_engine = TemplateEngine::new(std::collections::HashMap::new());
        let template =
            template_engine.get_template(&input.hook_type, config.message_template.as_ref());
        let rendered = template_engine.render(&template, input);

        // Get icon from config or use default Claude Code icon
        let icon = config.icon.as_deref().or(Some("Claude Code"));

        // Display notification
        Self::display_notification(&rendered.title, &rendered.body, timeout_ms, icon)
            .map_err(|e| ChannelError::InvalidConfig(e.to_string()))?;

        // Play sound if configured
        if let Some(sound) = &config.sound {
            if !sound.is_empty() {
                let sound = sound.clone();
                thread::spawn(move || {
                    if let Err(e) = Self::play_sound(&sound) {
                        eprintln!("Warning: Sound playback failed: {}", e);
                    }
                });
            }
        }

        Ok(())
    }

    async fn test(&self, config: &ChannelConfig) -> Result<String, ChannelError> {
        self.validate_config(config)?;

        // Send a test notification
        let test_input = HookInput::notification(
            "test-session".to_string(),
            None,
            "System channel test successful!".to_string(),
            Some("System Notification Test".to_string()),
        );

        self.send(&test_input, config).await?;
        Ok("System notification sent successfully".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_type() {
        let channel = SystemChannel::new();
        assert_eq!(channel.channel_type(), "system");
        assert_eq!(channel.display_name(), "System Notification");
    }

    #[test]
    fn test_sound_path_resolution_system() {
        #[cfg(target_os = "macos")]
        {
            let result = SystemChannel::resolve_sound_path("Glass");
            assert!(result.is_ok());
            let path = result.unwrap();
            assert!(path.contains("/System/Library/Sounds/Glass.aiff"));
        }
    }

    #[test]
    fn test_sound_path_resolution_custom() {
        use tempfile::NamedTempFile;
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_str().unwrap();

        let result = SystemChannel::resolve_sound_path(temp_path);
        assert!(result.is_ok());
        let path = result.unwrap();
        assert_eq!(path, temp_path);
    }

    #[test]
    fn test_sound_path_resolution_invalid() {
        let result = SystemChannel::resolve_sound_path("/nonexistent/file.wav");
        assert!(result.is_err());

        let result = SystemChannel::resolve_sound_path("NonExistentSound");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_config() {
        let channel = SystemChannel::new();

        let config_enabled = ChannelConfig {
            enabled: true,
            sound: Some("Glass".to_string()),
            ..Default::default()
        };

        #[cfg(target_os = "macos")]
        assert!(channel.validate_config(&config_enabled).is_ok());

        let config_disabled = ChannelConfig {
            enabled: false,
            ..Default::default()
        };
        assert!(matches!(
            channel.validate_config(&config_disabled),
            Err(ChannelError::DisabledError)
        ));
    }

    #[tokio::test]
    async fn test_send_notification() {
        let channel = SystemChannel::new();

        let config = ChannelConfig {
            enabled: true,
            sound: None,
            timeout_ms: Some(1000),
            ..Default::default()
        };

        let input = HookInput::notification(
            "test".to_string(),
            None,
            "Test message".to_string(),
            Some("Test Title".to_string()),
        );

        // This will display an actual notification in tests
        // In production, you might want to mock this
        let result = channel.send(&input, &config).await;
        #[cfg(not(target_os = "macos"))]
        assert!(result.is_err()); // notify-rust might not work on all platforms in tests

        #[cfg(target_os = "macos")]
        assert!(result.is_ok());
    }
}
