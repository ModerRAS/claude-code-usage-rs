//! Data loader for ccusage-rs
//! 
//! This module handles loading usage data from various sources including
//! JSON files, CSV files, databases, and APIs.

use crate::data::models::*;
use crate::error::{Result, CcusageError};
use crate::utils;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use clap::ValueEnum;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::io::BufReader;
use csv::{ReaderBuilder, StringRecord};
use tokio::sync::RwLock;

/// Data loader configuration
#[derive(Debug, Clone)]
pub struct DataLoaderConfig {
    /// Data source type
    pub source_type: DataSourceType,
    
    /// Source path or URL
    pub source_path: String,
    
    /// Cache directory
    pub cache_dir: Option<PathBuf>,
    
    /// Enable caching
    pub enable_caching: bool,
    
    /// Cache expiration in seconds
    pub cache_expiration: u64,
    
    /// Maximum number of records to load
    pub max_records: Option<usize>,
    
    /// Date range filter
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    
    /// Model filter
    pub model_filter: Option<Vec<String>>,
}

impl Default for DataLoaderConfig {
    fn default() -> Self {
        Self {
            source_type: DataSourceType::Json,
            source_path: String::new(),
            cache_dir: None,
            enable_caching: true,
            cache_expiration: 3600, // 1 hour
            max_records: None,
            date_range: None,
            model_filter: None,
        }
    }
}

/// Data source types
#[derive(Debug, Clone, PartialEq, clap::ValueEnum)]
pub enum DataSourceType {
    /// JSON file format
    Json,
    /// CSV file format
    Csv,
    /// SQLite database
    Sqlite,
    /// REST API
    Api,
    /// PostgreSQL database
    Postgres,
    /// Claude Code export format
    ClaudeExport,
}

impl std::fmt::Display for DataSourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataSourceType::Json => write!(f, "json"),
            DataSourceType::Csv => write!(f, "csv"),
            DataSourceType::Sqlite => write!(f, "sqlite"),
            DataSourceType::Api => write!(f, "api"),
            DataSourceType::Postgres => write!(f, "postgres"),
            DataSourceType::ClaudeExport => write!(f, "claude-export"),
        }
    }
}

/// Main data loader
pub struct DataLoader {
    config: DataLoaderConfig,
    cache: RwLock<HashMap<String, CacheEntry>>,
}

