//! Logging module for ccusage-rs
//! 
//! This module provides centralized logging configuration and utilities
//! for the application.

use tracing::{debug, error, info, span, warn, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, fmt};
use crate::error::Result;

/// Initialize logging for the application
pub fn init_logging(verbose: bool) -> Result<()> {
    // Set up the default log level based on verbose flag
    let default_level = if verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };

    // Create environment filter
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(default_level.to_string()));

    // Configure the subscriber
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Logging initialized with level: {}", default_level);
    Ok(())
}

/// Create a span for operation tracking
pub fn create_operation_span(operation: &str) -> span::Span {
    span!(Level::INFO, "operation", operation)
}

/// Log the start of an operation
pub fn log_operation_start(operation: &str) {
    let span = create_operation_span(operation);
    let _enter = span.enter();
    info!("Starting operation: {}", operation);
}

/// Log the completion of an operation
pub fn log_operation_complete(operation: &str, duration_ms: u64) {
    let span = create_operation_span(operation);
    let _enter = span.enter();
    info!("Completed operation: {} in {}ms", operation, duration_ms);
}

/// Log an error with context
pub fn log_error_with_context(error: &dyn std::fmt::Display, context: &str) {
    error!("{}: {}", context, error);
}

/// Log a warning with context
pub fn log_warning_with_context(warning: &str, context: &str) {
    warn!("{}: {}", context, warning);
}

/// Log debug information
pub fn log_debug(message: &str, fields: &[(&str, &str)]) {
    let span = span!(Level::DEBUG, "debug");
    let _enter = span.enter();
    
    if fields.is_empty() {
        debug!("{}", message);
    } else {
        let field_str = fields
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(", ");
        debug!("{} [{}]", message, field_str);
    }
}

/// Log performance metrics
pub fn log_performance_metric(operation: &str, metric: &str, value: f64) {
    debug!("Performance metric - {}: {} = {}", operation, metric, value);
}

/// Configuration for logging
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub level: Level,
    pub enable_file_logging: bool,
    pub log_file_path: Option<String>,
    pub enable_json_format: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: Level::INFO,
            enable_file_logging: false,
            log_file_path: None,
            enable_json_format: false,
        }
    }
}

impl LoggingConfig {
    /// Create a new logging configuration from environment variables
    pub fn from_env() -> Self {
        let level = std::env::var("CCUSAGE_LOG_LEVEL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(Level::INFO);

        let enable_file_logging = std::env::var("CCUSAGE_LOG_TO_FILE")
            .map(|s| s == "true" || s == "1")
            .unwrap_or(false);

        let log_file_path = std::env::var("CCUSAGE_LOG_FILE").ok();

        let enable_json_format = std::env::var("CCUSAGE_LOG_JSON")
            .map(|s| s == "true" || s == "1")
            .unwrap_or(false);

        Self {
            level,
            enable_file_logging,
            log_file_path,
            enable_json_format,
        }
    }

    /// Initialize logging with this configuration
    pub fn init(&self) -> Result<()> {
        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(self.level.to_string()));

        let subscriber = tracing_subscriber::registry()
            .with(filter);

        if self.enable_json_format {
            subscriber.with(fmt::layer().json()).init();
        } else {
            subscriber.with(fmt::layer()).init();
        }

        info!("Logging initialized with config: {:?}", self);
        Ok(())
    }
}

/// Utility for timing operations
pub struct OperationTimer {
    start: std::time::Instant,
    operation: String,
}

impl OperationTimer {
    /// Start timing an operation
    pub fn start(operation: &str) -> Self {
        log_operation_start(operation);
        Self {
            start: std::time::Instant::now(),
            operation: operation.to_string(),
        }
    }

    /// Stop timing and log the duration
    pub fn stop(self) {
        let duration = self.start.elapsed();
        log_operation_complete(&self.operation, duration.as_millis() as u64);
    }

    /// Get the elapsed duration without stopping
    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }
}

/// Macro for timing operations
#[macro_export]
macro_rules! time_operation {
    ($operation:expr, $block:block) => {{
        let _timer = $crate::logging::OperationTimer::start($operation);
        let result = $block;
        _timer.stop();
        result
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_config_from_env() {
        std::env::set_var("CCUSAGE_LOG_LEVEL", "DEBUG");
        std::env::set_var("CCUSAGE_LOG_TO_FILE", "true");
        std::env::set_var("CCUSAGE_LOG_FILE", "/tmp/test.log");
        std::env::set_var("CCUSAGE_LOG_JSON", "true");

        let config = LoggingConfig::from_env();
        assert_eq!(config.level, Level::DEBUG);
        assert!(config.enable_file_logging);
        assert_eq!(config.log_file_path, Some("/tmp/test.log".to_string()));
        assert!(config.enable_json_format);

        // Clean up
        std::env::remove_var("CCUSAGE_LOG_LEVEL");
        std::env::remove_var("CCUSAGE_LOG_TO_FILE");
        std::env::remove_var("CCUSAGE_LOG_FILE");
        std::env::remove_var("CCUSAGE_LOG_JSON");
    }

    #[test]
    fn test_operation_timer() {
        let timer = OperationTimer::start("test_operation");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let elapsed = timer.elapsed();
        assert!(elapsed >= std::time::Duration::from_millis(10));
    }
}