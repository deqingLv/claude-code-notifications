//! Summary generation from transcript analysis
//!
//! This module provides intelligent message generation based on transcript
//! analysis status, including markdown cleanup, sentence extraction, and
//! status-specific generators.

use crate::analyzer::Status;
use crate::transcript::*;
use regex::Regex;

/// Generate summary from transcript based on analysis status
///
/// This is the main entry point for summary generation. It analyzes the
/// transcript and generates an appropriate message based on the status.
pub fn generate_summary(transcript_path: &str, status: Status) -> String {
    // Try to parse transcript
    let messages = match parse_file(transcript_path) {
        Ok(msgs) if !msgs.is_empty() => msgs,
        _ => return get_default_message(status),
    };

    // Use status-specific generator
    match status {
        Status::Question => generate_question_summary(&messages),
        Status::PlanReady => generate_plan_summary(&messages),
        Status::ReviewComplete => generate_review_summary(&messages),
        Status::TaskComplete => generate_task_summary(&messages),
        Status::SessionLimitReached => {
            "Session limit reached. Please start a new conversation.".to_string()
        }
        Status::APIError => "Please run /login".to_string(),
        Status::Unknown => generate_task_summary(&messages),
    }
}

/// Generate question summary
///
/// Priority order:
/// 1. Extract question text from AskUserQuestion tool (must be within 60s)
/// 2. Find shortest text containing "?" in last 8 messages
/// 3. Extract first sentence from last message
/// 4. Fallback: "Claude needs your input to continue"
fn generate_question_summary(messages: &[Message]) -> String {
    // 1) Try to extract AskUserQuestion tool
    let (question, is_recent) = extract_ask_user_question(messages);
    if !question.is_empty() && is_recent {
        let cleaned = clean_markdown(&question);
        return truncate_text(&cleaned, 150);
    }

    // 2) Get recent messages from current response
    let recent_messages = get_recent_assistant_messages(messages, 8);
    let texts = extract_text_from_messages(&recent_messages);

    // Strategy A: Find texts with "?" and prioritize short ones
    let mut question_texts: Vec<&String> = texts.iter().filter(|text| text.contains('?')).collect();

    if !question_texts.is_empty() {
        question_texts.sort_by_key(|q| q.len());
        let shortest = &question_texts[0];
        if shortest.len() > 10 {
            let cleaned = clean_markdown(shortest);
            return truncate_text(&cleaned, 150);
        }
    }

    // Strategy B: First sentence from last message
    if let Some(last_text) = texts.last() {
        let cleaned = clean_markdown(last_text);
        let first_sentence = extract_first_sentence(&cleaned);
        if first_sentence.len() > 10 {
            return truncate_text(&first_sentence, 150);
        }
    }

    // Final fallback
    "Claude needs your input to continue".to_string()
}

/// Generate plan summary
fn generate_plan_summary(messages: &[Message]) -> String {
    let plan = extract_exit_plan_mode_plan(messages);
    if !plan.is_empty() {
        let lines: Vec<&str> = plan.lines().collect();
        for line in lines {
            let cleaned = clean_markdown(line);
            if !cleaned.trim().is_empty() {
                return truncate_text(&cleaned, 150);
            }
        }
    }
    "Plan is ready for review".to_string()
}

/// Generate review summary
fn generate_review_summary(messages: &[Message]) -> String {
    let recent_messages = get_last_assistant_messages(messages, 5);
    let texts = extract_text_from_messages(&recent_messages);
    let combined = texts.join(" ");

    // Check for review keywords
    let keywords = ["review", "analyzed", "analysis"];
    for keyword in &keywords {
        if combined.to_lowercase().contains(keyword) {
            for text in &texts {
                if text.to_lowercase().contains(keyword) {
                    let cleaned = clean_markdown(text);
                    return truncate_text(&cleaned, 150);
                }
            }
        }
    }

    // Count Read tools
    let tools = extract_tools(&recent_messages);
    let read_count = tools.iter().filter(|t| t.name == "Read").count();
    if read_count > 0 {
        let noun = if read_count == 1 { "file" } else { "files" };
        return format!("Reviewed {} {}", read_count, noun);
    }

    "Code review completed".to_string()
}

