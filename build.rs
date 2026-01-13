use std::env;
use std::process::Command;

fn main() {
    // Get git commit hash (short format)
    let commit_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // Get build timestamp (HH:MM:SS format)
    let build_timestamp = Command::new("date")
        .args(["+%H:%M:%S"])
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    // Set cargo environment variables that will be available at compile time
    println!("cargo:rustc-env=GIT_COMMIT_HASH={}", commit_hash);
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", build_timestamp);
}
