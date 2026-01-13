//! Configuration file loading and saving
//!
//! This module handles loading configuration from ~/.claude-code-notifications.json
//! and saving configuration updates.

use crate::config::schema::AppConfig;
use crate::error::{NotificationError, Result};
use dirs::home_dir;
use std::fs;
use std::path::PathBuf;

/// Get the default configuration file path
/// Returns ~/.claude-code-notifications.json
pub fn get_config_path() -> PathBuf {
    home_dir()
        .expect("Unable to determine home directory")
        .join(".claude-code-notifications.json")
}

/// Load configuration from the default path
/// If the file doesn't exist, returns a default configuration
pub fn load_config() -> Result<AppConfig> {
    load_config_from_path(&get_config_path())
}

/// Load configuration from a specific path
pub fn load_config_from_path(path: &PathBuf) -> Result<AppConfig> {
    if !path.exists() {
        return Ok(default_config());
    }

    let content = fs::read_to_string(path).map_err(|e| {
        NotificationError::ConfigError(format!("Failed to read config file: {}", e))
    })?;

    let config: AppConfig = serde_json::from_str(&content).map_err(|e| {
        NotificationError::ConfigError(format!("Failed to parse config JSON: {}", e))
    })?;

    Ok(config)
}

/// Save configuration to the default path
pub fn save_config(config: &AppConfig) -> Result<()> {
    save_config_to_path(config, &get_config_path())
}

/// Save configuration to a specific path
pub fn save_config_to_path(config: &AppConfig, path: &PathBuf) -> Result<()> {
    let content = serde_json::to_string_pretty(config).map_err(|e| {
        NotificationError::ConfigError(format!("Failed to serialize config: {}", e))
    })?;

    fs::write(path, content)
        .map_err(|e| NotificationError::ConfigError(format!("Failed to write config: {}", e)))?;

    Ok(())
}

/// Create a default configuration
pub fn default_config() -> AppConfig {
    let value = serde_json::json!({
        "version": "1.0",
        "default_channels": ["system"],
        "channels": {
            "system": {
                "name": "系统通知",
                "channel_type": "system",
                "enabled": true,
                "sound": "Glass",
                "timeout_ms": 5000
            }
        },
        "routing_rules": [],
        "global_templates": {
            "default": {
                "title": "{{hook_type}}",
                "body": "{{message}}"
            },
            "PreToolUse": {
                "title": "Tool: {{tool_name}}",
                "body": "{{context}}"
            },
            "Stop": {
                "title": "Claude Stopped",
                "body": "{{reason}}"
            }
        }
    });
    serde_json::from_value(value).expect("Default config should be valid")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = default_config();
        assert_eq!(config.version, "1.0");
        assert_eq!(config.default_channels, vec!["system"]);
        assert!(config.channels.contains_key("system"));
        assert!(config.channels["system"].enabled);
    }

    #[test]
    fn test_save_and_load_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();

        let original_config = default_config();
        save_config_to_path(&original_config, &path).unwrap();

        let loaded_config = load_config_from_path(&path).unwrap();
        assert_eq!(original_config.version, loaded_config.version);
        assert_eq!(
            original_config.default_channels,
            loaded_config.default_channels
        );
    }

    #[test]
    fn test_load_nonexistent_config_returns_default() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();
        let _ = fs::remove_file(&path); // Delete the file

        let config = load_config_from_path(&path).unwrap();
        assert_eq!(config.version, "1.0");
    }
}