impl DataLoader {
    /// Create a new data loader
    pub fn new(config: DataLoaderConfig) -> Self {
        Self {
            config,
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// Create a data loader with default configuration
    pub fn with_source(source_type: DataSourceType, source_path: String) -> Self {
        let config = DataLoaderConfig {
            source_type,
            source_path,
            ..Default::default()
        };
        Self::new(config)
    }

    /// Load usage data from the configured source
    pub async fn load_usage_data(&self) -> Result<Vec<UsageRecord>> {
        let cache_key = self.generate_cache_key();
        
        // Check cache first
        if self.config.enable_caching {
            if let Some(cached_data) = self.check_cache(&cache_key).await {
                return Ok(cached_data);
            }
        }

        // Load data from source
        let data = match self.config.source_type {
            DataSourceType::Json => self.load_from_json().await,
            DataSourceType::Csv => self.load_from_csv().await,
            DataSourceType::ClaudeExport => self.load_from_claude_export().await,
            DataSourceType::Sqlite => self.load_from_sqlite().await,
            DataSourceType::Postgres => self.load_from_postgres().await,
            DataSourceType::Api => self.load_from_api().await,
        }?;

        // Apply filters
        let filtered_data = self.apply_filters(data)?;

        // Cache the result
        if self.config.enable_caching {
            self.cache_data(&cache_key, filtered_data.clone()).await;
        }

        Ok(filtered_data)
    }

    /// Load data from JSON file
    async fn load_from_json(&self) -> Result<Vec<UsageRecord>> {
        let path = Path::new(&self.config.source_path);
        
        if !path.exists() {
            return Err(CcusageError::FileSystem(format!(
                "JSON file not found: {}", path.display()
            )));
        }

        let content = fs::read_to_string(path).map_err(|e| {
            CcusageError::FileSystem(format!("Failed to read JSON file: {}", e))
        })?;

        let records: Vec<UsageRecord> = serde_json::from_str(&content).map_err(|e| {
            CcusageError::Json(e)
        })?;

        Ok(records)
    }

    /// Load data from CSV file
    async fn load_from_csv(&self) -> Result<Vec<UsageRecord>> {
        let path = Path::new(&self.config.source_path);
        
        if !path.exists() {
            return Err(CcusageError::FileSystem(format!(
                "CSV file not found: {}", path.display()
            )));
        }

        let file = fs::File::open(path).map_err(|e| {
            CcusageError::FileSystem(format!("Failed to open CSV file: {}", e))
        })?;

        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);

        let mut records = Vec::new();
        
        for result in reader.records() {
            let record = result.map_err(|e| {
                CcusageError::DataLoading(format!("Failed to read CSV record: {}", e))
            })?;
            
            let usage_record = self.parse_csv_record(&record)?;
            records.push(usage_record);
        }

        Ok(records)
    }

    /// Load data from Claude Code export format
    async fn load_from_claude_export(&self) -> Result<Vec<UsageRecord>> {
        let path = Path::new(&self.config.source_path);
        
        if !path.exists() {
            return Err(CcusageError::FileSystem(format!(
                "Claude export file not found: {}", path.display()
            )));
        }

        let content = fs::read_to_string(path).map_err(|e| {
            CcusageError::FileSystem(format!("Failed to read Claude export: {}", e))
        })?;

        // Parse Claude export format (JSONL format with specific structure)
        let mut records = Vec::new();
        
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            
            let export_record: ClaudeExportRecord = serde_json::from_str(line).map_err(|e| {
                CcusageError::DataLoading(format!("Failed to parse Claude export line: {}", e))
            })?;
            
            let usage_record = self.convert_claude_export_to_usage(export_record)?;
            records.push(usage_record);
        }

        Ok(records)
    }

    /// Load data from SQLite database
    async fn load_from_sqlite(&self) -> Result<Vec<UsageRecord>> {
        // This is a simplified implementation
        // In a real implementation, you would use rusqlite or similar
        Err(CcusageError::Application(
            "SQLite loading not yet implemented".to_string()
        ))
    }

    /// Load data from PostgreSQL database
    async fn load_from_postgres(&self) -> Result<Vec<UsageRecord>> {
        // This is a simplified implementation
        // In a real implementation, you would use tokio-postgres or similar
        Err(CcusageError::Application(
            "PostgreSQL loading not yet implemented".to_string()
        ))
    }

    /// Load data from API
    async fn load_from_api(&self) -> Result<Vec<UsageRecord>> {
        use reqwest::Client;
        let client = Client::new();
        
        let response = client.get(&self.config.source_path)
            .send()
            .await
            .map_err(|e| CcusageError::Network(format!("Failed to fetch data from API: {}", e)))?;

        if !response.status().is_success() {
            return Err(CcusageError::Network(format!(
                "API request failed with status: {}", response.status()
            )));
        }

        let content = response.text().await.map_err(|e| {
            CcusageError::Network(format!("Failed to read API response: {}", e))
        })?;

        let records: Vec<UsageRecord> = serde_json::from_str(&content).map_err(|e| {
            CcusageError::DataLoading(format!("Failed to parse API response: {}", e))
        })?;

        Ok(records)
    }

    /// Parse CSV record into UsageRecord
    fn parse_csv_record(&self, record: &StringRecord) -> Result<UsageRecord> {
        let timestamp_str = record.get(0).ok_or_else(|| {
            CcusageError::Validation("Missing timestamp in CSV record".to_string())
        })?;
        
        let timestamp = utils::parse_date_flexible(timestamp_str)?;

        let model = record.get(1).ok_or_else(|| {
            CcusageError::Validation("Missing model in CSV record".to_string())
        })?.to_string();

        let input_tokens = record.get(2)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let output_tokens = record.get(3)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        let cost = record.get(4)
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);

        let session_id = record.get(5).map(|s| s.to_string());
        let user_id = record.get(6).map(|s| s.to_string());

