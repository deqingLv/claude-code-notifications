# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a high-performance Rust CLI tool designed as a Claude Code hook for displaying cross-platform desktop notifications.

## Build Commands

### Using Make (Recommended)

The project includes a comprehensive Makefile with optimized commands:

- `make build-release` - Compile optimized release binary with full optimizations
- `make install` - Build and install CLI globally as `claude-code-notification`
- `make test` - Run complete test suite with coverage
- `make fmt` - Format Rust code using rustfmt with project settings
- `make clippy` - Lint Rust code with clippy for code quality
- `make clean` - Clean all build artifacts and target directory
- `make help` - Show all available make targets with descriptions

### Using Cargo Directly

For direct cargo usage when Make is unavailable:

- `cargo build` - Compile debug binary with symbols
- `cargo build --release` - Compile optimized release binary
- `cargo run` - Run CLI from source with development settings
- `cargo install --path .` - Install CLI globally from current directory
- `cargo test` - Run test suite with default settings
- `cargo fmt` - Format Rust code with default configuration
- `cargo clippy` - Run clippy linter with default rules

## Usage as a Claude Code Hook

### Hook Configuration

This program integrates with Claude Code's notification system through the hooks configuration. Configure in your Claude Code settings file (`~/.claude/settings.json` on macOS):

**Automatic Configuration:**
```bash
# Configure hooks automatically (uses default sound: Hero, configures: Notification, PreToolUse, Stop, PermissionRequest)
claude-code-notifications init

# Configure with custom sound
claude-code-notifications init --sound Submarine

# Configure specific hook types only
claude-code-notifications init --hook-type Notification --hook-type PreToolUse

# Configure with custom PreToolUse matcher
claude-code-notifications init --pre-tool-use-matcher "ExitPlanMode|AskUserQuestion|Task"

# Configure with custom config file location
claude-code-notifications init --config /path/to/config.json
```

**Manual Configuration:**

**Basic Configuration:**
```json
{
  "hooks": {
    "Notification": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "claude-code-notifications"
          }
        ]
      }
    ]
  }
}
```

**Advanced Configuration with Custom Sound:**
```json
{
  "hooks": {
    "Notification": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "claude-code-notifications --sound Submarine"
          }
        ]
      }
    ]
  }
}
```

**Multiple Hook Types Configuration:**

Example with all hook types (note: SubagentStop is not installed by default):
```json
{
  "hooks": {
    "Notification": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "claude-code-notifications --sound Glass"
          }
        ]
      }
    ],
    "PreToolUse": [
      {
        "matcher": "ExitPlanMode|AskUserQuestion",
        "hooks": [
          {
            "type": "command",
            "command": "claude-code-notifications --sound Pop"
          }
        ]
      }
    ],
    "Stop": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "claude-code-notifications --sound Blow"
          }
        ]
      }
    ],
    "PermissionRequest": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "claude-code-notifications --sound Hero"
          }
        ]
      }
    ],
    "SubagentStop": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": "claude-code-notifications --sound Sosumi"
          }
        ]
      }
    ]
  }
}
```

To explicitly configure SubagentStop along with other hooks:
```bash
claude-code-notifications init --hook-type notification --hook-type pre-tool-use --hook-type stop --hook-type permission-request --hook-type subagent-stop
```

### Hook Input Schema

Hooks receive JSON data via stdin containing session information and event-specific data:

```json
{
  // Common fields
  session_id: string
  transcript_path: string  // Path to conversation JSON
  cwd: string              // The current working directory when the hook is invoked
  permission_mode: string  // Current permission mode: "default", "plan", "acceptEdits", "dontAsk", or "bypassPermissions"

  // Event-specific fields
  hook_event_name: string
  ...
}
```

具体类型的hook差异，可以直接查阅：https://code.claude.com/docs/en/hooks#hook-input


## CLI Parameters and Sound System

### Sound Parameter Options

The `--sound` parameter supports intelligent path resolution:

**System Sounds** (recommended for consistency):
- Format: `--sound {SoundName}` (no path separators)
- Resolves to: `/System/Library/Sounds/{SoundName}.aiff`
- Available: Glass (default), Submarine, Frog, Purr, Basso, Blow, Bottle, Funk, Hero, Morse, Ping, Pop, Sosumi, Tink

**Custom Audio Files** (for specialized notifications):
- Format: `--sound {/path/to/file}` (contains path separators)
- Supports: `.wav`, `.aiff`, `.mp3`, `.m4a`, and other `afplay`-compatible formats
- Examples:
  - `--sound ./assets/notification.wav`
  - `--sound /Users/dev/sounds/alert.m4a`
  - `--sound ~/Music/custom-alert.aiff`

## Development Workflow

### Local Development

**Quick Development Cycle:**
```bash
# Run with immediate feedback
cargo run

# Run with custom sound for testing
echo '{"session_id":"dev","transcript_path":"/tmp/dev.md","message":"Development test"}' | cargo run -- --sound Submarine
```