/// Generate task summary
fn generate_task_summary(messages: &[Message]) -> String {
    let recent_messages = get_last_assistant_messages(messages, 5);
    let texts = extract_text_from_messages(&recent_messages);

    let last_message = texts.last().cloned().unwrap_or_default();

    // Calculate duration and tool counts
    let duration = calculate_duration(messages);
    let tool_counts = count_tools_by_type(messages);
    let actions = build_actions_string(&tool_counts, &duration);

    if !last_message.is_empty() {
        let cleaned = clean_markdown(&last_message);
        let message_text = if cleaned.len() < 150 {
            cleaned
        } else {
            extract_first_sentence(&cleaned)
        };

        if !actions.is_empty() {
            let combined = format!("{}. {}", message_text, actions);
            return truncate_text(&combined, 150);
        }
        return truncate_text(&message_text, 150);
    }

    if !actions.is_empty() {
        return actions;
    }

    let tool_count: usize = tool_counts.values().sum();
    if tool_count > 0 {
        return format!("Completed task with {} operations", tool_count);
    }

    "Task completed successfully".to_string()
}

/// Get default message for status
fn get_default_message(status: Status) -> String {
    match status {
        Status::TaskComplete => "Task completed successfully".to_string(),
        Status::ReviewComplete => "Code review completed".to_string(),
        Status::Question => "Claude needs your input".to_string(),
        Status::PlanReady => "Plan is ready".to_string(),
        Status::SessionLimitReached => "Session limit reached".to_string(),
        Status::APIError => "Please run /login".to_string(),
        Status::Unknown => "Claude Code notification".to_string(),
    }
}

/// Clean markdown from text
///
/// Removes markdown formatting including headers, bullets, code blocks,
/// links, images, bold, and italic.
pub fn clean_markdown(text: &str) -> String {
    let patterns = create_markdown_patterns();

    // Remove code blocks
    let text = patterns.code_blocks.replace_all(text, "");

    // Convert images to alt text
    let text = patterns.images.replace_all(&text, "$1");

    // Convert links to text
    let text = patterns.links.replace_all(&text, "$1");

    // Remove bold, italic
    let text = patterns.bold.replace_all(&text, "$2");
    let text = patterns.italic.replace_all(&text, "$2");

    // Process line by line
    let lines: Vec<&str> = text.lines().collect();
    let cleaned: Vec<String> = lines
        .iter()
        .map(|line| {
            let line = line.trim();
            let line = patterns.headers.replace(line, "").to_string();
            let line = patterns.bullets.replace(&line, "").to_string();
            line.trim().to_string()
        })
        .filter(|line| !line.is_empty())
        .collect();

    cleaned.join(" ")
}

/// Extract first sentence from text
///
/// Extracts a complete sentence, handling abbreviations and edge cases.
pub fn extract_first_sentence(text: &str) -> String {
    const MIN_LENGTH: usize = 20;
    const MAX_LENGTH: usize = 200;

    let chars: Vec<char> = text.chars().collect();
    let mut sentences = Vec::new();
    let mut current_start = 0;

    for (i, &char) in chars.iter().enumerate() {
        if char == '.' || char == '!' || char == '?' {
            // Check if this is a real sentence end (not abbreviation, version number, etc.)
            if char == '.' {
                // Skip version numbers, decimals, abbreviations
                if i > 0 && chars[i - 1].is_ascii_digit() {
                    continue;
                }
                if i + 1 < chars.len() && chars[i + 1].is_ascii_digit() {
                    continue;
                }
                if i + 1 < chars.len() && !chars[i + 1].is_whitespace() {
                    continue;
                }
            }

            let sentence: String = chars[current_start..=i].iter().collect();
            let sentence = sentence.trim().to_string();
            if !sentence.is_empty() {
                sentences.push(sentence);
                current_start = i + 1;

                let total_len: usize = sentences.join(" ").len();
                if sentences.len() == 1 && total_len < MIN_LENGTH && total_len < MAX_LENGTH {
                    continue;
                }
                if total_len >= MAX_LENGTH {
                    return sentences[..sentences.len() - 1].join(" ");
                }
                return sentences.join(" ");
            }
        }
    }

    if !sentences.is_empty() {
        return sentences.join(" ");
    }

    if chars.len() > 100 {
        chars[..100].iter().collect()
    } else {
        text.to_string()
    }
}

