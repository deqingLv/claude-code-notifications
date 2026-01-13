//! CLI entry point for claude-code-notifications
//!
//! This program receives JSON input from Claude Code hooks via stdin,
//! parses command-line arguments, and displays desktop notifications
//! with optional sound playback.

use clap::Parser;
use claude_code_notifications::{parse_input, handle_hook, NotificationError};
use std::fs;
use std::path::PathBuf;

/// Hook types that can be configured
#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
enum HookType {
    Notification,
    PreToolUse,
    Stop,
    SubagentStop,
}

/// Command-line arguments for claude-code-notifications
#[derive(Parser, Debug)]
#[command(
    name = "claude-code-notifications",
    about = "Claude Code hook for cross-platform desktop notifications",
    version,
    long_about = "A CLI tool for Claude Code desktop notifications with automatic hook configuration.

Subcommands:
  run     - Receive JSON input and display notification (default)
  init    - Configure Claude Code hooks automatically

JSON input format for 'run' command:
{
  \"session_id\": \"string - Claude session identifier\",
  \"transcript_path\": \"string? - Optional path to session transcript file\",
  \"message\": \"string - Notification body text\",
  \"title\": \"string? - Optional notification title (defaults to 'Claude Code')\"
}

Sound parameter options:
- Default sound: Hero (plays automatically unless disabled)
- System sounds: --sound {SoundName} (Glass, Submarine, Frog, Purr, Basso, Blow, Bottle, Funk, Hero, Morse, Ping, Pop, Sosumi, Tink)
- Custom audio files: --sound {/path/to/file} (.wav, .aiff, .mp3, .m4a, etc.)
"
)]
struct Cli {
    /// Sound to play with notification (system sound name or path to audio file)
    #[arg(short, long, default_value = "Hero")]
    sound: String,

    /// Subcommand to execute
    #[command(subcommand)]
    command: Option<Commands>,
}

/// Available subcommands
#[derive(Parser, Debug)]
enum Commands {
    /// Run notification handler (default command)
    Run(RunArgs),

    /// Initialize Claude Code hooks configuration
    Init(InitArgs),
}

/// Arguments for the run command
#[derive(Parser, Debug)]
struct RunArgs {
    /// Sound to play with notification (system sound name or path to audio file)
    #[arg(short, long, default_value = "Hero")]
    sound: String,
}

/// Arguments for the init command
#[derive(Parser, Debug)]
struct InitArgs {
    /// Overwrite existing hook configuration without prompting
    #[arg(long)]
    force: bool,

    /// Sound to configure in the hook (system sound name or path to audio file)
    #[arg(short, long, default_value = "Hero")]
    sound: String,

    /// Hook types to configure (can be specified multiple times)
    #[arg(long, value_enum, default_values_t = vec![HookType::Notification, HookType::PreToolUse, HookType::Stop, HookType::SubagentStop])]
    hook_type: Vec<HookType>,

    /// Matcher pattern for PreToolUse hooks (default: "ExitPlanMode|AskUserQuestion")
    #[arg(long, default_value = "ExitPlanMode|AskUserQuestion")]
    pre_tool_use_matcher: String,

    /// Custom configuration file path (default: ~/.claude/settings.json)
    #[arg(long)]
    config: Option<String>,
}

fn main() -> Result<(), NotificationError> {
    // Parse command-line arguments
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Run(run_args)) => run_command(run_args),
        Some(Commands::Init(init_args)) => init_command(init_args),
        None => {
            // Default to run command with sound from top-level argument
            let run_args = RunArgs { sound: cli.sound };
            run_command(run_args)
        }
    }
}

/// Handle the run command - display notification from JSON input
fn run_command(args: RunArgs) -> Result<(), NotificationError> {
    // Parse JSON input from stdin
    let input = parse_input()?;

    // Send notification with sound (defaults to Hero, empty string disables sound)
    let sound_param = if args.sound.is_empty() {
        None
    } else {
        Some(args.sound.as_str())
    };
    handle_hook(&input, sound_param)?;

    // Give time for sound to start playing and for any error messages to be printed
    std::thread::sleep(std::time::Duration::from_millis(1500));

    Ok(())
}

