//! Message template engine
//!
//! This module provides template rendering with variable substitution
//! using simple {{variable}} syntax.

use crate::config::schema::MessageTemplate;
use crate::hooks::{HookData, HookInput, HookType};
use std::collections::HashMap;

/// Template engine for rendering messages
pub struct TemplateEngine {
    global_templates: HashMap<String, MessageTemplate>,
}

impl TemplateEngine {
    /// Create a new template engine with global templates
    pub fn new(global_templates: HashMap<String, MessageTemplate>) -> Self {
        Self { global_templates }
    }

    /// Render a template for a specific hook input
    pub fn render(&self, template: &MessageTemplate, input: &HookInput) -> RenderedMessage {
        let context = self.build_context(input);

        RenderedMessage {
            title: self.render_string(template.title.as_deref(), &context),
            body: self.render_string(template.body.as_deref(), &context),
        }
    }

    /// Get the appropriate template for a hook type
    pub fn get_template(
        &self,
        hook_type: &HookType,
        channel_template: Option<&MessageTemplate>,
    ) -> MessageTemplate {
        // Prefer channel-specific template
        if let Some(template) = channel_template {
            return template.clone();
        }

        // Fall back to hook type specific template
        let hook_type_str = format!("{:?}", hook_type);
        if let Some(template) = self.global_templates.get(&hook_type_str) {
            return template.clone();
        }

        // Fall back to default template
        self.global_templates
            .get("default")
            .cloned()
            .unwrap_or_default()
    }

    /// Build context variables from hook input
    fn build_context(&self, input: &HookInput) -> HashMap<String, String> {
        let mut ctx = HashMap::new();
        ctx.insert("hook_type".to_string(), format!("{:?}", input.hook_type));
        ctx.insert("session_id".to_string(), input.common.session_id.clone());

        if let Some(transcript_path) = &input.common.transcript_path {
            ctx.insert("transcript_path".to_string(), transcript_path.clone());
        }

        // Add hook-type specific variables
        match &input.data {
            HookData::Notification(data) => {
                ctx.insert("message".to_string(), data.message.clone());
                if let Some(title) = &data.title {
                    ctx.insert("title".to_string(), title.clone());
                }
            }
            HookData::PreToolUse(data) => {
                ctx.insert("tool_name".to_string(), data.tool_name.clone());
                // Add 'message' for compatibility with default templates
                let message = data
                    .context
                    .as_deref()
                    .unwrap_or(&data.tool_name)
                    .to_string();
                ctx.insert("message".to_string(), message);
                if let Some(context) = &data.context {
                    ctx.insert("context".to_string(), context.clone());
                }
            }
            HookData::Stop(data) => {
                // Add both 'message' and 'reason' for compatibility
                let message = data
                    .reason
                    .as_deref()
                    .unwrap_or("Claude stopped generating")
                    .to_string();
                ctx.insert("message".to_string(), message.clone());
                if let Some(reason) = &data.reason {
                    ctx.insert("reason".to_string(), reason.clone());
                }
            }
            HookData::SubagentStop(data) => {
                // Add both 'message' and 'reason' for compatibility
                let message = if let (Some(id), Some(reason)) = (&data.subagent_id, &data.reason) {
                    format!("Subagent {} stopped: {}", id, reason)
                } else if let Some(reason) = &data.reason {
                    format!("Subagent stopped: {}", reason)
                } else if let Some(id) = &data.subagent_id {
                    format!("Subagent {} stopped", id)
                } else {
                    "Subagent stopped".to_string()
                };
                ctx.insert("message".to_string(), message);
                if let Some(subagent_id) = &data.subagent_id {
                    ctx.insert("subagent_id".to_string(), subagent_id.clone());
                }
                if let Some(reason) = &data.reason {
                    ctx.insert("reason".to_string(), reason.clone());
                }
            }
            HookData::PermissionRequest(data) => {
                let message = if let Some(tool_name) = &data.tool_name {
                    format!("Claude requests permission to use {}", tool_name)
                } else {
                    "Claude requests permission to execute a tool".to_string()
                };
                ctx.insert("message".to_string(), message);
                if let Some(tool_name) = &data.tool_name {
                    ctx.insert("tool_name".to_string(), tool_name.clone());
                }
                if let Some(context) = &data.context {
                    ctx.insert("context".to_string(), context.clone());
                }
            }
        }

        ctx
    }

    /// Render a template string with variable substitution
    /// Supports {{variable}} syntax
    fn render_string(&self, template: Option<&str>, context: &HashMap<String, String>) -> String {
        let mut result = template.unwrap_or("").to_string();

        for (key, value) in context {
            result = result.replace(&format!("{{{{{}}}}}", key), value);
        }

        result
    }
}

/// Rendered message with title and body
#[derive(Debug, Clone)]
pub struct RenderedMessage {
    pub title: String,
    pub body: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_rendering() {
        let mut global_templates = HashMap::new();
        global_templates.insert(
            "default".to_string(),
            MessageTemplate {
                title: Some("{{hook_type}}".to_string()),
                body: Some("{{message}}".to_string()),
                ..Default::default()
            },
        );

        let engine = TemplateEngine::new(global_templates);

        let input = HookInput::notification(
            "test-session".to_string(),
            None,
            "Test message".to_string(),
            Some("Test Title".to_string()),
        );

        let template = MessageTemplate {
            title: Some("{{hook_type}} - {{title}}".to_string()),
            body: Some("{{message}}".to_string()),
            ..Default::default()
        };

        let rendered = engine.render(&template, &input);
        assert_eq!(rendered.title, "Notification - Test Title");
        assert_eq!(rendered.body, "Test message");
    }

    #[test]
    fn test_pre_tool_use_template() {
        let mut global_templates = HashMap::new();
        global_templates.insert(
            "PreToolUse".to_string(),
            MessageTemplate {
                title: Some("Tool: {{tool_name}}".to_string()),
                body: Some("{{context}}".to_string()),
                ..Default::default()
            },
        );

        let engine = TemplateEngine::new(global_templates);

        let input = HookInput::pre_tool_use(
            "test-session".to_string(),
            None,
            "ExitPlanMode".to_string(),
            Some("Exiting plan mode".to_string()),
        );

        let template = engine.get_template(&HookType::PreToolUse, None);
        let rendered = engine.render(&template, &input);

        assert_eq!(rendered.title, "Tool: ExitPlanMode");
        assert_eq!(rendered.body, "Exiting plan mode");
    }

    #[test]
    fn test_missing_variables() {
        let global_templates = HashMap::new();
        let engine = TemplateEngine::new(global_templates);

        let input =
            HookInput::notification("test-session".to_string(), None, "Test".to_string(), None);

        let template = MessageTemplate {
            title: Some("{{hook_type}} - {{missing_var}}".to_string()),
            body: Some("{{message}}".to_string()),
            ..Default::default()
        };

        let rendered = engine.render(&template, &input);
        assert_eq!(rendered.title, "Notification - {{missing_var}}");
        assert_eq!(rendered.body, "Test");
    }
}
