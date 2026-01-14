//! Debug logging utilities
//!
//! Simple logging utilities for debugging notification delivery.

use crate::config::AppConfig;

/// Global debug flag (set from config)
static mut DEBUG_ENABLED: bool = false;

/// Initialize debug logging from config
pub fn init_debug(config: &AppConfig) {
    unsafe {
        DEBUG_ENABLED = config.debug;
    }
}

/// Check if debug logging is enabled
pub fn is_debug_enabled() -> bool {
    unsafe { DEBUG_ENABLED }
}

/// Log debug message
#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if $crate::logging::is_debug_enabled() {
            eprintln!("[DEBUG] {}", format_args!($($arg)*));
        }
    };
}

/// Log debug message with context
#[macro_export]
macro_rules! debug_context {
    ($context:expr, $($arg:tt)*) => {
        if $crate::logging::is_debug_enabled() {
            eprintln!("[DEBUG] [{}] {}", $context, format_args!($($arg)*));
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_disabled_by_default() {
        assert!(!is_debug_enabled());
    }
}
