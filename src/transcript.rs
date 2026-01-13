//! Transcript parsing and analysis for Claude Code sessions
//!
//! This module provides functionality for parsing Claude Code transcript files
//! (JSONL format) and extracting relevant information for notification generation.

use crate::error::{NotificationError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};

/// Type of message in the transcript
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    User,
    Assistant,
}

/// Content block within a message
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Text content from user or assistant
    Text { text: String },
    /// Tool invocation by assistant
    ToolUse {
        name: String,
        #[serde(default)]
        input: serde_json::Value,
        #[serde(default)]
        id: String,
    },
    /// Result from tool execution
    ToolResult {
        tool_use_id: String,
        content: String,
        #[serde(default)]
        is_error: bool,
    },
}

/// Message structure from Claude Code transcript
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    #[serde(rename = "type")]
    pub message_type: MessageType,
    pub message: MessageContent,
    pub timestamp: String,
    #[serde(default)]
    pub parent_uuid: Option<String>,
}

/// Message content wrapper
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageContent {
    pub content: Vec<ContentBlock>,
}

/// Tool use information extracted from messages
#[derive(Debug, Clone)]
pub struct ToolUse {
    pub name: String,
    pub timestamp: String,
    pub input: serde_json::Value,
}

/// Parse JSONL transcript file
///
/// Returns all successfully parsed messages, skipping invalid lines gracefully.
pub fn parse_file(path: &str) -> Result<Vec<Message>> {
    let file = std::fs::File::open(path).map_err(|e| {
        NotificationError::TranscriptError(format!("Failed to open transcript: {}", e))
    })?;

    let reader = BufReader::new(file);
    let mut messages = Vec::new();
    let mut skipped_lines = 0;

    for line in reader.lines() {
        let line = line.map_err(|e| {
            NotificationError::TranscriptError(format!("Failed to read line: {}", e))
        })?;

        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        // Parse JSON, skip invalid lines gracefully
        match serde_json::from_str::<Message>(&line) {
            Ok(msg) => messages.push(msg),
            Err(_) => {
                skipped_lines += 1;
                // Only log if too many lines are skipped
                if skipped_lines <= 10 {
                    eprintln!("Warning: Skipping invalid transcript line");
                }
            }
        }
    }

    if skipped_lines > 10 {
        eprintln!(
            "Warning: Skipped {} invalid lines in transcript",
            skipped_lines
        );
    }

    Ok(messages)
}

/// Get timestamp of last user message
///
/// Returns empty string if no user messages found.
pub fn get_last_user_timestamp(messages: &[Message]) -> String {
    messages
        .iter()
        .rev()
        .find(|msg| msg.message_type == MessageType::User)
        .map(|msg| msg.timestamp.clone())
        .unwrap_or_default()
}

/// Filter messages after given timestamp
///
/// Returns all messages with timestamps greater than the provided timestamp.
/// If timestamp is empty, returns all messages.
pub fn filter_messages_after_timestamp(messages: &[Message], timestamp: &str) -> Vec<Message> {
    if timestamp.is_empty() {
        return messages.to_vec();
    }

    let filter_time = match parse_timestamp(timestamp) {
        Ok(t) => t,
        Err(_) => return messages.to_vec(),
    };

    messages
        .iter()
        .filter(|msg| {
            if let Ok(msg_time) = parse_timestamp(&msg.timestamp) {
                msg_time > filter_time
            } else {
                false
            }
        })
        .cloned()
        .collect()
}

/// Extract all tools from assistant messages
pub fn extract_tools(messages: &[Message]) -> Vec<ToolUse> {
    messages
        .iter()
        .filter(|msg| msg.message_type == MessageType::Assistant)
        .flat_map(|msg| {
            msg.message.content.iter().filter_map(|content| {
                if let ContentBlock::ToolUse { name, input, .. } = content {
                    Some(ToolUse {
                        name: name.clone(),
                        timestamp: msg.timestamp.clone(),
                        input: input.clone(),
                    })
                } else {
                    None
                }
            })
        })
        .collect()
}

/// Get the name of the last tool used
pub fn get_last_tool(tools: &[ToolUse]) -> Option<&str> {
    tools.last().map(|tool| tool.name.as_str())
}

/// Find position of tool by name in the tools list
///
/// Returns -1 if tool not found.
pub fn find_tool_position(tools: &[ToolUse], tool_name: &str) -> isize {
    tools
        .iter()
        .position(|tool| tool.name == tool_name)
        .map(|i| i as isize)
        .unwrap_or(-1)
}

/// Count tools after given position
pub fn count_tools_after_position(tools: &[ToolUse], position: isize) -> usize {
    if position < 0 {
        return 0;
    }
    tools.iter().skip(position as usize + 1).count()
}

/// Count tools by names
pub fn count_tools_by_names(tools: &[ToolUse], names: &[&str]) -> usize {
    tools
        .iter()
        .filter(|tool| names.contains(&tool.name.as_str()))
        .count()
}

/// Check if any active tool is present
pub fn has_any_active_tool(tools: &[ToolUse], active_tools: &[&str]) -> bool {
    tools
        .iter()
        .any(|tool| active_tools.contains(&tool.name.as_str()))
}

/// Extract text content from messages
pub fn extract_text_from_messages(messages: &[Message]) -> Vec<String> {
    messages
        .iter()
        .filter_map(|msg| {
            msg.message
                .content
                .iter()
                .filter_map(|content| {
                    if let ContentBlock::Text { text } = content {
                        Some(text.clone())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
                .into()
        })
        .collect()
}

/// Extract recent text from last N messages
pub fn extract_recent_text(messages: &[Message], limit: usize) -> String {
    let recent_messages = if messages.len() > limit {
        &messages[messages.len() - limit..]
    } else {
        messages
    };

    extract_text_from_messages(recent_messages).join(" ")
}

/// Get last N assistant messages
pub fn get_last_assistant_messages(messages: &[Message], limit: usize) -> Vec<Message> {
    messages
        .iter()
        .rev()
        .filter(|msg| msg.message_type == MessageType::Assistant)
        .take(limit)
        .cloned()
        .collect()
}

/// Get timestamp of last assistant message
pub fn get_last_assistant_timestamp(messages: &[Message]) -> String {
    messages
        .iter()
        .rev()
        .find(|msg| msg.message_type == MessageType::Assistant)
        .map(|msg| msg.timestamp.clone())
        .unwrap_or_default()
}

/// Parse RFC3339 timestamp
fn parse_timestamp(ts: &str) -> Result<DateTime<Utc>> {
    ts.parse::<DateTime<Utc>>().map_err(|e| {
        NotificationError::TranscriptError(format!("Failed to parse timestamp '{}': {}", ts, e))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_transcript() {
        let messages = parse_file("nonexistent.jsonl");
        assert!(messages.is_err());
    }

    #[test]
    fn test_filter_empty_timestamp() {
        let messages = vec![Message {
            message_type: MessageType::User,
            message: MessageContent { content: vec![] },
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            parent_uuid: None,
        }];
        let filtered = filter_messages_after_timestamp(&messages, "");
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_extract_tools_from_empty_messages() {
        let tools = extract_tools(&[]);
        assert!(tools.is_empty());
    }

    #[test]
    fn test_get_last_tool_empty() {
        let tools: Vec<ToolUse> = vec![];
        assert!(get_last_tool(&tools).is_none());
    }
}
