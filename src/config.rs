//! Configuration management for ccusage-rs
//! 
//! This module handles application configuration, settings, and environment variables.

use crate::data::models::*;
use crate::error::{Result, CcusageError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Application settings
    pub app: AppSettings,
    
    /// Data source configuration
    pub data_source: DataSourceConfig,
    
    /// Output configuration
    pub output: OutputConfig,
    
    /// Budget configuration
    pub budget: Option<BudgetConfig>,
    
    /// MCP server configuration
    pub mcp: Option<McpConfig>,
    
    /// Logging configuration
    pub logging: LoggingConfig,
    
    /// Cache configuration
    pub cache: CacheConfig,
}

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Application name
    pub name: String,
    
    /// Application version
    pub version: String,
    
    /// Debug mode
    pub debug: bool,
    
    /// Verbose output
    pub verbose: bool,
    
    /// Data directory
    pub data_dir: PathBuf,
    
    /// Cache directory
    pub cache_dir: PathBuf,
    
    /// Config directory
    pub config_dir: PathBuf,
}

/// Data source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceConfig {
    /// Default data source type
    pub default_source: String,
    
    /// Data source paths
    pub source_paths: HashMap<String, String>,
    
    /// API endpoints
    pub api_endpoints: HashMap<String, String>,
    
    /// Pricing data path
    pub pricing_data_path: Option<String>,
    
    /// Auto-refresh interval (seconds)
    pub auto_refresh_interval: u64,
}

/// Output configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Default output format
    pub default_format: String,
    
    /// Output directory
    pub output_dir: PathBuf,
    
    /// Table settings
    pub table: TableSettings,
    
    /// Chart settings
    pub chart: ChartSettings,
}

/// Table settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSettings {
    /// Maximum table width
    pub max_width: usize,
    
    /// Enable table colors
    pub enable_colors: bool,
    
    /// Compact mode
    pub compact: bool,
}

/// Chart settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSettings {
    /// Enable charts
    pub enable_charts: bool,
    
    /// Chart width
    pub width: usize,
    
    /// Chart height
    pub height: usize,
    
    /// Chart theme
    pub theme: String,
}

/// Budget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    /// Monthly budget limit
    pub monthly_limit: f64,
    
    /// Currency
    pub currency: String,
    
    /// Warning threshold (percentage)
    pub warning_threshold: f64,
    
    /// Alert threshold (percentage)
    pub alert_threshold: f64,
    
    /// Enable budget alerts
    pub enable_alerts: bool,
}

/// MCP server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    /// Enable MCP server
    pub enabled: bool,
    
    /// Server port
    pub port: u16,
    
    /// Server host
    pub host: String,
    
    /// Enable authentication
    pub enable_auth: bool,
    
    /// API key (if authentication enabled)
    pub api_key: Option<String>,
    
    /// Allowed origins
    pub allowed_origins: Vec<String>,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    pub level: String,
    
    /// Enable file logging
    pub enable_file_logging: bool,
    
    /// Log file path
    pub log_file: Option<PathBuf>,
    
    /// Enable JSON format
    pub json_format: bool,
    
    /// Enable colored output
    pub colored: bool,
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable caching
    pub enabled: bool,
    
    /// Cache expiration (seconds)
    pub expiration: u64,
    
    /// Max cache size (MB)
    pub max_size: u64,
    
    /// Cache cleanup interval (seconds)
    pub cleanup_interval: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app: AppSettings::default(),
            data_source: DataSourceConfig::default(),
            output: OutputConfig::default(),
            budget: None,
            mcp: None,
            logging: LoggingConfig::default(),
            cache: CacheConfig::default(),
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let app_dir = home_dir.join(".ccusage");
        
        Self {
            name: "ccusage-rs".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            debug: false,
            verbose: false,
            data_dir: app_dir.join("data"),
            cache_dir: app_dir.join("cache"),
            config_dir: app_dir.join("config"),
        }
    }
}

