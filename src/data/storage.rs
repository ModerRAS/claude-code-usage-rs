//! Data storage module for ccusage-rs
//! 
//! This module handles storing usage data in various formats and locations.

use crate::data::models::*;
use crate::error::{Result, CcusageError};
use crate::utils;
use chrono::{DateTime, Utc, NaiveDate};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
// use std::io::Write; // Not currently used

/// Storage configuration
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Storage backend type
    pub backend: StorageBackend,
    
    /// Base storage directory
    pub base_dir: PathBuf,
    
    /// Enable compression
    pub enable_compression: bool,
    
    /// Enable encryption
    pub enable_encryption: bool,
    
    /// Backup enabled
    pub enable_backup: bool,
    
    /// Backup directory
    pub backup_dir: Option<PathBuf>,
    
    /// Retention policy
    pub retention: RetentionPolicy,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: StorageBackend::Json,
            base_dir: PathBuf::from("./data"),
            enable_compression: false,
            enable_encryption: false,
            enable_backup: true,
            backup_dir: None,
            retention: RetentionPolicy::default(),
        }
    }
}

/// Storage backend types
#[derive(Debug, Clone, PartialEq)]
pub enum StorageBackend {
    /// JSON file storage
    Json,
    /// CSV file storage
    Csv,
    /// SQLite database
    Sqlite,
    /// PostgreSQL database
    Postgres,
    /// In-memory storage (for testing)
    Memory,
}

/// Retention policy
#[derive(Debug, Clone)]
pub struct RetentionPolicy {
    /// Maximum age of data in days
    pub max_age_days: Option<u32>,
    
    /// Maximum number of records
    pub max_records: Option<usize>,
    
    /// Cleanup interval in days
    pub cleanup_interval_days: u32,
}

impl Default for RetentionPolicy {
    fn default() -> Self {
        Self {
            max_age_days: Some(365), // 1 year
            max_records: None,
            cleanup_interval_days: 30,
        }
    }
}

/// Main storage manager
pub struct StorageManager {
    config: StorageConfig,
    /// In-memory cache for testing and performance
    memory_cache: HashMap<String, Vec<UsageRecord>>,
}

impl StorageManager {
    /// Create a new storage manager
    pub fn new(config: StorageConfig) -> Result<Self> {
        // Ensure base directory exists
        if !config.base_dir.exists() {
            fs::create_dir_all(&config.base_dir).map_err(|e| {
                CcusageError::FileSystem(format!("Failed to create storage directory: {}", e))
            })?;
        }

        // Create backup directory if enabled
        if config.enable_backup {
            if let Some(backup_dir) = &config.backup_dir {
                if !backup_dir.exists() {
                    fs::create_dir_all(backup_dir).map_err(|e| {
                        CcusageError::FileSystem(format!("Failed to create backup directory: {}", e))
                    })?;
                }
            }
        }

        Ok(Self {
            config,
            memory_cache: HashMap::new(),
        })
    }

    /// Create storage manager with default configuration
    pub fn default() -> Result<Self> {
        let config = StorageConfig::default();
        Self::new(config)
    }

    /// Store usage records
    pub async fn store_usage_records(&mut self, records: &[UsageRecord]) -> Result<()> {
        if records.is_empty() {
            return Ok(());
        }

        // Group records by date for efficient storage
        let mut records_by_date: HashMap<NaiveDate, Vec<UsageRecord>> = HashMap::new();
        
        for record in records {
            let date = record.timestamp.date_naive();
            records_by_date.entry(date).or_insert_with(Vec::new).push(record.clone());
        }

        // Store each date's records
        for (date, day_records) in records_by_date {
            self.store_daily_records(date, &day_records).await?;
        }

        // Update memory cache
        if self.config.backend == StorageBackend::Memory {
            let cache_key = Utc::now().date_naive().to_string();
            self.memory_cache.insert(cache_key, records.to_vec());
        }

        Ok(())
    }

    /// Store daily records
    async fn store_daily_records(&self, date: NaiveDate, records: &[UsageRecord]) -> Result<()> {
        let file_path = self.get_daily_file_path(date);
        
        // Create backup if file exists
        if file_path.exists() && self.config.enable_backup {
            self.create_backup(&file_path).await?;
        }

        match self.config.backend {
            StorageBackend::Json => self.store_json_file(&file_path, records).await,
            StorageBackend::Csv => self.store_csv_file(&file_path, records).await,
            StorageBackend::Memory => Ok(()), // Already stored in memory cache
            StorageBackend::Sqlite => self.store_sqlite_records(date, records).await,
            StorageBackend::Postgres => self.store_postgres_records(date, records).await,
        }
    }

