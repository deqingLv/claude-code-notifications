//! Hook type definitions for claude-code-notifications
//!
//! This module defines the different hook types that Claude Code can send
//! and their corresponding data structures.

use serde::{Deserialize, Serialize};

/// Type of hook being invoked by Claude Code
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum HookType {
    /// Desktop notification hook
    Notification,
    /// Hook invoked before a tool is used
    PreToolUse,
    /// Hook invoked when Claude stops generating
    Stop,
    /// Hook invoked when a subagent stops
    SubagentStop,
}

/// Common fields present in all hook types
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CommonHookFields {
    /// Claude session identifier
    pub session_id: String,
    /// Optional path to session transcript file
    #[serde(default)]
    pub transcript_path: Option<String>,
    /// Current working directory when hook is invoked
    #[serde(default)]
    pub cwd: Option<String>,
    /// Current permission mode: "default", "plan", "acceptEdits", "dontAsk", or "bypassPermissions"
    #[serde(default)]
    pub permission_mode: Option<String>,
}

/// Data specific to Notification hooks
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NotificationData {
    /// Notification body text
    pub message: String,
    /// Optional notification title (defaults to "Claude Code")
    pub title: Option<String>,
    /// Type of notification (optional, currently not sent by Claude Code due to bug #11964)
    /// Expected values: "permission_prompt", "idle_prompt", "auth_success", "elicitation_dialog"
    #[serde(default)]
    pub notification_type: Option<String>,
}

/// Data specific to PreToolUse hooks
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PreToolUseData {
    /// Name of the tool being invoked
    pub tool_name: String,
    /// Tool input parameters (schema depends on the specific tool)
    #[serde(default)]
    pub tool_input: Option<serde_json::Value>,
    /// Unique identifier for this tool use
    #[serde(default)]
    pub tool_use_id: Option<String>,
}

/// Data specific to Stop hooks
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StopData {
    /// Whether a stop hook is already active (prevents infinite loops)
    #[serde(default)]
    pub stop_hook_active: Option<bool>,
}

/// Data specific to SubagentStop hooks
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SubagentStopData {
    /// Whether a stop hook is already active (prevents infinite loops)
    #[serde(default)]
    pub stop_hook_active: Option<bool>,
}

/// Enum representing the type-specific data for each hook type
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum HookData {
    /// Notification-specific data
    Notification(NotificationData),
    /// PreToolUse-specific data
    PreToolUse(PreToolUseData),
    /// Stop-specific data
    Stop(StopData),
    /// SubagentStop-specific data
    SubagentStop(SubagentStopData),
}

/// Complete hook input structure
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HookInput {
    /// Type of hook being invoked
    /// Supports both "hook_type" (primary) and "hook_event_name" (alternative) field names
    #[serde(alias = "hook_event_name")]
    pub hook_type: HookType,
    /// Common fields present in all hook types
    #[serde(flatten)]
    pub common: CommonHookFields,
    /// Type-specific data (depends on hook_type)
    #[serde(flatten)]
    pub data: HookData,
}

impl HookInput {
    /// Create a notification hook input (for testing)
    pub fn notification(
        session_id: String,
        transcript_path: Option<String>,
        message: String,
        title: Option<String>,
    ) -> Self {
        Self {
            hook_type: HookType::Notification,
            common: CommonHookFields {
                session_id,
                transcript_path,
                cwd: None,
                permission_mode: None,
            },
            data: HookData::Notification(NotificationData {
                message,
                title,
                notification_type: None,
            }),
        }
    }

    /// Create a PreToolUse hook input (for testing)
    pub fn pre_tool_use(
        session_id: String,
        transcript_path: Option<String>,
        tool_name: String,
        _context: Option<String>,
    ) -> Self {
        Self {
            hook_type: HookType::PreToolUse,
            common: CommonHookFields {
                session_id,
                transcript_path,
                cwd: None,
                permission_mode: None,
            },
            data: HookData::PreToolUse(PreToolUseData {
                tool_name,
                tool_input: None,
                tool_use_id: None,
            }),
        }
    }

    /// Create a Stop hook input (for testing)
    pub fn stop(
        session_id: String,
        transcript_path: Option<String>,
        _reason: Option<String>,
    ) -> Self {
        Self {
            hook_type: HookType::Stop,
            common: CommonHookFields {
                session_id,
                transcript_path,
                cwd: None,
                permission_mode: None,
            },
            data: HookData::Stop(StopData {
                stop_hook_active: None,
            }),
        }
    }

    /// Create a SubagentStop hook input (for testing)
    pub fn subagent_stop(
        session_id: String,
        transcript_path: Option<String>,
        _subagent_id: Option<String>,
        _reason: Option<String>,
    ) -> Self {
        Self {
            hook_type: HookType::SubagentStop,
            common: CommonHookFields {
                session_id,
                transcript_path,
                cwd: None,
                permission_mode: None,
            },
            data: HookData::SubagentStop(SubagentStopData {
                stop_hook_active: None,
            }),
        }
    }
}
