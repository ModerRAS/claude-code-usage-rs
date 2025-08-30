//! ccusage-rs - Rust implementation of Claude Code usage analysis tool
//! 
//! This is a comprehensive tool for analyzing and tracking usage patterns,
//! costs, and performance metrics for Claude Code and other AI services.
//! 
//! ## Features
//! 
//! - **Usage Analysis**: Detailed analysis of API usage patterns and trends
//! - **Cost Tracking**: Comprehensive cost calculation and budget management
//! - **Data Processing**: Support for multiple data sources and formats
//! - **Visualization**: Rich output formats including tables and charts
//! - **Insights**: Intelligent recommendations and anomaly detection
//! - **MCP Server**: Model Context Protocol server integration
//! - **CLI Interface**: Full-featured command-line interface
//! 
//! ## Quick Start
//! 
//! ```rust
//! use ccusage_rs::{ConfigManager, data::DataLoader, analysis::CostCalculator};
//! 
//! // Load configuration
//! let config_manager = ConfigManager::new()?;
//! 
//! // Load usage data
//! let data_loader = DataLoader::with_source(
//!     ccusage_rs::data::DataSourceType::Json,
//!     "./data/usage.json".to_string(),
//! );
//! let records = data_loader.load_usage_data().await?;
//! 
//! // Calculate costs
//! let calculator = CostCalculator::default();
//! let total_cost = calculator.calculate_total_cost(&records)?;
//! 
//! println!("Total cost: ${}", total_cost);
//! ```
//! 
//! ## Configuration
//! 
//! The application can be configured through:
//! - Configuration file (`~/.config/ccusage-rs/config.toml`)
//! - Environment variables (prefixed with `CCUSAGE_`)
//! - Command-line arguments
//! 
//! ## Data Sources
//! 
//! Supported data sources:
//! - JSON files
//! - CSV files
//! - SQLite databases
//! - REST APIs
//! - Claude Code export format
//! - Custom log formats

pub mod error;
pub mod logging;
pub mod utils;
pub mod config;
pub mod data;
pub mod analysis;
pub mod commands;
pub mod output;
pub mod mcp;
pub mod cli;
pub mod simple_app;
pub mod minimal_app;
pub mod test_app;

// Re-export commonly used types and functions
pub use error::{CcusageError, Result};
pub use config::ConfigManager;
pub use data::*;
pub use analysis::*;
pub use cli::App;
pub use simple_app::*;
pub use minimal_app::*;
pub use test_app::*;

/// Application version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
pub const NAME: &str = "ccusage-rs";

/// Initialize the application
pub async fn init() -> Result<()> {
    // Initialize logging
    let config_manager = ConfigManager::new()?;
    let config = config_manager.get_config();
    
    logging::init_logging(config.app.verbose)?;
    
    // Validate configuration
    config_manager.validate_config()?;
    
    Ok(())
}

/// Run the application with command-line arguments
pub async fn run() -> Result<()> {
    init().await?;
    
    let app = App::new();
    app.run().await
}

/// Get system information
pub fn get_system_info() -> std::collections::HashMap<String, String> {
    utils::get_system_info()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_init() {
        let result = init().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
        assert_eq!(NAME, "ccusage-rs");
    }

    #[test]
    fn test_system_info() {
        let info = get_system_info();
        assert!(info.contains_key("os"));
        assert!(info.contains_key("arch"));
        assert!(info.contains_key("version"));
    }
}