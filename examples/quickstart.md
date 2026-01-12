# Quick Start Guide

Get up and running with `claude-code-notifications` in minutes.

## Prerequisites

1. **Install Rust** (if not installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. **Clone and build**:
   ```bash
   # Clone the repository
   git clone <repository-url>
   cd claude-code-notifications

   # Build and install
   make build-release
   make install
   ```

## Basic Setup

1. **Configure Claude Code**:

   Edit your Claude Code settings file (usually at `~/.config/claude-code/config.json`) and add:

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

2. **Test the configuration**:

   ```bash
   # Send a test notification
   echo '{"session_id":"test","transcript_path":"/tmp/test.md","message":"Claude Code notification test","title":"Test"}' | claude-code-notifications
   ```

## Common Configurations

### 1. Default (No Sound)
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

### 2. With System Sound
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

### 3. With Custom Sound
```json
{
  "hooks": {
    "Notification": [
      {
        "type": "command",
        "command": "claude-code-notifications --sound /path/to/your/sound.wav"
      }
    ]
  }
}
```

## Testing Commands

### Manual Testing
```bash
# Basic test
echo '{"session_id":"test","transcript_path":"/tmp/test.md","message":"Test message","title":"Test Title"}' | claude-code-notifications

# Test with sound
echo '{"session_id":"test","transcript_path":"/tmp/test.md","message":"Test with sound","title":"Sound Test"}' | claude-code-notifications --sound Glass

# Test with different system sounds
echo '{"session_id":"test","transcript_path":"/tmp/test.md","message":"Submarine test","title":"Submarine"}' | claude-code-notifications --sound Submarine
```

### Run Test Script
```bash
# Make the test script executable
chmod +x examples/test.sh

# Run comprehensive tests
./examples/test.sh
```

## Troubleshooting

### No notifications appearing?
- Check system notification settings
- Verify Claude Code is actually triggering notifications
- Test the CLI directly using the manual testing commands above

### Sound not playing?
- Verify `afplay` is available: `which afplay`
- Test a system sound directly: `afplay /System/Library/Sounds/Glass.aiff`
- Check file permissions for custom audio files

### Build errors?
- Ensure Rust is properly installed: `rustc --version`
- Clear build cache: `cargo clean && make build-release`

## Next Steps

1. **Customize sounds**: Try different system sounds or add your own audio files
2. **Integrate with workflows**: Use as part of larger automation scripts
3. **Contribute**: Check out the source code and consider adding features