impl Default for DataSourceConfig {
    fn default() -> Self {
        let mut source_paths = HashMap::new();
        source_paths.insert("json".to_string(), "./data/usage.json".to_string());
        source_paths.insert("csv".to_string(), "./data/usage.csv".to_string());
        
        let mut api_endpoints = HashMap::new();
        api_endpoints.insert("claude".to_string(), "https://api.anthropic.com".to_string());
        
        Self {
            default_source: "json".to_string(),
            source_paths,
            api_endpoints,
            pricing_data_path: Some("./config/pricing.json".to_string()),
            auto_refresh_interval: 3600,
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            default_format: "table".to_string(),
            output_dir: PathBuf::from("./output"),
            table: TableSettings::default(),
            chart: ChartSettings::default(),
        }
    }
}

impl Default for TableSettings {
    fn default() -> Self {
        Self {
            max_width: 120,
            enable_colors: true,
            compact: false,
        }
    }
}

impl Default for ChartSettings {
    fn default() -> Self {
        Self {
            enable_charts: true,
            width: 80,
            height: 20,
            theme: "default".to_string(),
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            enable_file_logging: false,
            log_file: None,
            json_format: false,
            colored: true,
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            expiration: 3600,
            max_size: 100,
            cleanup_interval: 86400,
        }
    }
}

/// Configuration manager
#[derive(Clone)]
pub struct ConfigManager {
    config: AppConfig,
    config_path: PathBuf,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Result<Self> {
        let config_path = Self::get_config_file_path()?;
        let config = Self::load_or_create_config(&config_path)?;
        
        Ok(Self {
            config,
            config_path,
        })
    }
    
    /// Create a new configuration manager with specific config path
    pub fn new_with_config(config_path: &Path) -> Result<Self> {
        let config = if config_path.exists() {
            Self::load_config(config_path)?
        } else {
            AppConfig::default()
        };
        
        Ok(Self {
            config,
            config_path: config_path.to_path_buf(),
        })
    }

    /// Load configuration from file
    fn load_or_create_config(config_path: &Path) -> Result<AppConfig> {
        if config_path.exists() {
            Self::load_config(config_path)
        } else {
            let config = AppConfig::default();
            Self::save_config(&config, config_path)?;
            Ok(config)
        }
    }

    /// Load configuration from file
    fn load_config(config_path: &Path) -> Result<AppConfig> {
        let content = std::fs::read_to_string(config_path).map_err(|e| {
            CcusageError::FileSystem(format!("Failed to read config file: {}", e))
        })?;
        
        let mut config: AppConfig = toml::from_str(&content).map_err(|e| {
            CcusageError::Config(format!("Failed to parse config: {}", e))
        })?;
        
        // Apply environment variable overrides
        Self::apply_env_overrides(&mut config);
        
        Ok(config)
    }
    
    /// Apply environment variable overrides
    fn apply_env_overrides(config: &mut AppConfig) {
        if let Ok(verbose) = std::env::var("CCUSAGE_VERBOSE") {
            config.app.verbose = verbose.to_lowercase() == "true" || verbose == "1";
        }
        
        if let Ok(log_level) = std::env::var("CCUSAGE_LOG_LEVEL") {
            config.logging.level = log_level;
        }
        
        if let Ok(data_dir) = std::env::var("CCUSAGE_DATA_DIR") {
            config.app.data_dir = PathBuf::from(data_dir);
        }
    }