**Testing and Quality Assurance:**
```bash
# Run comprehensive tests
make test

# Check code formatting
make fmt

# Run linter for code quality
make clippy

# Full quality check pipeline
make test && make fmt && make clippy
```

### Manual Testing Scenarios

**Basic Functionality Testing:**
```bash
# Test default configuration (Notification hook)
echo '{"hook_event_name":"Notification","session_id":"test","transcript_path":"/tmp/test.md","message":"Default notification test"}' | cargo run

# Test system sound variants
echo '{"hook_event_name":"Notification","session_id":"test","transcript_path":"/tmp/test.md","message":"System sound test"}' | cargo run -- --sound Glass
echo '{"hook_event_name":"Notification","session_id":"test","transcript_path":"/tmp/test.md","message":"Submarine sound test"}' | cargo run -- --sound Submarine

# Test custom audio files
echo '{"hook_event_name":"Notification","session_id":"test","transcript_path":"/tmp/test.md","message":"Custom sound test"}' | cargo run -- --sound ./366102__original_sound__confirmation-upward.wav

# Test other hook types with unified format
echo '{"hook_event_name":"PreToolUse","session_id":"test","transcript_path":"/tmp/test.md","tool_name":"ExitPlanMode"}' | cargo run -- --sound Pop
echo '{"hook_event_name":"Stop","session_id":"test","transcript_path":"/tmp/test-transcript.jsonl"}' | cargo run -- --sound Blow
echo '{"hook_event_name":"SubagentStop","session_id":"test","transcript_path":"/tmp/test-transcript.jsonl"}' | cargo run -- --sound Sosumi
```

**Error Handling Testing:**
```bash
# Test invalid JSON handling
echo '{"invalid": json}' | cargo run 2>&1 | head -5

# Test missing sound file handling
echo '{"hook_event_name":"Notification","session_id":"test","transcript_path":"/tmp/test.md","message":"Missing sound test"}' | cargo run -- --sound /nonexistent/file.wav
```

## Architecture and Implementation

### Project Structure

The codebase follows Rust best practices with clear separation of concerns:

- **`src/main.rs`** - CLI entry point with `clap` argument parsing and error handling
- **`src/lib.rs`** - Core notification logic, sound management, and parallel execution
- **`src/error.rs`** - Structured error types with `thiserror` for comprehensive error handling
- **`Cargo.toml`** - Dependency management with optimized release profile
- **`Makefile`** - Development workflow automation and build management

### Key Dependencies and Their Roles

**Core Functionality:**
- **`clap`** - Command-line argument parsing with derive macros for maintainability
- **`notify-rust`** - Cross-platform desktop notifications (Windows/macOS/Linux)
- **`serde`/`serde_json`** - JSON serialization/deserialization with error handling

**Error Management:**
- **`anyhow`** - Simplified error handling with context preservation
- **`thiserror`** - Structured error types with automatic trait implementations

**System Integration:**
- **`std::process`** - System command execution for `afplay` integration
- **`std::thread`** - Parallel execution of notifications and sound playback

### Performance Optimizations

**Release Build Configuration:**
```toml
[profile.release]
opt-level = 3         # Maximum optimization
lto = true           # Link-time optimization for smaller binaries
codegen-units = 1    # Single compilation unit for better optimization
panic = "abort"      # Smaller binaries by avoiding unwind handling
strip = true         # Remove debug symbols from release builds
```

**Parallel Execution Design:**
- Notifications and sound playback execute simultaneously using threading
- Non-blocking error handling ensures notification display even if sound fails
- Graceful degradation on systems without `afplay` support

### Testing Strategy

**Unit Test Coverage:**
- JSON parsing and validation with various input scenarios
- Sound path resolution logic for both system and custom sounds  
- Error handling for invalid inputs and missing files
- Cross-platform compatibility testing

**Integration Testing:**
- End-to-end notification display testing (when possible)
- Sound playback verification with different audio formats
- Claude Code hook integration validation

## Troubleshooting and Common Issues

### Development Issues

**Build Problems:**
- Ensure Rust toolchain is up-to-date: `rustup update`
- Clear build cache: `make clean` then rebuild
- Check dependency compatibility: `cargo tree` for conflicts

**Sound Issues:**
- Verify `afplay` availability: `which afplay`
- Test sound file directly: `afplay /System/Library/Sounds/Glass.aiff`
- Check file permissions for custom audio files

**Hook Integration Issues:**
- Validate JSON input format with online validators
- Test CLI independently before Claude Code integration
- Check Claude Code hook configuration syntax

### Performance Considerations

- Release builds are significantly faster than debug builds
- Custom sound files should be reasonably sized (< 1MB recommended)
- Parallel execution ensures UI responsiveness during sound playback
- Error logging helps diagnose issues without blocking functionality

This comprehensive development guide ensures efficient contribution and maintenance of the Claude Code notification system.