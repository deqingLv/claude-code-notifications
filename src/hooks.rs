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
    /// Hook invoked when permission dialog is shown
    PermissionRequest,
}

/// Common fields present in all hook types
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CommonHookFields {
    /// Claude session identifier
    pub session_id: String,
    /// Optional path to session transcript file
    #[serde(default)]
    pub transcript_path: Option<String>,
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
    /// Optional additional context about the tool use
    #[serde(default)]
    pub context: Option<String>,
}

/// Data specific to PermissionRequest hooks
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PermissionRequestData {
    /// Name of the tool requiring permission
    #[serde(default)]
    pub tool_name: Option<String>,
    /// Optional additional context about the permission request
    #[serde(default)]
    pub context: Option<String>,
}

/// Data specific to Stop hooks
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StopData {
    /// Optional reason for stopping
    #[serde(default)]
    pub reason: Option<String>,
}

/// Data specific to SubagentStop hooks
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SubagentStopData {
    /// Optional identifier of the subagent that stopped
    #[serde(default)]
    pub subagent_id: Option<String>,
    /// Optional reason for subagent stopping
    #[serde(default)]
    pub reason: Option<String>,
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
    /// PermissionRequest-specific data
    PermissionRequest(PermissionRequestData),
}

/// Complete hook input structure
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HookInput {
    /// Type of hook being invoked
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
        context: Option<String>,
    ) -> Self {
        Self {
            hook_type: HookType::PreToolUse,
            common: CommonHookFields {
                session_id,
                transcript_path,
            },
            data: HookData::PreToolUse(PreToolUseData { tool_name, context }),
        }
    }

    /// Create a Stop hook input (for testing)
    pub fn stop(
        session_id: String,
        transcript_path: Option<String>,
        reason: Option<String>,
    ) -> Self {
        Self {
            hook_type: HookType::Stop,
            common: CommonHookFields {
                session_id,
                transcript_path,
            },
            data: HookData::Stop(StopData { reason }),
        }
    }

    /// Create a SubagentStop hook input (for testing)
    pub fn subagent_stop(
        session_id: String,
        transcript_path: Option<String>,
        subagent_id: Option<String>,
        reason: Option<String>,
    ) -> Self {
        Self {
            hook_type: HookType::SubagentStop,
            common: CommonHookFields {
                session_id,
                transcript_path,
            },
            data: HookData::SubagentStop(SubagentStopData {
                subagent_id,
                reason,
            }),
        }
    }

    /// Create a PermissionRequest hook input (for testing)
    pub fn permission_request(
        session_id: String,
        transcript_path: Option<String>,
        tool_name: Option<String>,
        context: Option<String>,
    ) -> Self {
        Self {
            hook_type: HookType::PermissionRequest,
            common: CommonHookFields {
                session_id,
                transcript_path,
            },
            data: HookData::PermissionRequest(PermissionRequestData { tool_name, context }),
        }
    }
}