    /// Get daily file path
    fn get_daily_file_path(&self, date: NaiveDate) -> PathBuf {
        let filename = format!("usage_{}.json", date.format("%Y-%m-%d"));
        self.config.base_dir.join(filename)
    }

    /// Store records as JSON file
    async fn store_json_file(&self, file_path: &Path, records: &[UsageRecord]) -> Result<()> {
        let json_data = serde_json::to_string_pretty(records).map_err(|e| {
            CcusageError::DataLoading(format!("Failed to serialize records to JSON: {}", e))
        })?;

        fs::write(file_path, json_data).map_err(|e| {
            CcusageError::FileSystem(format!("Failed to write JSON file: {}", e))
        })?;

        Ok(())
    }

    /// Store records as CSV file
    async fn store_csv_file(&self, file_path: &Path, records: &[UsageRecord]) -> Result<()> {
        let mut writer = csv::Writer::from_path(file_path).map_err(|e| {
            CcusageError::DataLoading(format!("Failed to create CSV writer: {}", e))
        })?;

        // Write header
        writer.write_record(&[
            "id", "timestamp", "model", "input_tokens", "output_tokens", 
            "cost", "session_id", "user_id"
        ]).map_err(|e| {
            CcusageError::DataLoading(format!("Failed to write CSV header: {}", e))
        })?;

        // Write records
        for record in records {
            writer.write_record(&[
                &record.id,
                &record.timestamp.to_rfc3339(),
                &record.model,
                &record.input_tokens.to_string(),
                &record.output_tokens.to_string(),
                &record.cost.to_string(),
                &record.session_id.clone().unwrap_or_default(),
                &record.user_id.clone().unwrap_or_default(),
            ]).map_err(|e| {
                CcusageError::DataLoading(format!("Failed to write CSV record: {}", e))
            })?;
        }

        writer.flush().map_err(|e| {
            CcusageError::DataLoading(format!("Failed to flush CSV writer: {}", e))
        })?;

        Ok(())
    }

    /// Store records in SQLite database
    async fn store_sqlite_records(&self, date: NaiveDate, records: &[UsageRecord]) -> Result<()> {
        // This is a simplified implementation
        // In a real implementation, you would use rusqlite or similar
        Err(CcusageError::Application(
            "SQLite storage not yet implemented".to_string()
        ))
    }

    /// Store records in PostgreSQL database
    async fn store_postgres_records(&self, date: NaiveDate, records: &[UsageRecord]) -> Result<()> {
        // This is a simplified implementation
        // In a real implementation, you would use tokio-postgres or similar
        Err(CcusageError::Application(
            "PostgreSQL storage not yet implemented".to_string()
        ))
    }

    /// Load usage records for a date range
    pub async fn load_usage_records(&self, start_date: NaiveDate, end_date: NaiveDate) -> Result<Vec<UsageRecord>> {
        let mut all_records = Vec::new();
        
        let mut current_date = start_date;
        while current_date <= end_date {
            let records = self.load_daily_records(current_date).await?;
            all_records.extend(records);
            current_date += chrono::Duration::days(1);
        }

        Ok(all_records)
    }

    /// Load daily records
    async fn load_daily_records(&self, date: NaiveDate) -> Result<Vec<UsageRecord>> {
        // Check memory cache first
        if self.config.backend == StorageBackend::Memory {
            let cache_key = date.to_string();
            if let Some(records) = self.memory_cache.get(&cache_key) {
                return Ok(records.clone());
            }
        }

        let file_path = self.get_daily_file_path(date);
        
        if !file_path.exists() {
            return Ok(Vec::new());
        }

        match self.config.backend {
            StorageBackend::Json => self.load_json_file(&file_path).await,
            StorageBackend::Csv => self.load_csv_file(&file_path).await,
            StorageBackend::Memory => Ok(Vec::new()),
            StorageBackend::Sqlite => self.load_sqlite_records(date).await,
            StorageBackend::Postgres => self.load_postgres_records(date).await,
        }
    }

    /// Load records from JSON file
    async fn load_json_file(&self, file_path: &Path) -> Result<Vec<UsageRecord>> {
        let content = fs::read_to_string(file_path).map_err(|e| {
            CcusageError::FileSystem(format!("Failed to read JSON file: {}", e))
        })?;

        let records: Vec<UsageRecord> = serde_json::from_str(&content).map_err(CcusageError::Json)?;

        Ok(records)
    }

    /// Load records from CSV file
    async fn load_csv_file(&self, file_path: &Path) -> Result<Vec<UsageRecord>> {
        let file = fs::File::open(file_path).map_err(|e| {
            CcusageError::FileSystem(format!("Failed to open CSV file: {}", e))
        })?;

        let mut reader = csv::Reader::from_reader(file);
        let mut records = Vec::new();

        for result in reader.records() {
            let record = result.map_err(CcusageError::Csv)?;

            let usage_record = self.parse_csv_record(&record)?;
            records.push(usage_record);
        }

        Ok(records)
    }

