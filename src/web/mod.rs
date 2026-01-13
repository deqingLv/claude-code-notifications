//! Web UI module for configuration management
//!
//! This module provides a web-based configuration interface using Actix-web,
//! allowing users to visually configure notification channels and routing rules.

pub mod server;

pub use server::start_web_server;
