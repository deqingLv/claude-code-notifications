use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum HookType {
    Notification,
    PreToolUse,
    Stop,
    SubagentStop,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HookInputV1 {
    pub hook_type: HookType,
    pub session_id: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HookInputV2 {
    pub hook_event_name: HookType,
    pub session_id: String,
}

fn main() {
    // Test both formats
    let json_v1 = r#"{"hook_type":"Stop","session_id":"test"}"#;
    let json_v2 = r#"{"hook_event_name":"Stop","session_id":"test"}"#;
    
    println!("Testing hook_type field:");
    match serde_json::from_str::<HookInputV1>(json_v1) {
        Ok(v) => println!("  ✓ Success: {:?}", v),
        Err(e) => println!("  ✗ Error: {}", e),
    }
    
    println!("\nTesting hook_event_name field:");
    match serde_json::from_str::<HookInputV2>(json_v2) {
        Ok(v) => println!("  ✓ Success: {:?}", v),
        Err(e) => println!("  ✗ Error: {}", e),
    }
}
