//! Transcript analysis engine for determining task status
//!
//! This module implements a state machine that analyzes Claude Code session
//! transcripts to determine task completion status and generate appropriate
//! notifications.

use crate::error::{NotificationError, Result};
use crate::transcript::*;

use crate::debug_context;

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
    debug_context!(
        "analyzer",
        "analyze_transcript() called with: {}",
        transcript_path
    );
    let start = std::time::Instant::now();

    // Parse transcript
    debug_context!("analyzer", "Parsing transcript...");
    let messages = parse_file(transcript_path).map_err(|e| {
        debug_context!("analyzer", "Failed to parse transcript: {}", e);
        NotificationError::AnalysisError(format!("Failed to parse transcript: {}", e))
    })?;

    debug_context!("analyzer", "Parsed {} messages", messages.len());

    if messages.is_empty() {
        debug_context!("analyzer", "No messages found, returning Unknown");
        return Ok(Status::Unknown);
    }

    // PRIORITY CHECK 1: Session limit reached
    if detect_session_limit_reached(&messages) {
        debug_context!("analyzer", "Detected: SessionLimitReached");
        return Ok(Status::SessionLimitReached);
    }

    // PRIORITY CHECK 2: API authentication error
    if detect_api_error(&messages) {
        debug_context!("analyzer", "Detected: APIError");
        return Ok(Status::APIError);
    }

    // Find last user message timestamp for temporal filtering
    let user_ts = get_last_user_timestamp(&messages);
    debug_context!("analyzer", "Last user timestamp: {}", user_ts);

    // Filter messages after last user message (current response only)
    let filtered_messages = filter_messages_after_timestamp(&messages, &user_ts);
    debug_context!(
        "analyzer",
        "Filtered to {} messages after last user message",
        filtered_messages.len()
    );

    if filtered_messages.is_empty() {
        debug_context!(
            "analyzer",
            "No messages after last user message, returning Unknown"
        );
        return Ok(Status::Unknown);
    }

    // Take last 15 messages (temporal window for analysis)
    let recent_messages = if filtered_messages.len() > 15 {
        &filtered_messages[filtered_messages.len() - 15..]
    } else {
        &filtered_messages
    };
    debug_context!(
        "analyzer",
        "Analyzing last {} messages",
        recent_messages.len()
    );

    // Extract tools from filtered messages
    let tools = extract_tools(recent_messages);
    debug_context!("analyzer", "Extracted {} tools", tools.len());
    if !tools.is_empty() {
        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        debug_context!("analyzer", "Tools: {:?}", tool_names);
    }

    // STATE MACHINE LOGIC
    if !tools.is_empty() {
        let last_tool = get_last_tool(&tools);
        debug_context!("analyzer", "Last tool: {:?}", last_tool);

        // Check: Last tool is ExitPlanMode
        if last_tool == Some("ExitPlanMode") {
            debug_context!(
                "analyzer",
                "Detected: PlanReady (last tool is ExitPlanMode)"
            );
            return Ok(Status::PlanReady);
        }

        // Check: Last tool is AskUserQuestion
        if last_tool == Some("AskUserQuestion") {
            debug_context!(
                "analyzer",
                "Detected: Question (last tool is AskUserQuestion)"
            );
            return Ok(Status::Question);
        }

        // Check: ExitPlanMode exists AND tools after it
        let exit_plan_pos = find_tool_position(&tools, "ExitPlanMode");
        if exit_plan_pos >= 0 {
            let tools_after = count_tools_after_position(&tools, exit_plan_pos);
            if tools_after > 0 {
                debug_context!(
                    "analyzer",
                    "Detected: TaskComplete (ExitPlanMode with {} tools after)",
                    tools_after
                );
                return Ok(Status::TaskComplete);
            }
        }

        // Check: Review detection (only read-like tools + long text)
        let read_like_count = count_tools_by_names(&tools, &["Read", "Grep", "Glob"]);
        let has_active_tools = has_any_active_tool(&tools, ToolCategories::ACTIVE_TOOLS);
        debug_context!(
            "analyzer",
            "Read-like tools: {}, Has active tools: {}",
            read_like_count,
            has_active_tools
        );

        if read_like_count >= 1 && !has_active_tools {
            let recent_text = extract_recent_text(recent_messages, 5);
            debug_context!("analyzer", "Recent text length: {}", recent_text.len());
            if recent_text.len() > 200 {
                debug_context!(
                    "analyzer",
                    "Detected: ReviewComplete (read-like tools + long text)"
                );
                return Ok(Status::ReviewComplete);
            }
        }

        // Check: Last tool is active (Write/Edit/Bash/etc)
        if last_tool.is_some_and(|name| ToolCategories::ACTIVE_TOOLS.contains(&name)) {
            debug_context!("analyzer", "Detected: TaskComplete (last tool is active)");
            return Ok(Status::TaskComplete);
        }

        // Check: Any tool usage at all
        debug_context!("analyzer", "Detected: TaskComplete (any tool usage)");
        return Ok(Status::TaskComplete);
    }

    // No tools found
    debug_context!("analyzer", "No tools found, returning Unknown");
    debug_context!("analyzer", "Analysis completed in {:?}", start.elapsed());
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
