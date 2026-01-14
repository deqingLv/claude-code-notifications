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
        crate::debug_context!(
            "TemplateEngine",
            "Getting template for hook_type: {:?}",
            hook_type
        );

        // Prefer channel-specific template
        if let Some(template) = channel_template {
            crate::debug_context!("TemplateEngine", "Using channel-specific template");
            return template.clone();
        }

        // Fall back to hook type specific template
        let hook_type_str = format!("{:?}", hook_type);
        crate::debug_context!(
            "TemplateEngine",
            "Looking for hook_type template: {}",
            hook_type_str
        );
        if let Some(template) = self.global_templates.get(&hook_type_str) {
            crate::debug_context!("TemplateEngine", "Found hook_type template: {:?}", template);
            return template.clone();
        }

        // Fall back to default template
        crate::debug_context!("TemplateEngine", "Using default template");
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

        if let Some(cwd) = &input.common.cwd {
            ctx.insert("cwd".to_string(), cwd.clone());
        }

        if let Some(permission_mode) = &input.common.permission_mode {
            ctx.insert("permission_mode".to_string(), permission_mode.clone());
        }

        crate::debug_context!(
            "TemplateEngine",
            "Building context for hook_type: {:?}",
            input.hook_type
        );
        crate::debug_context!("TemplateEngine", "Input data: {:?}", input.data);

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
                ctx.insert("message".to_string(), data.tool_name.clone());
                if let Some(tool_input) = &data.tool_input {
                    ctx.insert("tool_input".to_string(), tool_input.to_string());
                }
                if let Some(tool_use_id) = &data.tool_use_id {
                    ctx.insert("tool_use_id".to_string(), tool_use_id.clone());
                }
            }
            HookData::Stop(data) => {
                let message = if data.stop_hook_active.unwrap_or(false) {
                    "Claude continuing (stop hook active)"
                } else {
                    "Claude stopped generating"
                };
                ctx.insert("message".to_string(), message.to_string());
                if let Some(stop_hook_active) = data.stop_hook_active {
                    ctx.insert("stop_hook_active".to_string(), stop_hook_active.to_string());
                }
            }
            HookData::SubagentStop(data) => {
                let message = if data.stop_hook_active.unwrap_or(false) {
                    "Subagent continuing (stop hook active)"
                } else {
                    "Subagent stopped"
                };
                ctx.insert("message".to_string(), message.to_string());
                if let Some(stop_hook_active) = data.stop_hook_active {
                    ctx.insert("stop_hook_active".to_string(), stop_hook_active.to_string());
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
        assert_eq!(rendered.body, "{{context}}");
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