    /// Load records from SQLite database
    async fn load_sqlite_records(&self, date: NaiveDate) -> Result<Vec<UsageRecord>> {
        // This is a simplified implementation
        Err(CcusageError::Application(
            "SQLite loading not yet implemented".to_string()
        ))
    }

    /// Load records from PostgreSQL database
    async fn load_postgres_records(&self, date: NaiveDate) -> Result<Vec<UsageRecord>> {
        // This is a simplified implementation
        Err(CcusageError::Application(
            "PostgreSQL loading not yet implemented".to_string()
        ))
    }

    /// Parse CSV record into UsageRecord
    fn parse_csv_record(&self, record: &csv::StringRecord) -> Result<UsageRecord> {
        let timestamp_str = record.get(1).ok_or_else(|| {
            CcusageError::Validation("Missing timestamp in CSV record".to_string())
        })?;
        
        let timestamp = utils::parse_date_flexible(timestamp_str)?;

        let model = record.get(2).ok_or_else(|| {
            CcusageError::Validation("Missing model in CSV record".to_string())
        })?.to_string();

        let input_tokens = record.get(3)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let output_tokens = record.get(4)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let cost = record.get(5)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);

        let session_id = record.get(6)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let user_id = record.get(7)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let mut usage_record = UsageRecord::new(timestamp, model, input_tokens, output_tokens, cost);
        usage_record.session_id = session_id;
        usage_record.user_id = user_id;