    /// Save configuration to file
    fn save_config(config: &AppConfig, config_path: &Path) -> Result<()> {
        // Ensure config directory exists
        if let Some(parent) = config_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    CcusageError::FileSystem(format!("Failed to create config directory: {}", e))
                })?;
            }
        }

        let config_str = toml::to_string_pretty(config).map_err(|e| {
            CcusageError::Config(format!("Failed to serialize config: {}", e))
        })?;

        std::fs::write(config_path, config_str).map_err(|e| {
            CcusageError::FileSystem(format!("Failed to write config file: {}", e))
        })?;

        Ok(())
    }

    /// Get configuration file path
    fn get_config_file_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| CcusageError::Config("Could not determine config directory".to_string()))?
            .join("ccusage-rs");

        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir).map_err(|e| {
                CcusageError::FileSystem(format!("Failed to create config directory: {}", e))
            })?;
        }

        Ok(config_dir.join("config.toml"))
    }

    /// Get current configuration
    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }

    /// Get mutable configuration
    pub fn get_config_mut(&mut self) -> &mut AppConfig {
        &mut self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, new_config: AppConfig) -> Result<()> {
        self.config = new_config;
        self.save_current_config()
    }

    /// Save current configuration
    pub fn save_current_config(&self) -> Result<()> {
        Self::save_config(&self.config, &self.config_path)
    }

    /// Set data source path
    pub fn set_data_source_path(&mut self, source_type: &str, path: &str) -> Result<()> {
        self.config.data_source.source_paths.insert(
            source_type.to_string(),
            path.to_string()
        );
        self.save_current_config()
    }

    /// Set API endpoint
    pub fn set_api_endpoint(&mut self, service: &str, endpoint: &str) -> Result<()> {
        self.config.data_source.api_endpoints.insert(
            service.to_string(),
            endpoint.to_string()
        );
        self.save_current_config()
    }

    /// Set budget configuration
    pub fn set_budget(&mut self, budget: BudgetConfig) -> Result<()> {
        self.config.budget = Some(budget);
        self.save_current_config()
    }

    /// Set log level
    pub fn set_log_level(&mut self, level: &str) -> Result<()> {
        self.config.logging.level = level.to_string();
        self.save_current_config()
    }

    /// Enable/disable verbose mode
    pub fn set_verbose(&mut self, verbose: bool) -> Result<()> {
        self.config.app.verbose = verbose;
        self.save_current_config()
    }

    /// Get budget information
    pub fn get_budget(&self) -> Option<BudgetInfo> {
        self.config.budget.as_ref().map(|budget| BudgetInfo {
            monthly_limit: budget.monthly_limit,
            currency: budget.currency.clone(),
            warning_threshold: budget.warning_threshold,
            alert_threshold: budget.alert_threshold,
        })
    }

    /// Get pricing data path
    pub fn get_pricing_data_path(&self) -> Option<&str> {
        self.config.data_source.pricing_data_path.as_deref()
    }

    /// Get data source path
    pub fn get_data_source_path(&self, source_type: &str) -> Option<&str> {
        self.config.data_source.source_paths.get(source_type).map(|s| s.as_str())
    }

    /// Check if MCP server is enabled
    pub fn is_mcp_enabled(&self) -> bool {
        self.config.mcp.as_ref().map_or(false, |mcp| mcp.enabled)
    }

    /// Get MCP server configuration
    pub fn get_mcp_config(&self) -> Option<&McpConfig> {
        self.config.mcp.as_ref()
    }

    /// Create default configuration file
    pub fn create_default_config(path: &Path) -> Result<()> {
        let config = AppConfig::default();
        Self::save_config(&config, path)
    }

    /// Validate configuration
    pub fn validate_config(&self) -> Result<()> {
        // Validate data directory
        if !self.config.app.data_dir.exists() {
            std::fs::create_dir_all(&self.config.app.data_dir).map_err(|e| {
                CcusageError::FileSystem(format!("Failed to create data directory: {}", e))
            })?;
        }

        // Validate cache directory
        if !self.config.app.cache_dir.exists() {
            std::fs::create_dir_all(&self.config.app.cache_dir).map_err(|e| {
                CcusageError::FileSystem(format!("Failed to create cache directory: {}", e))
            })?;
        }

        // Validate budget if set
        if let Some(budget) = &self.config.budget {
            if budget.monthly_limit <= 0.0 {
                return Err(CcusageError::Config(
                    "Budget limit must be positive".to_string()
                ));
            }
            if budget.warning_threshold <= 0.0 || budget.warning_threshold > 100.0 {
                return Err(CcusageError::Config(
                    "Warning threshold must be between 0 and 100".to_string()
                ));
            }
            if budget.alert_threshold <= 0.0 || budget.alert_threshold > 100.0 {
                return Err(CcusageError::Config(
                    "Alert threshold must be between 0 and 100".to_string()
                ));
            }
        }

        // Validate MCP configuration if set
        if let Some(mcp) = &self.config.mcp {
            if mcp.port == 0 {
                return Err(CcusageError::Config(
                    "MCP server port must be valid".to_string()
                ));
            }
        }

        Ok(())
    }

    /// Reset configuration to defaults
    pub fn reset_to_defaults(&mut self) -> Result<()> {
        self.config = AppConfig::default();
        self.save_current_config()
    }

    /// Export configuration to string
    pub fn export_config(&self) -> Result<String> {
        toml::to_string_pretty(&self.config).map_err(|e| {
            CcusageError::Config(format!("Failed to export config: {}", e))
        })
    }

    /// Import configuration from string
    pub fn import_config(&mut self, config_str: &str) -> Result<()> {
        let config: AppConfig = toml::from_str(config_str).map_err(|e| {
            CcusageError::Config(format!("Failed to import config: {}", e))
        })?;
        
        self.config = config;
        self.save_current_config()
    }

    /// Get configuration file path
    pub fn get_config_path(&self) -> &Path {
        &self.config_path
    }

    /// Check if configuration file exists
    pub fn config_file_exists(&self) -> bool {
        self.config_path.exists()
    }
}

