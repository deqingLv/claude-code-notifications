//! Transcript analysis engine for determining task status
//!
//! This module implements a state machine that analyzes Claude Code session
//! transcripts to determine task completion status and generate appropriate
//! notifications.

use crate::error::{NotificationError, Result};
use crate::transcript::*;

/// Task completion status determined by transcript analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// Task completed successfully with active tool usage
    TaskComplete,
    /// Code review completed (only passive tools + text analysis)
    ReviewComplete,
    /// Claude is asking the user a question
    Question,
    /// Plan is ready for review
    PlanReady,
    /// Session limit reached
    SessionLimitReached,
    /// API authentication error
    APIError,
    /// Unable to determine status
    Unknown,
}

/// Tool classification categories
pub struct ToolCategories;

impl ToolCategories {
    /// Tools that modify state or execute actions
    pub const ACTIVE_TOOLS: &'static [&'static str] = &[
        "Write",
        "Edit",
        "Bash",
        "NotebookEdit",
        "SlashCommand",
        "KillShell",
    ];

    /// Tools that ask user questions
    pub const QUESTION_TOOLS: &'static [&'static str] = &["AskUserQuestion"];

    /// Tools related to planning
    pub const PLANNING_TOOLS: &'static [&'static str] = &["ExitPlanMode", "TodoWrite"];

    /// Tools that read or query without modifying
    pub const PASSIVE_TOOLS: &'static [&'static str] = &[
        "Read",
        "Grep",
        "Glob",
        "WebFetch",
        "WebSearch",
        "Search",
        "Fetch",
        "Task",
    ];
}

/// Analyze transcript and determine task completion status
///
/// This function implements a priority-based state machine:
/// 1. Check for session limit (highest priority)
/// 2. Check for API authentication error
/// 3. Perform tool-based analysis with temporal filtering
pub fn analyze_transcript(transcript_path: &str) -> Result<Status> {
    // Parse transcript
    let messages = parse_file(transcript_path).map_err(|e| {
        NotificationError::AnalysisError(format!("Failed to parse transcript: {}", e))
    })?;

    if messages.is_empty() {
        return Ok(Status::Unknown);
    }

    // PRIORITY CHECK 1: Session limit reached
    if detect_session_limit_reached(&messages) {
        return Ok(Status::SessionLimitReached);
    }

    // PRIORITY CHECK 2: API authentication error
    if detect_api_error(&messages) {
        return Ok(Status::APIError);
    }

    // Find last user message timestamp for temporal filtering
    let user_ts = get_last_user_timestamp(&messages);

    // Filter messages after last user message (current response only)
    let filtered_messages = filter_messages_after_timestamp(&messages, &user_ts);

    if filtered_messages.is_empty() {
        return Ok(Status::Unknown);
    }

    // Take last 15 messages (temporal window for analysis)
    let recent_messages = if filtered_messages.len() > 15 {
        &filtered_messages[filtered_messages.len() - 15..]
    } else {
        &filtered_messages
    };

    // Extract tools from filtered messages
    let tools = extract_tools(recent_messages);

    // STATE MACHINE LOGIC
    if !tools.is_empty() {
        let last_tool = get_last_tool(&tools);

        // Check: Last tool is ExitPlanMode
        if last_tool == Some("ExitPlanMode") {
            return Ok(Status::PlanReady);
        }

        // Check: Last tool is AskUserQuestion
        if last_tool == Some("AskUserQuestion") {
            return Ok(Status::Question);
        }

        // Check: ExitPlanMode exists AND tools after it
        let exit_plan_pos = find_tool_position(&tools, "ExitPlanMode");
        if exit_plan_pos >= 0 {
            let tools_after = count_tools_after_position(&tools, exit_plan_pos);
            if tools_after > 0 {
                return Ok(Status::TaskComplete);
            }
        }

        // Check: Review detection (only read-like tools + long text)
        let read_like_count = count_tools_by_names(&tools, &["Read", "Grep", "Glob"]);
        let has_active_tools = has_any_active_tool(&tools, ToolCategories::ACTIVE_TOOLS);

        if read_like_count >= 1 && !has_active_tools {
            let recent_text = extract_recent_text(recent_messages, 5);
            if recent_text.len() > 200 {
                return Ok(Status::ReviewComplete);
            }
        }

        // Check: Last tool is active (Write/Edit/Bash/etc)
        if last_tool.is_some_and(|name| ToolCategories::ACTIVE_TOOLS.contains(&name)) {
            return Ok(Status::TaskComplete);
        }

        // Check: Any tool usage at all
        return Ok(Status::TaskComplete);
    }

    // No tools found
    Ok(Status::Unknown)
}

/// Detect session limit reached
///
/// Checks last 3 assistant messages for "session limit reached" patterns.
fn detect_session_limit_reached(messages: &[Message]) -> bool {
    let recent_messages = get_last_assistant_messages(messages, 3);
    let texts = extract_text_from_messages(&recent_messages);

    texts.iter().any(|text| {
        let lower = text.to_lowercase();
        lower.contains("session limit reached") || lower.contains("session limit has been reached")
    })
}

/// Detect API authentication error
///
/// Checks for both "API Error: 401" AND "run /login" in last 3 assistant messages.
fn detect_api_error(messages: &[Message]) -> bool {
    let recent_messages = get_last_assistant_messages(messages, 3);
    let texts = extract_text_from_messages(&recent_messages);

    let has_api_error = texts.iter().any(|text| {
        let lower = text.to_lowercase();
        lower.contains("api error: 401") || lower.contains("api error 401")
    });

    let has_login_prompt = texts.iter().any(|text| {
        let lower = text.to_lowercase();
        lower.contains("please run /login") || lower.contains("run /login")
    });

    has_api_error && has_login_prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_categories() {
        assert!(ToolCategories::ACTIVE_TOOLS.contains(&"Write"));
        assert!(ToolCategories::ACTIVE_TOOLS.contains(&"Edit"));
        assert!(ToolCategories::QUESTION_TOOLS.contains(&"AskUserQuestion"));
        assert!(ToolCategories::PLANNING_TOOLS.contains(&"ExitPlanMode"));
        assert!(ToolCategories::PASSIVE_TOOLS.contains(&"Read"));
    }

    #[test]
    fn test_detect_session_limit_reached_empty() {
        let messages: Vec<Message> = vec![];
        assert!(!detect_session_limit_reached(&messages));
    }

    #[test]
    fn test_detect_api_error_empty() {
        let messages: Vec<Message> = vec![];
        assert!(!detect_api_error(&messages));
    }
}
