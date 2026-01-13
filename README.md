# claude-code-notifications

High-performance Rust CLI tool for Claude Code desktop notifications.

## Overview

`claude-code-notifications` is a cross-platform desktop notification system designed specifically for Claude Code hooks. It receives JSON input from Claude Code and displays desktop notifications with optional sound playback.

## Features

- ✅ **Multi-channel support** - System notifications, WeChat Work, Feishu/Lark, DingTalk
- ✅ **Multiple instances** - Configure multiple channels of the same type (e.g., personal & team DingTalk)
- ✅ **Intelligent routing** - Route notifications based on hook types and message patterns
- ✅ **Web UI** - Visual configuration interface at http://localhost:3000
- ✅ **Cross-platform notifications** (Windows, macOS, Linux via `notify-rust`)
- ✅ **Sound support** with system sounds and custom audio files
- ✅ **Parallel execution** - notifications and sounds play simultaneously
- ✅ **Error resilience** - graceful degradation if sound playback fails
- ✅ **JSON input** via stdin for easy Claude Code hook integration
- ✅ **Optimized builds** with LTO and minimal binary size

## Installation

### Prerequisites

1. **Install Rust** (if not already installed):
   ```bash
   # macOS/Linux
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

   # Windows (with PowerShell)
   winget install --id Rustlang.Rustup
   ```

2. **Verify installation**:
   ```bash
   rustc --version
   cargo --version
   ```

### Building from Source

```bash
# Clone the repository
git clone <repository-url>
cd claude-code-notifications

# Build optimized release binary
make build-release

# Install globally
make install
```

### Alternative Installation

```bash
# Install directly with cargo
cargo install --path .
```

## Usage as a Claude Code Hook

### Basic Configuration

Add to your Claude Code settings file (typically `~/.config/claude-code/config.json` or similar):

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

### Advanced Configuration with Custom Sound

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

## JSON Input Format

The CLI receives JSON via stdin with the following structure:

```json
{
  "session_id": "string - Claude session identifier",
  "transcript_path": "string? - Optional path to session transcript file",
  "message": "string - Notification body text",
  "title": "string? - Optional notification title (defaults to 'Claude Code')"
}
```

## Multi-Channel Configuration

The tool supports multiple notification channels with intelligent routing rules. Configuration is stored in `~/.claude-code-notifications.json`.

### Channel Types

- **system** - Desktop notifications (default)
- **dingtalk** - DingTalk webhook notifications
- **feishu** - Feishu/Lark webhook notifications
- **wechat** - WeChat Work webhook notifications

### Multiple Instances

You can configure multiple instances of the same channel type:

```json
{
  "version": "1.0",
  "channels": {
    "system": {
      "name": "系统通知",
      "channel_type": "system",
      "enabled": true,
      "sound": "Glass"
    },
    "dingtalk_personal": {
      "name": "个人钉钉",
      "channel_type": "dingtalk",
      "enabled": true,
      "webhook_url": "https://oapi.dingtalk.com/robot/send?access_token=YOUR_TOKEN"
    },
    "dingtalk_team": {
      "name": "团队协作群",
      "channel_type": "dingtalk",
      "enabled": true,
      "webhook_url": "https://oapi.dingtalk.com/robot/send?access_token=YOUR_TEAM_TOKEN"
    }
  },
  "routing_rules": [
    {
      "name": "All notifications to personal DingTalk",
      "match": { "hook_types": [] },
      "channels": ["system", "dingtalk_personal"],
      "enabled": true
    }
  ]
}
```

### Web UI Configuration

Launch the web configuration interface:

```bash
# Start web UI (opens browser automatically)
cargo run -- ui

# Custom port without auto-opening
cargo run -- ui --port 8080 --no-open
```

The web UI allows you to:
- Configure channel settings visually
- Test channel connectivity
- Set up routing rules
- Customize message templates

### Command-Line Channel Override

Bypass routing rules and send to specific channels:

```bash
echo '{"hook_type":"Notification","session_id":"test","message":"Test"}' | \
  cargo run -- --channels system,dingtalk_personal
```

## Sound System

By default, the notification will play the **Hero** system sound. You can customize this using the `--sound` parameter.

### System Sounds (macOS)

Use system sound names without path separators:

```bash
# Default sound (automatically plays)
claude-code-notifications

# Custom system sounds
claude-code-notifications --sound Glass      # Classic glass tap
claude-code-notifications --sound Submarine  # Recommended alternative
claude-code-notifications --sound Frog       # Fun option
claude-code-notifications --sound Purr       # Subtle notification

# Disable sound (use empty string)
claude-code-notifications --sound ""
```

**Full list of macOS system sounds**: Glass, Submarine, Frog, Purr, Basso, Blow, Bottle, Funk, Hero, Morse, Ping, Pop, Sosumi, Tink

### Custom Audio Files

Use full paths or relative paths with audio file extensions:

```bash
# Custom sound files (supports .wav, .aiff, .mp3, .m4a, etc.)
claude-code-notifications --sound ./assets/notification.wav
claude-code-notifications --sound /Users/me/sounds/alert.m4a
claude-code-notifications --sound ~/Music/custom-alert.aiff
```

## Development

### Build Commands

```bash
# Development build
make build

# Optimized release build (recommended)
make build-release

# Install globally
make install

# Run tests
make test

# Format code
make fmt

# Lint with clippy
make clippy

# Clean build artifacts
make clean

# Show all commands
make help
```

### Manual Testing

```bash
# Test with default sound
echo '{"session_id":"test","transcript_path":"/tmp/test.md","message":"Test notification","title":"Test Title"}' | cargo run

# Test with system sound
echo '{"session_id":"test","transcript_path":"/tmp/test.md","message":"Submarine sound test","title":"Sound Test"}' | cargo run -- --sound Submarine

# Test with custom sound
echo '{"session_id":"test","transcript_path":"/tmp/test.md","message":"Custom sound test","title":"Custom Test"}' | cargo run -- --sound ./assets/notification.wav
```

## Architecture

### Project Structure

```
claude-code-notifications/
├── Cargo.toml            # Rust dependencies and configuration
├── Makefile              # Build automation
├── src/
│   ├── main.rs          # CLI entry point with argument parsing
│   ├── lib.rs           # Core notification and sound logic
│   └── error.rs         # Structured error handling
└── README.md            # This file
```

### Key Dependencies

- **`notify-rust`** - Cross-platform desktop notifications
- **`clap`** - Command-line argument parsing
- **`serde`/`serde_json`** - JSON serialization/deserialization
- **`anyhow`** - Simplified error handling
- **`thiserror`** - Structured error types

## Troubleshooting

### Common Issues

**1. "Command not found: cargo"**
- Install Rust toolchain using the instructions above

**2. Sound not playing**
- Verify `afplay` is available: `which afplay`
- Test system sound directly: `afplay /System/Library/Sounds/Glass.aiff`
- Check file permissions for custom audio files

**3. Notification not displaying**
- Check system notification settings
- Verify `notify-rust` compatibility with your OS

**4. JSON parsing errors**
- Validate JSON input format
- Ensure all required fields are present

### Debugging

```bash
# Run with verbose output
RUST_LOG=debug cargo run -- --sound Glass < test-input.json

# Check installed version
claude-code-notifications --version
```

## License

MIT License - see [LICENSE](LICENSE) file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `make test && make fmt && make clippy`
5. Submit a pull request 