/// Environment variable helper
pub struct EnvHelper;

impl EnvHelper {
    /// Get environment variable with default value
    pub fn get_env_var(key: &str, default: &str) -> String {
        env::var(key).unwrap_or_else(|_| default.to_string())
    }

    /// Get boolean environment variable
    pub fn get_bool_env_var(key: &str, default: bool) -> bool {
        match env::var(key) {
            Ok(value) => {
                value.to_lowercase() == "true" || value == "1"
            }
            Err(_) => default,
        }
    }

    /// Get integer environment variable
    pub fn get_int_env_var(key: &str, default: u64) -> u64 {
        env::var(key)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(default)
    }

    /// Get float environment variable
    pub fn get_float_env_var(key: &str, default: f64) -> f64 {
        env::var(key)
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(default)
    }

    /// Get path environment variable
    pub fn get_path_env_var(key: &str, default: &Path) -> PathBuf {
        env::var(key)
            .ok()
            .map(PathBuf::from)
            .unwrap_or_else(|| default.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_config_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");
        
        let config = AppConfig::default();
        ConfigManager::save_config(&config, &config_path).unwrap();
        
        assert!(config_path.exists());
    }

    #[test]
    fn test_config_manager() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.toml");
        
        let mut manager = ConfigManager::new().unwrap();
        manager.config_path = config_path.clone();
        
        // Test setting data source path
        manager.set_data_source_path("json", "/test/path.json").unwrap();
        assert_eq!(manager.get_data_source_path("json"), Some("/test/path.json"));
        
        // Test setting budget
        let budget = BudgetConfig {
            monthly_limit: 100.0,
            currency: "USD".to_string(),
            warning_threshold: 80.0,
            alert_threshold: 95.0,
            enable_alerts: true,
        };
        
        manager.set_budget(budget).unwrap();
        assert!(manager.get_budget().is_some());
        
        // Test config validation
        assert!(manager.validate_config().is_ok());
    }

    #[test]
    fn test_env_helper() {
        env::set_var("TEST_VAR", "test_value");
        env::set_var("TEST_BOOL", "true");
        env::set_var("TEST_INT", "42");
        env::set_var("TEST_FLOAT", "3.14");
        
        assert_eq!(EnvHelper::get_env_var("TEST_VAR", "default"), "test_value");
        assert_eq!(EnvHelper::get_bool_env_var("TEST_BOOL", false), true);
        assert_eq!(EnvHelper::get_int_env_var("TEST_INT", 0), 42);
        assert!((EnvHelper::get_float_env_var("TEST_FLOAT", 0.0) - 3.14).abs() < 0.001);
        
        // Clean up
        env::remove_var("TEST_VAR");
        env::remove_var("TEST_BOOL");
        env::remove_var("TEST_INT");
        env::remove_var("TEST_FLOAT");
    }

    #[test]
    fn test_config_defaults() {
        let config = AppConfig::default();
        
        assert_eq!(config.app.name, "ccusage-rs");
        assert_eq!(config.app.debug, false);
        assert_eq!(config.data_source.default_source, "json");
        assert_eq!(config.output.default_format, "table");
        assert_eq!(config.logging.level, "info");
        assert!(config.cache.enabled);
    }
}