        Ok(usage_record)
    }

    /// Create backup of a file
    async fn create_backup(&self, file_path: &Path) -> Result<()> {
        let default_backup_dir = self.config.base_dir.join("backups");
        let backup_dir = self.config.backup_dir.as_ref()
            .unwrap_or(&default_backup_dir);
        
        if !backup_dir.exists() {
            fs::create_dir_all(backup_dir).map_err(|e| {
                CcusageError::FileSystem(format!("Failed to create backup directory: {}", e))
            })?;
        }

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = file_path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("backup");
        
        let backup_filename = format!("{}_{}.backup", filename, timestamp);
        let backup_path = backup_dir.join(backup_filename);

        fs::copy(file_path, &backup_path).map_err(|e| {
            CcusageError::FileSystem(format!("Failed to create backup: {}", e))
        })?;

        Ok(())
    }

    /// Delete records older than retention policy
    pub async fn cleanup_old_records(&self) -> Result<usize> {
        let mut deleted_count = 0;
        
        if let Some(max_age_days) = self.config.retention.max_age_days {
            let cutoff_date = Utc::now().date_naive() - chrono::Duration::days(max_age_days as i64);
            
            let entries = fs::read_dir(&self.config.base_dir).map_err(|e| {
                CcusageError::FileSystem(format!("Failed to read storage directory: {}", e))
            })?;

            for entry in entries {
                let entry = entry.map_err(|e| {
                    CcusageError::FileSystem(format!("Failed to read directory entry: {}", e))
                })?;
                
                let path = entry.path();
                
                if path.is_file() {
                    if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                        if file_name.starts_with("usage_") && file_name.ends_with(".json") {
                            if let Some(date_str) = file_name.strip_prefix("usage_").and_then(|s| s.strip_suffix(".json")) {
                                if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                                    if date < cutoff_date {
                                        fs::remove_file(&path).map_err(|e| {
                                            CcusageError::FileSystem(format!("Failed to delete file: {}", e))
                                        })?;
                                        deleted_count += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(deleted_count)
    }

    /// Get storage statistics
    pub async fn get_storage_stats(&self) -> Result<StorageStats> {
        let mut total_files = 0;
        let mut total_size = 0;
        let mut oldest_record = None;
        let mut newest_record = None;

        let entries = fs::read_dir(&self.config.base_dir).map_err(|e| {
            CcusageError::FileSystem(format!("Failed to read storage directory: {}", e))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                CcusageError::FileSystem(format!("Failed to read directory entry: {}", e))
            })?;
            
            let path = entry.path();
            
            if path.is_file() {
                let metadata = fs::metadata(&path).map_err(|e| {
                    CcusageError::FileSystem(format!("Failed to get file metadata: {}", e))
                })?;
                
                total_files += 1;
                total_size += metadata.len();
                
                // Extract date from filename
                if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                    if file_name.starts_with("usage_") && file_name.ends_with(".json") {
                        if let Some(date_str) = file_name.strip_prefix("usage_").and_then(|s| s.strip_suffix(".json")) {
                            if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                                if oldest_record.is_none() || date < oldest_record.unwrap() {
                                    oldest_record = Some(date);
                                }
                                if newest_record.is_none() || date > newest_record.unwrap() {
                                    newest_record = Some(date);
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(StorageStats {
            backend: self.config.backend.clone(),
            total_files,
            total_size,
            oldest_record,
            newest_record,
            base_dir: self.config.base_dir.clone(),
        })
    }

    /// Export data to different format
    pub async fn export_data(&self, format: ExportFormat, output_path: &Path, start_date: NaiveDate, end_date: NaiveDate) -> Result<()> {
        let records = self.load_usage_records(start_date, end_date).await?;
        
        match format {
            ExportFormat::Json => self.export_json(&records, output_path).await,
            ExportFormat::Csv => self.export_csv(&records, output_path).await,
            ExportFormat::Parquet => self.export_parquet(&records, output_path).await,
        }
    }

    /// Export data as JSON
    async fn export_json(&self, records: &[UsageRecord], output_path: &Path) -> Result<()> {
        let json_data = serde_json::to_string_pretty(records).map_err(|e| {
            CcusageError::DataLoading(format!("Failed to serialize records to JSON: {}", e))
        })?;

        fs::write(output_path, json_data).map_err(|e| {
            CcusageError::FileSystem(format!("Failed to write export file: {}", e))
        })?;

        Ok(())
    }

    /// Export data as CSV
    async fn export_csv(&self, records: &[UsageRecord], output_path: &Path) -> Result<()> {
        let mut writer = csv::Writer::from_path(output_path).map_err(|e| {
            CcusageError::DataLoading(format!("Failed to create CSV writer: {}", e))
        })?;

        // Write header
        writer.write_record(&[
            "id", "timestamp", "model", "input_tokens", "output_tokens", 
            "cost", "session_id", "user_id"
        ]).map_err(|e| {
            CcusageError::DataLoading(format!("Failed to write CSV header: {}", e))
        })?;

        // Write records
        for record in records {
            writer.write_record(&[
                &record.id,
                &record.timestamp.to_rfc3339(),
                &record.model,
                &record.input_tokens.to_string(),
                &record.output_tokens.to_string(),
                &record.cost.to_string(),
                &record.session_id.clone().unwrap_or_default(),
                &record.user_id.clone().unwrap_or_default(),
            ]).map_err(|e| {
                CcusageError::DataLoading(format!("Failed to write CSV record: {}", e))
            })?;
        }

        writer.flush().map_err(|e| {
            CcusageError::DataLoading(format!("Failed to flush CSV writer: {}", e))
        })?;

        Ok(())
    }

    /// Export data as Parquet
    async fn export_parquet(&self, _records: &[UsageRecord], _output_path: &Path) -> Result<()> {
        // This is a simplified implementation
        // In a real implementation, you would use parquet crate
        Err(CcusageError::Application(
            "Parquet export not yet implemented".to_string()
        ))
    }
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStats {
    pub backend: StorageBackend,
    pub total_files: usize,
    pub total_size: u64,
    pub oldest_record: Option<NaiveDate>,
    pub newest_record: Option<NaiveDate>,
    pub base_dir: PathBuf,
}

/// Export format
#[derive(Debug, Clone, PartialEq)]
pub enum ExportFormat {
    Json,
    Csv,
    Parquet,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use chrono::NaiveDateTime;

    #[tokio::test]
    async fn test_storage_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            base_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        
        let manager = StorageManager::new(config).unwrap();
        assert!(manager.config.base_dir.exists());
    }

    #[tokio::test]
    async fn test_store_and_load_json() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            backend: StorageBackend::Json,
            base_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        
        let mut manager = StorageManager::new(config).unwrap();
        
        let records = vec![
            UsageRecord::new(
                Utc::now(),
                "claude-3-sonnet".to_string(),
                1000,
                500,
                0.015,
            ),
        ];
        
        // Store records
        manager.store_usage_records(&records).await.unwrap();
        
        // Load records
        let date = Utc::now().date_naive();
        let loaded_records = manager.load_daily_records(date).await.unwrap();
        
        assert_eq!(loaded_records.len(), 1);
        assert_eq!(loaded_records[0].model, "claude-3-sonnet");
    }

    #[tokio::test]
    async fn test_storage_stats() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageConfig {
            backend: StorageBackend::Json,
            base_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        
        let manager = StorageManager::new(config).unwrap();
        let stats = manager.get_storage_stats().await.unwrap();
        
        assert_eq!(stats.backend, StorageBackend::Json);
        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.total_size, 0);
    }

    #[test]
    fn test_retention_policy() {
        let policy = RetentionPolicy::default();
        assert_eq!(policy.max_age_days, Some(365));
        assert_eq!(policy.cleanup_interval_days, 30);
    }
}