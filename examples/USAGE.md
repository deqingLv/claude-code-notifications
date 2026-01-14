# Usage Examples

Practical examples of using `claude-code-notifications` in different scenarios.

## Basic Usage

### Command Line Interface
```bash
# Basic usage (reads JSON from stdin)
echo '{"session_id":"abc123","transcript_path":"/tmp/transcript.md","message":"Task completed"}' | claude-code-notifications

# With system sound
echo '{"session_id":"abc123","transcript_path":"/tmp/transcript.md","message":"Task completed with sound"}' | claude-code-notifications --sound Glass

# With custom sound file
echo '{"session_id":"abc123","transcript_path":"/tmp/transcript.md","message":"Custom sound alert"}' | claude-code-notifications --sound /path/to/alert.wav
```

### File Input
```bash
# Read JSON from file
claude-code-notifications < notification.json

# With sound parameter
claude-code-notifications --sound Submarine < notification.json
```

## Claude Code Hook Configurations

### Minimal Configuration
```json
{
  "hooks": {
    "Notification": [
      {
        "type": "command",
        "command": "claude-code-notifications"
      }
    ]
  }
}
```

### With Different System Sounds
```json
{
  "hooks": {
    "Notification": [
      {
        "type": "command",
        "command": "claude-code-notifications --sound Submarine"
      }
    ]
  }
}
```

### Multiple Notification Hooks
```json
{
  "hooks": {
    "Notification": [
      {
        "type": "command",
        "command": "claude-code-notifications --sound Glass"
      },
      {
        "type": "command",
        "command": "claude-code-notifications --sound Submarine"
      }
    ]
  }
}
```

## Integration Examples

### Shell Script Integration
```bash
#!/bin/bash
# notify.sh - Send notification from shell script

send_notification() {
    local message="$1"

    cat << EOF | claude-code-notifications --sound Glass
{
  "session_id": "shell-script-$(date +%s)",
  "transcript_path": "/tmp/script.log",
  "message": "$message"
}
EOF
}

# Usage
send_notification "Backup completed successfully"
send_notification "Build failed with errors"
```

### Python Integration
```python
#!/usr/bin/env python3
# notify.py - Send notification from Python

import json
import subprocess

def send_notification(message, sound="Glass"):
    """Send desktop notification from Python."""

    notification_data = {
        "session_id": f"python-{hash(message)}",
        "transcript_path": "/tmp/python-notification.log",
        "message": message
    }

    json_input = json.dumps(notification_data)

    cmd = ["claude-code-notifications"]
    if sound:
        cmd.extend(["--sound", sound])

    result = subprocess.run(
        cmd,
        input=json_input.encode(),
        capture_output=True,
        text=True
    )

    if result.returncode != 0:
        print(f"Error: {result.stderr}")

    return result.returncode == 0

# Usage
if __name__ == "__main__":
    send_notification("Python script completed", "Submarine")
    send_notification("Error detected in processing", "Basso")
```

### Automated Workflow
```bash
#!/bin/bash
# workflow.sh - Example automated workflow with notifications

echo "Starting automated workflow..."

# Step 1: Process data
process_data() {
    echo "Processing data..."
    sleep 2
    echo '{"session_id":"workflow-1","transcript_path":"/tmp/workflow.log","message":"Data processing completed"}' | claude-code-notifications --sound Ping
}

# Step 2: Generate reports
generate_reports() {
    echo "Generating reports..."
    sleep 3
    echo '{"session_id":"workflow-2","transcript_path":"/tmp/workflow.log","message":"Report generation completed"}' | claude-code-notifications --sound Ping
}

# Step 3: Upload results
upload_results() {
    echo "Uploading results..."
    sleep 2
    echo '{"session_id":"workflow-3","transcript_path":"/tmp/workflow.log","message":"Results uploaded successfully"}' | claude-code-notifications --sound Glass
}

# Execute workflow
process_data
generate_reports
upload_results

echo "Workflow completed!"
```

## Sound Reference

### Available System Sounds (macOS)

| Sound Name | Description | Use Case |
|------------|-------------|----------|
| **Glass** | Default glass tap sound | General notifications |
| **Submarine** | Sonar ping sound | Attention-required alerts |
| **Frog** | Frog croak sound | Fun/playful notifications |
| **Purr** | Cat purring sound | Subtle, gentle alerts |
| **Basso** | Low bass sound | Error/warning alerts |
| **Blow** | Whistle blow sound | Time-sensitive alerts |
| **Hero** | Heroic fanfare sound | Success/completion alerts |
| **Morse** | Morse code beeping | Code/technical notifications |
| **Ping** | Simple ping sound | Progress updates |
| **Tink** | Metal tink sound | Lightweight notifications |

### Custom Sound Files
- Supported formats: `.wav`, `.aiff`, `.mp3`, `.m4a`, `.caf`
- Maximum recommended size: 1MB
- Should be short (1-3 seconds for best UX)

## Error Handling Examples

### Graceful Error Handling in Scripts
```bash
#!/bin/bash
# error_handling.sh

send_notification_safe() {
    local json_input="$1"
    local sound="${2:-}"

    local cmd="claude-code-notifications"
    if [ -n "$sound" ]; then
        cmd="$cmd --sound $sound"
    fi

    if ! echo "$json_input" | $cmd 2>/dev/null; then
        echo "Warning: Notification failed (continuing execution)" >&2
        # Continue execution without failing
        return 1
    fi
    return 0
}

# Usage - script continues even if notification fails
send_notification_safe '{"session_id":"test","transcript_path":"/tmp/test.md","message":"Important message"}' "Glass"

# Important processing continues...
echo "Continuing with important work..."
```

### Validation Script
```bash
#!/bin/bash
# validate_notification.sh

validate_notification_json() {
    local json_file="$1"

    # Check if jq is available for validation
    if command -v jq >/dev/null 2>&1; then
        if ! jq empty "$json_file" 2>/dev/null; then
            echo "Error: Invalid JSON in $json_file"
            return 1
        fi

        # Check required fields
        local session_id=$(jq -r '.session_id // empty' "$json_file")
        local message=$(jq -r '.message // empty' "$json_file")

        if [ -z "$session_id" ] || [ -z "$message" ]; then
            echo "Error: Missing required fields in $json_file"
            return 1
        fi
    fi

    return 0
}

# Usage
if validate_notification_json "notification.json"; then
    claude-code-notifications < "notification.json"
else
    echo "Skipping invalid notification"
fi
```

## Performance Tips

1. **Use release builds**: Always use `make build-release` for production
2. **Keep sounds short**: Custom sounds should be 1-3 seconds maximum
3. **Batch notifications**: Avoid sending notifications more than once per second
4. **Error resilience**: Use the `send_notification_safe` pattern above for critical workflows
5. **Logging**: Add logging to debug notification issues:
   ```bash
   RUST_LOG=debug claude-code-notifications --sound Glass < notification.json 2>&1 | tee notification.log
   ```