/// Handle the init command - configure Claude Code hooks
fn init_command(args: InitArgs) -> Result<(), NotificationError> {
    // Get configuration file path
    let config_path = match args.config {
        Some(path) => {
            let expanded = shellexpand::full(&path).map_err(|e| {
                NotificationError::InvalidInput(format!("Failed to expand path: {}", e))
            })?;
            PathBuf::from(expanded.into_owned())
        }
        None => {
            let mut path = dirs::home_dir().ok_or_else(|| {
                NotificationError::InvalidInput("Could not determine home directory".to_string())
            })?;
            path.push(".claude");
            path.push("settings.json");
            path
        }
    };

    println!("Configuring Claude Code hooks...");
    println!("Config file: {}", config_path.display());

    // Read existing configuration or create new
    let mut config: serde_json::Value = if config_path.exists() {
        let content = fs::read_to_string(&config_path).map_err(|e| {
            NotificationError::IoError(e)
        })?;
        serde_json::from_str(&content).map_err(|e| {
            NotificationError::InvalidInput(format!("Invalid JSON in config file: {}", e))
        })?
    } else {
        serde_json::json!({})
    };

    // Ensure hooks object exists
    if !config.is_object() {
        return Err(NotificationError::InvalidInput(
            "Configuration must be a JSON object".to_string()
        ));
    }

    let config_obj = config.as_object_mut().unwrap();

    // Ensure hooks object exists within config
    if !config_obj.contains_key("hooks") {
        config_obj.insert("hooks".to_string(), serde_json::json!({}));
    }

    let hooks = config_obj.get_mut("hooks").unwrap().as_object_mut().ok_or_else(|| {
        NotificationError::InvalidInput("hooks must be a JSON object".to_string())
    })?;

    // Build command with sound parameter
    let command = if args.sound.is_empty() {
        "claude-code-notifications".to_string()
    } else {
        format!("claude-code-notifications --sound {}", args.sound)
    };

    // Configure each requested hook type
    let mut configured_hooks = Vec::new();
    for hook_type in &args.hook_type {
        let hook_type_name = match hook_type {
            HookType::Notification => "Notification",
            HookType::PreToolUse => "PreToolUse",
            HookType::Stop => "Stop",
            HookType::SubagentStop => "SubagentStop",
        };

        // Check if hook already exists
        if hooks.contains_key(hook_type_name) {
            if !args.force {
                println!("{} hook already exists in config file. Skipping...", hook_type_name);
                continue;
            }
            println!("Overwriting existing {} hook configuration...", hook_type_name);
        }

        // Determine matcher based on hook type
        let matcher = match hook_type {
            HookType::PreToolUse => args.pre_tool_use_matcher.as_str(),
            _ => "", // Empty matcher for other hook types
        };

        // Create hook configuration
        let hook_config = serde_json::json!([
            {
                "matcher": matcher,
                "hooks": [
                    {
                        "type": "command",
                        "command": command
                    }
                ]
            }
        ]);

        hooks.insert(hook_type_name.to_string(), hook_config);
        configured_hooks.push(hook_type_name);
    }

    if configured_hooks.is_empty() {
        println!("No hooks were configured (all requested hooks already exist).");
        println!("Use --force to overwrite existing configurations.");
        return Ok(());
    }

    // Create parent directories if needed
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            NotificationError::IoError(e)
        })?;
    }

    // Write updated configuration back to file
    let updated_content = serde_json::to_string_pretty(&config).map_err(|e| {
        NotificationError::InvalidInput(format!("Failed to serialize config: {}", e))
    })?;

    fs::write(&config_path, updated_content).map_err(|e| {
        NotificationError::IoError(e)
    })?;

    println!("Successfully configured Claude Code hooks!");
    println!("Configured hooks: {}", configured_hooks.join(", "));
    println!("Sound: {}",
             if args.sound.is_empty() { "none (disabled)" } else { &args.sound });
    if args.hook_type.contains(&HookType::PreToolUse) {
        println!("PreToolUse matcher: {}", args.pre_tool_use_matcher);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn test_cli_args() {
        // Verify that Cli implements CommandFactory
        Cli::command().debug_assert();
    }

    #[test]
    fn test_run_with_sound() {
        // Test parsing CLI arguments with sound parameter
        let cli = Cli::try_parse_from(["claude-code-notifications", "run", "--sound", "Glass"]).unwrap();
        match cli.command {
            Some(Commands::Run(run_args)) => assert_eq!(run_args.sound, "Glass"),
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_run_without_sound() {
        // Test parsing CLI arguments without sound parameter (should use default Hero)
        let cli = Cli::try_parse_from(["claude-code-notifications", "run"]).unwrap();
        match cli.command {
            Some(Commands::Run(run_args)) => assert_eq!(run_args.sound, "Hero"),
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_run_with_custom_sound() {
        // Test parsing CLI arguments with custom sound file
        let cli = Cli::try_parse_from([
            "claude-code-notifications",
            "run",
            "--sound",
            "./assets/notification.wav",
        ])
        .unwrap();
        match cli.command {
            Some(Commands::Run(run_args)) => assert_eq!(run_args.sound, "./assets/notification.wav"),
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn test_init_command() {
        // Test parsing init command with default values
        let cli = Cli::try_parse_from(["claude-code-notifications", "init"]).unwrap();
        match cli.command {
            Some(Commands::Init(init_args)) => {
                assert_eq!(init_args.sound, "Hero");
                assert!(!init_args.force);
                assert!(init_args.config.is_none());
            }
            _ => panic!("Expected Init command"),
        }
    }

    #[test]
    fn test_init_with_options() {
        // Test parsing init command with all options
        let cli = Cli::try_parse_from([
            "claude-code-notifications",
            "init",
            "--force",
            "--sound",
            "Submarine",
            "--config",
            "/tmp/test-config.json",
        ])
        .unwrap();
        match cli.command {
            Some(Commands::Init(init_args)) => {
                assert_eq!(init_args.sound, "Submarine");
                assert!(init_args.force);
                assert_eq!(init_args.config, Some("/tmp/test-config.json".to_string()));
            }
            _ => panic!("Expected Init command"),
        }
    }
}