#!/bin/bash

# Test script for claude-code-notifications
# This script demonstrates various test scenarios for the notification system

echo "=== Claude Code Notifications Test Script ==="
echo

# Create a test input file
TEST_INPUT='{
  "session_id": "test-session-123",
  "transcript_path": "/tmp/test-transcript.md",
  "message": "Test notification from automated script",
  "title": "Test Script"
}'

echo "1. Testing with default configuration (no sound)..."
echo "$TEST_INPUT" | cargo run
echo

echo "2. Testing with system sound (Glass)..."
echo "$TEST_INPUT" | cargo run -- --sound Glass
echo

echo "3. Testing with system sound (Submarine)..."
echo "$TEST_INPUT" | cargo run -- --sound Submarine
echo

echo "4. Testing with system sound (Frog)..."
echo "$TEST_INPUT" | cargo run -- --sound Frog
echo

# Create a custom sound test if a file exists
CUSTOM_SOUND="./examples/test-sound.wav"
if [ -f "$CUSTOM_SOUND" ]; then
    echo "5. Testing with custom sound file..."
    echo "$TEST_INPUT" | cargo run -- --sound "$CUSTOM_SOUND"
else
    echo "5. Skipping custom sound test (test-sound.wav not found in examples/)"
    echo "   To test custom sounds, place a .wav file in examples/ directory"
fi
echo

echo "6. Testing invalid JSON input..."
echo '{"invalid": json}' | cargo run
echo

echo "7. Testing missing required fields..."
echo '{"session_id": "test", "message": ""}' | cargo run
echo

echo "=== Test Script Complete ==="
echo
echo "Note: These tests require:"
echo "  1. Rust toolchain installed (cargo available)"
echo "  2. Build completed: 'cargo build --release' or 'make build-release'"
echo "  3. Desktop notification permissions enabled on your system"
echo "  4. Sound system available (for sound tests)"