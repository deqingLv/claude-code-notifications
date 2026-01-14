# Debug Logging Support

## Overview

Debug logging has been added to help diagnose notification issues including:
- Notification icon showing Finder icon instead of custom icon
- Empty notification content
- Notification delivery delays

## Enabling Debug Logging

To enable debug logging, add `"debug": true` to your `~/.claude-code-notifications.json` configuration file:

```json
{
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
  "debug": true
}
```

## Debug Output Locations

Debug logs are printed to stderr (`eprintln!`), which means:
- When running from Claude Code hooks, logs will appear in Claude Code's stderr stream
- When running manually from CLI, logs will appear in your terminal

## What Gets Logged

### 1. Configuration Loading
- Configuration file path
- Debug mode status
- Default channels
- Channel configurations

### 2. Hook Processing (`handle_hook`)
- Hook type received (Notification, Stop, SubagentStop, etc.)
- Session ID
- Transcript path (if available)
- Notification title and body being prepared

### 3. Channel Routing (`ChannelManager`)
- Matched channels for the current hook
- Per-channel success/failure status
- Total time for notification delivery

### 4. System Channel (`system`)
- Notification preparation (title, body, timeout, icon)
- Icon resolution attempts and results
- Template rendering (title and body)
- Sound playback status
- Timing information for each operation

### 5. Transcript Analysis (`analyzer`)
- Transcript parsing progress
- Number of messages parsed
- Timestamp filtering results
- Tools extracted from transcript
- Status detection logic and reasoning
- Analysis completion time

## Example Debug Output

```bash
[DEBUG] Configuration loaded, debug mode: true
[DEBUG] [handle_hook] Received hook: Stop
[DEBUG] [handle_hook] Session ID: abc123
[DEBUG] [handle_hook] Transcript path: Some("/path/to/transcript.md")
[DEBUG] [analyzer] analyze_transcript() called with: /path/to/transcript.md
[DEBUG] [analyzer] Parsing transcript...
[DEBUG] [analyzer] Parsed 42 messages
[DEBUG] [analyzer] Last user timestamp: 2025-01-14T10:30:15.123Z
[DEBUG] [analyzer] Filtered to 8 messages after last user message
[DEBUG] [analyzer] Analyzing last 8 messages
[DEBUG] [analyzer] Extracted 3 tools
[DEBUG] [analyzer] Tools: ["Read", "Edit", "Bash"]
[DEBUG] [analyzer] Last tool: Some("Bash")
[DEBUG] [analyzer] Detected: TaskComplete (last tool is active)
[DEBUG] [handle_hook] Title: Claude Code
[DEBUG] [handle_hook] Body: Created 1 files. Edited 2 files. Ran 1 command. Took 2m 15s
[DEBUG] [ChannelManager] send_notification_async() called
[DEBUG] [ChannelManager] Matched channels: ["system"]
[DEBUG] [system] send() called
[DEBUG] [system] Timeout: 5000ms
[DEBUG] [system] Rendering template...
[DEBUG] [system] Rendered title: Claude Code
[DEBUG] [system] Rendered body: Created 1 files. Edited 2 files. Ran 1 command. Took 2m 15s
[DEBUG] [system] Using icon: Some("Claude Code")
[DEBUG] [system] Calling display_notification()...
[DEBUG] [system] Preparing notification
[DEBUG] [system] Title: Claude Code
[DEBUG] [system] Body: Created 1 files. Edited 2 files. Ran 1 command. Took 2m 15s
[DEBUG] [system] Timeout: 5000ms
[DEBUG] [system] Icon: Some("Claude Code")
[DEBUG] [system] Resolving icon: Claude Code
[DEBUG] [system] Icon 'Claude Code' not found, using system default
[DEBUG] [system] Showing notification...
[DEBUG] [system] Notification displayed successfully
[DEBUG] [system] display_notification() completed in 23.45ms
[DEBUG] [system] Playing sound: Glass
[DEBUG] [system] send() completed in 45.67ms
[DEBUG] [ChannelManager] Channel system succeeded
[DEBUG] [ChannelManager] send_notification_async() completed in 48.12ms
```

## Diagnosing Issues

### Issue 1: Finder Icon Instead of Custom Icon

**What to look for in debug output:**
```
[DEBUG] [system] Icon: Some("Claude Code")
[DEBUG] [system] Resolving icon: Claude Code
[DEBUG] [system] Icon 'Claude Code' not found, using system default
```

**Solution:** The icon file is not found in any of the expected locations. You need to:
1. Place an icon file named `claude-code.png`, `claude-code.icns`, or `claude.png` in one of:
   - `./assets/` relative to the executable
   - `~/.claude/`
   - Current working directory `./assets/`
2. Or specify a full path in the config:
   ```json
   {
     "channels": {
       "system": {
         "icon": "/absolute/path/to/icon.png"
       }
     }
   }
   ```

### Issue 2: Empty Notification Content

**What to look for in debug output:**
```
[DEBUG] [system] Rendered title: ...
[DEBUG] [system] Rendered body: ...
```

**Check if:**
- The body is empty or very short
- The template variables are not being replaced

**Solution:** Verify your message templates in the config file are correct.

### Issue 3: Notification Delay

**What to look for in debug output:**
```
[DEBUG] [ChannelManager] send_notification_async() completed in 48.12ms
[DEBUG] [system] send() completed in 45.67ms
[DEBUG] [system] display_notification() completed in 23.45ms
```

**If times are high (> 100ms):**
- Check if webhook channels are causing delays (they run in parallel but may timeout)
- The 2-second timeout for channel delivery may be causing delays

**Solution:** Consider disabling webhook channels or adjusting timeout settings.

## Files Modified

1. **src/lib.rs**
   - Added `mod logging;` declaration
   - Initialize debug flag in `ChannelManager::load()`
   - Added debug logging to `handle_hook()`
   - Added debug logging to `send_notification_async()`

2. **src/logging.rs** (new file)
   - Global `DEBUG_ENABLED` flag
   - `init_debug()` function to enable debug from config
   - `debug_log!()` and `debug_context!()` macros

3. **src/config/schema.rs**
   - Added `pub debug: bool` field to `AppConfig`

4. **src/config/loader.rs**
   - Updated `default_config()` to include `"debug": false`

5. **src/channels/system.rs**
   - Added debug logging to `send()` method
   - Added debug logging to `display_notification()` method
   - Added timing information

6. **src/analyzer.rs**
   - Added debug logging to `analyze_transcript()` function
   - Logs each step of the analysis process

7. **src/router.rs**
   - Added `debug: false` to test config

## Testing Debug Logging

To test debug logging:

```bash
# Enable debug in config
echo '"debug": true' >> ~/.claude-code-notifications.json

# Trigger a test notification
echo '{"session_id":"test","message":"Debug test"}' | cargo run

# Or with release build
echo '{"session_id":"test","message":"Debug test"}' | ./target/release/claude-code-notifications
```

## Disabling Debug Logging

Simply remove the `"debug": true` line or set it to `false` in your config file:

```json
{
  "debug": false
}
```

Debug logging defaults to `false` when not specified.
