//! Error handling module for ccusage-rs
//! 
//! This module defines custom error types and utilities for consistent error handling
//! across the application.

use thiserror::Error;

/// Custom error types for the ccusage-rs application
#[derive(Error, Debug)]
pub enum CcusageError {
    /// Configuration-related errors
    #[error("Configuration error: {0}")]
    Config(String),
    
    /// Data loading and parsing errors
    #[error("Data loading error: {0}")]
    DataLoading(String),
    
    /// File system errors
    #[error("File system error: {0}")]
    FileSystem(String),
    
    /// Network-related errors
    #[error("Network error: {0}")]
    Network(String),
    
    /// Data validation errors
    #[error("Validation error: {0}")]
    Validation(String),
    
    /// Input/output errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    /// JSON parsing errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    /// CSV parsing errors
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),
    
    /// Chrono parsing errors
    #[error("Date/time error: {0}")]
    Chrono(#[from] chrono::ParseError),
    
    /// Configuration parsing errors
    #[error("Config parsing error: {0}")]
    ConfigParse(#[from] config::ConfigError),
    
    /// Reqwest HTTP errors
    #[error("HTTP error: {0}")]
    Reqwest(#[from] reqwest::Error),
    
    /// URL parsing errors
    #[error("URL error: {0}")]
    Url(#[from] url::ParseError),
    
    /// Clap command line parsing errors
    #[error("Command line error: {0}")]
    Clap(#[from] clap::Error),
    
    /// Generic application errors
    #[error("Application error: {0}")]
    Application(String),
}

/// Result type alias for the application
pub type Result<T> = std::result::Result<T, CcusageError>;

/// Macro for creating configuration errors with context
#[macro_export]
macro_rules! config_error {
    ($msg:expr) => {
        $crate::error::CcusageError::Config($msg.to_string())
    };
    ($fmt:expr, $($arg:expr),+) => {
        $crate::error::CcusageError::Config(format!($fmt, $($arg),+))
    };
}

/// Macro for creating data loading errors with context
#[macro_export]
macro_rules! data_error {
    ($msg:expr) => {
        $crate::error::CcusageError::DataLoading($msg.to_string())
    };
    ($fmt:expr, $($arg:expr),+) => {
        $crate::error::CcusageError::DataLoading(format!($fmt, $($arg),+))
    };
}

/// Macro for creating validation errors with context
#[macro_export]
macro_rules! validation_error {
    ($msg:expr) => {
        $crate::error::CcusageError::Validation($msg.to_string())
    };
    ($fmt:expr, $($arg:expr),+) => {
        $crate::error::CcusageError::Validation(format!($fmt, $($arg),+))
    };
}

/// Macro for creating application errors with context
#[macro_export]
macro_rules! app_error {
    ($msg:expr) => {
        $crate::error::CcusageError::Application($msg.to_string())
    };
    ($fmt:expr, $($arg:expr),+) => {
        $crate::error::CcusageError::Application(format!($fmt, $($arg),+))
    };
}

/// Utility trait for adding context to errors
pub trait ErrorContext<T> {
    fn with_context<F, S>(self, context: F) -> Result<T>
    where
        F: FnOnce() -> S,
        S: Into<String>;
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: std::fmt::Display,
{
    fn with_context<F, S>(self, context: F) -> Result<T>
    where
        F: FnOnce() -> S,
        S: Into<String>,
    {
        self.map_err(|e| {
            let ctx = context().into();
            CcusageError::Application(format!("{}: {}", ctx, e))
        })
    }
}

/// Utility function to chain multiple results
pub fn chain_results<T, I>(results: I) -> Result<Vec<T>>
where
    I: IntoIterator<Item = Result<T>>,
{
    results.into_iter().collect()
}

/// Utility function to handle file operations with context
pub fn file_operation_context<F, T>(operation: F, file_path: &str) -> Result<T>
where
    F: FnOnce() -> std::io::Result<T>,
{
    operation().map_err(|e| {
        CcusageError::FileSystem(format!("Failed to operate on file '{}': {}", file_path, e))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_macros() {
        let err = config_error!("test config error");
        assert!(matches!(err, CcusageError::Config(_)));
        
        let err = data_error!("test data error");
        assert!(matches!(err, CcusageError::DataLoading(_)));
        
        let err = validation_error!("test validation error");
        assert!(matches!(err, CcusageError::Validation(_)));
        
        let err = app_error!("test app error");
        assert!(matches!(err, CcusageError::Application(_)));
    }

    #[test]
    fn test_error_context() {
        let result: Result<i32> = Ok(42);
        let result = result.with_context(|| "test context");
        assert!(result.is_ok());
        
        let result: std::result::Result<i32, &str> = Err("error");
        let result = result.with_context(|| "test context");
        assert!(result.is_err());
    }

    #[test]
    fn test_chain_results() {
        let results = vec![Ok(1), Ok(2), Ok(3)];
        let chained = chain_results(results);
        assert!(chained.is_ok());
        assert_eq!(chained.unwrap(), vec![1, 2, 3]);
        
        let results = vec![Ok(1), Err(CcusageError::Application("error".to_string())), Ok(3)];
        let chained = chain_results(results);
        assert!(chained.is_err());
    }
}