        let mut usage_record = UsageRecord::new(timestamp, model, input_tokens, output_tokens, cost);
        usage_record.session_id = session_id;
        usage_record.user_id = user_id;

        Ok(usage_record)
    }

    /// Convert Claude export record to UsageRecord
    fn convert_claude_export_to_usage(&self, export_record: ClaudeExportRecord) -> Result<UsageRecord> {
        let timestamp = utils::parse_date_flexible(&export_record.timestamp)?;
        
        let mut usage_record = UsageRecord::new(
            timestamp,
            export_record.model,
            export_record.input_tokens,
            export_record.output_tokens,
            export_record.cost,
        );
        
        usage_record.session_id = export_record.session_id;
        usage_record.user_id = export_record.user_id;
        
        // Convert metadata
        if let Some(metadata) = export_record.metadata {
            usage_record.metadata = metadata;
        }
        
        Ok(usage_record)
    }

    /// Apply filters to loaded data
    fn apply_filters(&self, data: Vec<UsageRecord>) -> Result<Vec<UsageRecord>> {
        let mut filtered = data;

        // Apply date range filter
        if let Some((start, end)) = &self.config.date_range {
            filtered = filtered
                .into_iter()
                .filter(|record| record.is_within_range(*start, *end))
                .collect();
        }

        // Apply model filter
        if let Some(models) = &self.config.model_filter {
            filtered = filtered
                .into_iter()
                .filter(|record| models.contains(&record.model))
                .collect();
        }

        // Apply max records limit
        if let Some(max_records) = self.config.max_records {
            if filtered.len() > max_records {
                filtered.truncate(max_records);
            }
        }

        Ok(filtered)
    }

    /// Generate cache key
    fn generate_cache_key(&self) -> String {
        let mut key = format!("{}:{}", self.config.source_type, self.config.source_path);
        
        if let Some((start, end)) = &self.config.date_range {
            key.push_str(&format!(":{}:{}", start.timestamp(), end.timestamp()));
        }
        
        if let Some(models) = &self.config.model_filter {
            key.push_str(&format!(":models={}", models.join(",")));
        }
        
        key
    }

    /// Check cache for data
    async fn check_cache(&self, cache_key: &str) -> Option<Vec<UsageRecord>> {
        let cache = self.cache.read().await;
        
        if let Some(entry) = cache.get(cache_key) {
            let now = Utc::now();
            let age = (now - entry.timestamp).num_seconds();
            
            if age <= self.config.cache_expiration as i64 {
                return Some(entry.data.clone());
            }
        }
        
        None
    }

    /// Cache data
    async fn cache_data(&self, cache_key: &str, data: Vec<UsageRecord>) {
        let mut cache = self.cache.write().await;
        
        let entry = CacheEntry {
            data: data.clone(),
            timestamp: Utc::now(),
        };
        
        cache.insert(cache_key.to_string(), entry);
    }

    /// Clear cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let now = Utc::now();
        
        let mut valid_entries = 0;
        let mut expired_entries = 0;
        
        for entry in cache.values() {
            let age = (now - entry.timestamp).num_seconds();
            if age <= self.config.cache_expiration as i64 {
                valid_entries += 1;
            } else {
                expired_entries += 1;
            }
        }
        
        CacheStats {
            total_entries: cache.len(),
            valid_entries,
            expired_entries,
            cache_expiration: self.config.cache_expiration,
        }
    }

    /// Load pricing data
    pub async fn load_pricing_data(&self, pricing_path: &str) -> Result<Vec<PricingInfo>> {
        let path = Path::new(pricing_path);
        
        if !path.exists() {
            return Err(CcusageError::FileSystem(format!(
                "Pricing file not found: {}", path.display()
            )));
        }

        let content = fs::read_to_string(path).map_err(|e| {
            CcusageError::FileSystem(format!("Failed to read pricing file: {}", e))
        })?;

        let pricing_data: Vec<PricingInfo> = serde_json::from_str(&content).map_err(|e| {
            CcusageError::DataLoading(format!("Failed to parse pricing data: {}", e))
        })?;

        Ok(pricing_data)
    }

    /// Load sessions from usage records
    pub fn load_sessions(&self, records: &[UsageRecord]) -> Vec<Session> {
        let mut sessions: HashMap<String, Session> = HashMap::new();
        
        for record in records {
            if let Some(session_id) = &record.session_id {
                let session = sessions.entry(session_id.clone())
                    .or_insert_with(|| Session::new(
                        session_id.clone(),
                        record.timestamp,
                        record.user_id.clone(),
                    ));
                
                session.add_record(record);
            }
        }
        
        // Calculate durations
        for session in sessions.values_mut() {
            session.calculate_duration();
        }
        
        sessions.into_values().collect()
    }
}

