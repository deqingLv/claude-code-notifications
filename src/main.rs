//! CLI entry point for claude-code-notifications
//!
//! This program receives JSON input from Claude Code hooks via stdin,
//! parses command-line arguments, and displays desktop notifications
//! with optional sound playback.

use clap::Parser;
use claude_code_notifications::{parse_input, send_notification, NotificationError};

/// Command-line arguments for claude-code-notifications
#[derive(Parser, Debug)]
#[command(
    name = "claude-code-notifications",
    about = "Claude Code hook for cross-platform desktop notifications",
    version,
    long_about = "Receives JSON input from Claude Code hooks and displays desktop notifications with optional sound playback.

JSON input format:
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
}

fn main() -> Result<(), NotificationError> {
    // Parse command-line arguments
    let args = Cli::parse();

    // Parse JSON input from stdin
    let input = parse_input()?;

    // Send notification with sound (defaults to Hero, empty string disables sound)
    let sound_param = if args.sound.is_empty() {
        None
    } else {
        Some(args.sound.as_str())
    };
    send_notification(&input, sound_param)?;

    // Give time for sound to start playing and for any error messages to be printed
    std::thread::sleep(std::time::Duration::from_millis(1500));

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
    fn test_cli_with_sound() {
        // Test parsing CLI arguments with sound parameter
        let args = Cli::try_parse_from(["claude-code-notifications", "--sound", "Glass"]).unwrap();
        assert_eq!(args.sound, "Glass");
    }

    #[test]
    fn test_cli_without_sound() {
        // Test parsing CLI arguments without sound parameter (should use default Hero)
        let args = Cli::try_parse_from(["claude-code-notifications"]).unwrap();
        assert_eq!(args.sound, "Hero");
    }

    #[test]
    fn test_cli_with_custom_sound() {
        // Test parsing CLI arguments with custom sound file
        let args = Cli::try_parse_from([
            "claude-code-notifications",
            "--sound",
            "./assets/notification.wav",
        ])
        .unwrap();
        assert_eq!(args.sound, "./assets/notification.wav");
    }
}