/// Truncate text to max length
///
/// Smart truncation at sentence or word boundaries.
pub fn truncate_text(text: &str, max_len: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= max_len {
        return text.to_string();
    }

    // Try to find sentence boundary
    let search_text: String = chars[..max_len].iter().collect();
    let sentence_enders = [". ", "! ", "? ", ".\n", "!\n", "?\n"];

    for ender in &sentence_enders {
        if let Some(pos) = search_text.rfind(ender) {
            if pos > max_len / 3 {
                return search_text[..pos + ender.len() - 1].trim().to_string();
            }
        }
    }

    // Try word boundary
    let truncated: String = chars[..max_len - 3].iter().collect();
    if let Some(last_space) = truncated.rfind(' ') {
        if last_space > max_len / 2 {
            return format!("{}...", &truncated[..last_space]);
        }
    }

    format!("{}...", truncated)
}

// Helper functions

fn get_recent_assistant_messages(messages: &[Message], limit: usize) -> Vec<Message> {
    let user_ts = get_last_user_timestamp(messages);
    let filtered = filter_messages_after_timestamp(messages, &user_ts);

    if !filtered.is_empty() {
        let recent = if filtered.len() > limit {
            &filtered[filtered.len() - limit..]
        } else {
            &filtered
        };
        return recent.to_vec();
    }

    get_last_assistant_messages(messages, limit)
}

fn extract_ask_user_question(messages: &[Message]) -> (String, bool) {
    let mut question_text = String::new();
    let mut question_timestamp = String::new();

    for msg in messages.iter().rev() {
        if msg.message_type != MessageType::Assistant {
            continue;
        }

        for content in &msg.message.content {
            if let ContentBlock::ToolUse { name, input, .. } = content {
                if name == "AskUserQuestion" {
                    if let Some(questions) = input.get("questions").and_then(|v| v.as_array()) {
                        if let Some(q) = questions.first().and_then(|v| v.as_object()) {
                            if let Some(qtext) = q.get("question").and_then(|v| v.as_str()) {
                                question_text = qtext.to_string();
                                question_timestamp = msg.timestamp.clone();
                                break;
                            }
                        }
                    }
                }
            }
        }

        if !question_text.is_empty() {
            break;
        }
    }

    if question_text.is_empty() {
        return (String::new(), false);
    }

    // Check recency (60s window)
    let last_assistant_ts = get_last_assistant_timestamp(messages);
    if question_timestamp.is_empty() || last_assistant_ts.is_empty() {
        return (question_text, false);
    }

    match (
        question_timestamp.parse::<chrono::DateTime<chrono::Utc>>(),
        last_assistant_ts.parse::<chrono::DateTime<chrono::Utc>>(),
    ) {
        (Ok(q_time), Ok(last_time)) => {
            let age = last_time.signed_duration_since(q_time);
            let is_recent = age.num_seconds() >= 0 && age.num_seconds() <= 60;
            (question_text, is_recent)
        }
        _ => (question_text, false),
    }
}

fn extract_exit_plan_mode_plan(messages: &[Message]) -> String {
    for msg in messages.iter().rev() {
        for content in &msg.message.content {
            if let ContentBlock::ToolUse { name, input, .. } = content {
                if name == "ExitPlanMode" {
                    if let Some(plan) = input.get("plan").and_then(|v| v.as_str()) {
                        return plan.to_string();
                    }
                }
            }
        }
    }
    String::new()
}

fn calculate_duration(messages: &[Message]) -> String {
    let user_ts = get_last_user_timestamp(messages);
    let assistant_ts = get_last_assistant_timestamp(messages);

    if user_ts.is_empty() || assistant_ts.is_empty() {
        return String::new();
    }

    match (
        user_ts.parse::<chrono::DateTime<chrono::Utc>>(),
        assistant_ts.parse::<chrono::DateTime<chrono::Utc>>(),
    ) {
        (Ok(user_time), Ok(assistant_time)) => {
            let duration = assistant_time.signed_duration_since(user_time);
            if duration.num_seconds() < 0 {
                return String::new();
            }
            format_duration(duration)
        }
        _ => String::new(),
    }
}