/// Cache entry
#[derive(Debug, Clone)]
struct CacheEntry {
    data: Vec<UsageRecord>,
    timestamp: DateTime<Utc>,
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub valid_entries: usize,
    pub expired_entries: usize,
    pub cache_expiration: u64,
}

/// Claude export record format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClaudeExportRecord {
    pub timestamp: String,
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cost: f64,
    pub session_id: Option<String>,
    pub user_id: Option<String>,
    pub metadata: Option<HashMap<String, Value>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_load_from_json() {
        // Create temporary JSON file
        let mut temp_file = NamedTempFile::new().unwrap();
        let json_data = r#"
        [
            {
                "id": "test1",
                "timestamp": "2023-12-25T10:00:00Z",
                "model": "claude-3-sonnet",
                "input_tokens": 1000,
                "output_tokens": 500,
                "cost": 0.015,
                "session_id": "session1",
                "user_id": "user1",
                "metadata": {}
            },
            {
                "id": "test2",
                "timestamp": "2023-12-25T11:00:00Z",
                "model": "claude-3-opus",
                "input_tokens": 2000,
                "output_tokens": 1000,
                "cost": 0.045,
                "session_id": "session1",
                "user_id": "user1",
                "metadata": {}
            }
        ]
        "#;
        
        temp_file.write_all(json_data.as_bytes()).unwrap();
        let temp_path = temp_file.path().to_string_lossy().to_string();

        let loader = DataLoader::with_source(DataSourceType::Json, temp_path);
        let records = loader.load_usage_data().await.unwrap();
        
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].model, "claude-3-sonnet");
        assert_eq!(records[1].model, "claude-3-opus");
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        let config = DataLoaderConfig {
            source_type: DataSourceType::Json,
            source_path: "nonexistent.json".to_string(),
            enable_caching: true,
            cache_expiration: 3600,
            ..Default::default()
        };
        
        let loader = DataLoader::new(config);
        
        // Initially cache should be empty
        let stats = loader.cache_stats().await;
        assert_eq!(stats.total_entries, 0);
        
        // Test cache key generation
        let key = loader.generate_cache_key();
        assert!(!key.is_empty());
    }

    #[test]
    fn test_parse_csv_record() {
        let config = DataLoaderConfig::default();
        let loader = DataLoader::new(config);
        
        let mut record = StringRecord::new();
        record.push_field("2023-12-25T10:00:00Z");
        record.push_field("claude-3-sonnet");
        record.push_field("1000");
        record.push_field("500");
        record.push_field("0.015");
        record.push_field("session1");
        record.push_field("user1");
        
        let usage_record = loader.parse_csv_record(&record).unwrap();
        assert_eq!(usage_record.model, "claude-3-sonnet");
        assert_eq!(usage_record.input_tokens, 1000);
        assert_eq!(usage_record.output_tokens, 500);
        assert_eq!(usage_record.cost, 0.015);
    }

    #[test]
    fn test_apply_filters() {
        let config = DataLoaderConfig {
            max_records: Some(1),
            ..Default::default()
        };
        
        let loader = DataLoader::new(config);
        
        let records = vec![
            UsageRecord::new(
                Utc::now(),
                "claude-3-sonnet".to_string(),
                1000,
                500,
                0.015,
            ),
            UsageRecord::new(
                Utc::now(),
                "claude-3-opus".to_string(),
                2000,
                1000,
                0.045,
            ),
        ];
        
        let filtered = loader.apply_filters(records).unwrap();
        assert_eq!(filtered.len(), 1);
    }
}