fn format_duration(d: chrono::Duration) -> String {
    let seconds = d.num_seconds();
    if seconds < 60 {
        return format!("Took {}s", seconds);
    }

    let minutes = seconds / 60;
    let secs = seconds % 60;

    if minutes < 60 {
        if secs > 0 {
            return format!("Took {}m {}s", minutes, secs);
        }
        return format!("Took {}m", minutes);
    }

    let hours = minutes / 60;
    let mins = minutes % 60;

    if mins > 0 {
        return format!("Took {}h {}m", hours, mins);
    }
    format!("Took {}h", hours)
}

fn count_tools_by_type(messages: &[Message]) -> std::collections::HashMap<String, usize> {
    let mut counts = std::collections::HashMap::new();

    let user_ts = get_last_user_timestamp(messages);
    let since_time = user_ts.parse::<chrono::DateTime<chrono::Utc>>().ok();

    for msg in messages {
        if msg.message_type != MessageType::Assistant {
            continue;
        }

        // Check if message is after user message
        if let Some(since) = since_time {
            if let Ok(msg_time) = msg.timestamp.parse::<chrono::DateTime<chrono::Utc>>() {
                if msg_time < since {
                    continue;
                }
            }
        }

        for content in &msg.message.content {
            if let ContentBlock::ToolUse { name, .. } = content {
                *counts.entry(name.clone()).or_insert(0) += 1;
            }
        }
    }

    counts
}

fn build_actions_string(
    tool_counts: &std::collections::HashMap<String, usize>,
    duration: &str,
) -> String {
    let mut parts = Vec::new();

    if let Some(&count) = tool_counts.get("Write") {
        let noun = if count == 1 { "file" } else { "files" };
        parts.push(format!("Created {} {}", count, noun));
    }

    if let Some(&count) = tool_counts.get("Edit") {
        let noun = if count == 1 { "file" } else { "files" };
        parts.push(format!("Edited {} {}", count, noun));
    }

    if let Some(&count) = tool_counts.get("Bash") {
        let noun = if count == 1 { "command" } else { "commands" };
        parts.push(format!("Ran {} {}", count, noun));
    }

    if !duration.is_empty() {
        parts.push(duration.to_string());
    }

    if parts.is_empty() {
        String::new()
    } else {
        parts.join(". ")
    }
}

struct MarkdownPatterns {
    headers: Regex,
    bullets: Regex,
    code_blocks: Regex,
    links: Regex,
    images: Regex,
    bold: Regex,
    italic: Regex,
}

fn create_markdown_patterns() -> MarkdownPatterns {
    MarkdownPatterns {
        headers: Regex::new(r"^#+\s*").unwrap(),
        bullets: Regex::new(r"^[-*•]\s*").unwrap(),
        code_blocks: Regex::new(r"```[\s\S]*?```").unwrap(),
        links: Regex::new(r"\[([^\]]+)\]\([^\)]+\)").unwrap(),
        images: Regex::new(r"!\[([^\]]*)\]\([^\)]+\)").unwrap(),
        // Match both **bold** and __bold__
        bold: Regex::new(r"(\*\*.*?\*\*|__.*?__)").unwrap(),
        // Simplified italic matching (may not be perfect but works for common cases)
        italic: Regex::new(r"\*[^*]+\*|_[^_]+_").unwrap(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_markdown() {
        let text = "# Header\n\n**Bold** and *italic* text.";
        let cleaned = clean_markdown(text);
        assert!(!cleaned.contains("**") && !cleaned.contains("*"));
    }

    #[test]
    fn test_extract_first_sentence() {
        let text = "This is sentence one. This is sentence two.";
        let sentence = extract_first_sentence(text);
        assert_eq!(sentence.trim(), "This is sentence one.");
    }

    #[test]
    fn test_truncate_text() {
        let text = "This is a short text.";
        let truncated = truncate_text(text, 100);
        assert_eq!(truncated, text);
    }

    #[test]
    fn test_get_default_message() {
        assert_eq!(
            get_default_message(Status::TaskComplete),
            "Task completed successfully"
        );
        assert_eq!(
            get_default_message(Status::Question),
            "Claude needs your input"
        );
    }
}
