//! Data parser module for ccusage-rs
//! 
//! This module handles parsing various data formats and converting them
//! into internal data structures.

use crate::data::models::*;
use crate::error::{Result, CcusageError};
use crate::utils;
use chrono::{DateTime, Utc, NaiveDate};
use regex::Regex;
use serde_json::{Value, Map};
use std::collections::HashMap;

/// Parser for different data formats
pub struct DataParser;

impl DataParser {
    /// Parse JSON usage data
    pub fn parse_json_usage_data(json_data: &str) -> Result<Vec<UsageRecord>> {
        let records: Vec<UsageRecord> = serde_json::from_str(json_data).map_err(CcusageError::Json)?;

        // Validate records
        for record in &records {
            Self::validate_usage_record(record)?;
        }

        Ok(records)
    }

    /// Parse JSONL (JSON Lines) usage data
    pub fn parse_jsonl_usage_data(jsonl_data: &str) -> Result<Vec<UsageRecord>> {
        let mut records = Vec::new();
        
        for line in jsonl_data.lines() {
            if line.trim().is_empty() {
                continue;
            }
            
            let record: UsageRecord = serde_json::from_str(line).map_err(CcusageError::Json)?;
            
            Self::validate_usage_record(&record)?;
            records.push(record);
        }

        Ok(records)
    }

    /// Parse CSV usage data
    pub fn parse_csv_usage_data(csv_data: &str, has_headers: bool) -> Result<Vec<UsageRecord>> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(has_headers)
            .from_reader(csv_data.as_bytes());

        let mut records = Vec::new();
        
        for result in reader.records() {
            let record = result.map_err(|e| {
                CcusageError::Csv(format!("Failed to read CSV record: {}", e))
            })?;
            
            let usage_record = Self::parse_csv_record(&record)?;
            Self::validate_usage_record(&usage_record)?;
            records.push(usage_record);
        }

        Ok(records)
    }

    /// Parse a single CSV record
    fn parse_csv_record(record: &csv::StringRecord) -> Result<UsageRecord> {
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

        let session_id = record.get(5)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let user_id = record.get(6)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let mut usage_record = UsageRecord::new(timestamp, model, input_tokens, output_tokens, cost);
        usage_record.session_id = session_id;
        usage_record.user_id = user_id;

        // Parse additional metadata if present
        if let Some(metadata_str) = record.get(7) {
            if !metadata_str.is_empty() {
                if let Ok(metadata) = serde_json::from_str(metadata_str) {
                    usage_record.metadata = metadata;
                }
            }
        }

        Ok(usage_record)
    }

    /// Parse Claude Code export format
    pub fn parse_claude_export(data: &str) -> Result<Vec<UsageRecord>> {
        let mut records = Vec::new();
        
        for line in data.lines() {
            if line.trim().is_empty() {
                continue;
            }
            
            let export_record: ClaudeExportRecord = serde_json::from_str(line).map_err(CcusageError::Json)?;
            
            let usage_record = Self::convert_claude_export_to_usage(export_record)?;
            Self::validate_usage_record(&usage_record)?;
            records.push(usage_record);
        }

        Ok(records)
    }

    /// Convert Claude export record to UsageRecord
    fn convert_claude_export_to_usage(export_record: ClaudeExportRecord) -> Result<UsageRecord> {
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

    /// Parse log file data
    pub fn parse_log_file(log_data: &str, log_format: LogFormat) -> Result<Vec<UsageRecord>> {
        match log_format {
            LogFormat::ClaudeCode => Self::parse_claude_code_log(log_data),
            LogFormat::OpenAI => Self::parse_openai_log(log_data),
            LogFormat::Custom(pattern) => Self::parse_custom_log(log_data, &pattern),
        }
    }

    /// Parse Claude Code log format
    fn parse_claude_code_log(log_data: &str) -> Result<Vec<UsageRecord>> {
        let mut records = Vec::new();
        
        // Claude Code log pattern
        let pattern = r#"(?P<timestamp>\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z?)\s+\[.*?\]\s+Request:\s+(?P<model>\S+)\s+Input:\s+(?P<input_tokens>\d+)\s+Output:\s+(?P<output_tokens>\d+)\s+Cost:\s+\$(?P<cost>\d+\.\d+)"#;
        
        let re = Regex::new(pattern).map_err(|e| {
            CcusageError::Validation(format!("Invalid log pattern: {}", e))
        })?;

        for line in log_data.lines() {
            if let Some(caps) = re.captures(line) {
                let timestamp_str = &caps["timestamp"];
                let timestamp = utils::parse_date_flexible(timestamp_str)?;
                
                let model = caps["model"].to_string();
                let input_tokens = caps["input_tokens"].parse().unwrap_or(0);
                let output_tokens = caps["output_tokens"].parse().unwrap_or(0);
                let cost = caps["cost"].parse().unwrap_or(0.0);
                
                let record = UsageRecord::new(timestamp, model, input_tokens, output_tokens, cost);
                records.push(record);
            }
        }

        Ok(records)
    }

    /// Parse OpenAI API log format
    fn parse_openai_log(log_data: &str) -> Result<Vec<UsageRecord>> {
        let mut records = Vec::new();
        
        // OpenAI log pattern (simplified)
        let pattern = r#"(?P<timestamp>\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z?)\s+.*?model.*?(?P<model>gpt-\S+).*?usage.*?prompt_tokens.*?(?P<input_tokens>\d+).*?completion_tokens.*?(?P<output_tokens>\d+)"#;
        
        let re = Regex::new(pattern).map_err(|e| {
            CcusageError::Validation(format!("Invalid log pattern: {}", e))
        })?;

        for line in log_data.lines() {
            if let Some(caps) = re.captures(line) {
                let timestamp_str = &caps["timestamp"];
                let timestamp = utils::parse_date_flexible(timestamp_str)?;
                
                let model = caps["model"].to_string();
                let input_tokens = caps["input_tokens"].parse().unwrap_or(0);
                let output_tokens = caps["output_tokens"].parse().unwrap_or(0);
                
                // Estimate cost for OpenAI models
                let cost = Self::estimate_openai_cost(&model, input_tokens, output_tokens);
                
                let record = UsageRecord::new(timestamp, model, input_tokens, output_tokens, cost);
                records.push(record);
            }
        }

        Ok(records)
    }

    /// Parse custom log format
    fn parse_custom_log(log_data: &str, pattern: &str) -> Result<Vec<UsageRecord>> {
        let mut records = Vec::new();
        
        let re = Regex::new(pattern).map_err(|e| {
            CcusageError::Validation(format!("Invalid custom log pattern: {}", e))
        })?;

        for line in log_data.lines() {
            if let Some(caps) = re.captures(line) {
                let record = Self::parse_custom_log_record(&caps)?;
                records.push(record);
            }
        }

        Ok(records)
    }

    /// Parse custom log record from regex captures
    fn parse_custom_log_record(caps: &regex::Captures) -> Result<UsageRecord> {
        let timestamp_str = caps.name("timestamp")
            .ok_or_else(|| CcusageError::Validation("Missing timestamp in custom log".to_string()))?
            .as_str();
        
        let timestamp = utils::parse_date_flexible(timestamp_str)?;

        let model = caps.name("model")
            .ok_or_else(|| CcusageError::Validation("Missing model in custom log".to_string()))?
            .as_str()
            .to_string();

        let input_tokens = caps.name("input_tokens")
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0);

        let output_tokens = caps.name("output_tokens")
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or(0);

        let cost = caps.name("cost")
            .and_then(|m| m.as_str().parse().ok())
            .unwrap_or_else(|| Self::estimate_cost(&model, input_tokens, output_tokens));

        let mut record = UsageRecord::new(timestamp, model, input_tokens, output_tokens, cost);
        
        // Extract optional fields
        if let Some(session_id) = caps.name("session_id") {
            record.session_id = Some(session_id.as_str().to_string());
        }
        
        if let Some(user_id) = caps.name("user_id") {
            record.user_id = Some(user_id.as_str().to_string());
        }

        Ok(record)
    }

    /// Estimate cost for a model
    fn estimate_cost(model: &str, input_tokens: u32, output_tokens: u32) -> f64 {
        // Simplified cost estimation
        // In a real implementation, this would use actual pricing data
        if model.contains("claude-3-opus") {
            (input_tokens as f64 * 0.015 / 1000.0) + (output_tokens as f64 * 0.075 / 1000.0)
        } else if model.contains("claude-3-sonnet") {
            (input_tokens as f64 * 0.003 / 1000.0) + (output_tokens as f64 * 0.015 / 1000.0)
        } else if model.contains("claude-3-haiku") {
            (input_tokens as f64 * 0.00025 / 1000.0) + (output_tokens as f64 * 0.00125 / 1000.0)
        } else if model.contains("gpt-4") {
            (input_tokens as f64 * 0.03 / 1000.0) + (output_tokens as f64 * 0.06 / 1000.0)
        } else if model.contains("gpt-3.5") {
            (input_tokens as f64 * 0.0015 / 1000.0) + (output_tokens as f64 * 0.002 / 1000.0)
        } else {
            // Default cost estimation
            (input_tokens as f64 * 0.001 / 1000.0) + (output_tokens as f64 * 0.002 / 1000.0)
        }
    }

    /// Estimate OpenAI cost
    fn estimate_openai_cost(model: &str, input_tokens: u32, output_tokens: u32) -> f64 {
        if model.contains("gpt-4") {
            (input_tokens as f64 * 0.03 / 1000.0) + (output_tokens as f64 * 0.06 / 1000.0)
        } else if model.contains("gpt-3.5") {
            (input_tokens as f64 * 0.0015 / 1000.0) + (output_tokens as f64 * 0.002 / 1000.0)
        } else {
            Self::estimate_cost(model, input_tokens, output_tokens)
        }
    }

    /// Parse pricing data
    pub fn parse_pricing_data(json_data: &str) -> Result<Vec<PricingInfo>> {
        let pricing_data: Vec<PricingInfo> = serde_json::from_str(json_data).map_err(CcusageError::Json)?;

        // Validate pricing data
        for pricing in &pricing_data {
            Self::validate_pricing_info(pricing)?;
        }

        Ok(pricing_data)
    }

    /// Validate usage record
    fn validate_usage_record(record: &UsageRecord) -> Result<()> {
        if record.model.is_empty() {
            return Err(CcusageError::Validation("Model cannot be empty".to_string()));
        }

        if record.cost < 0.0 {
            return Err(CcusageError::Validation("Cost cannot be negative".to_string()));
        }

        if record.input_tokens == 0 && record.output_tokens == 0 {
            return Err(CcusageError::Validation("At least one token count must be positive".to_string()));
        }

        Ok(())
    }

    /// Validate pricing info
    fn validate_pricing_info(pricing: &PricingInfo) -> Result<()> {
        if pricing.model.is_empty() {
            return Err(CcusageError::Validation("Model cannot be empty in pricing info".to_string()));
        }

        if pricing.input_cost_per_1k < 0.0 {
            return Err(CcusageError::Validation("Input cost cannot be negative".to_string()));
        }

        if pricing.output_cost_per_1k < 0.0 {
            return Err(CcusageError::Validation("Output cost cannot be negative".to_string()));
        }

        if pricing.currency.is_empty() {
            return Err(CcusageError::Validation("Currency cannot be empty".to_string()));
        }

        Ok(())
    }

    /// Extract metadata from JSON
    pub fn extract_metadata(json_data: &str) -> Result<HashMap<String, Value>> {
        let value: Value = serde_json::from_str(json_data).map_err(CcusageError::Json)?;

        let metadata = value.as_object()
            .ok_or_else(|| CcusageError::Validation("Metadata must be a JSON object".to_string()))?
            .clone();

        Ok(metadata)
    }

    /// Normalize model names
    pub fn normalize_model_name(model: &str) -> String {
        model.to_lowercase()
            .replace(" ", "-")
            .replace("_", "-")
    }

    /// Parse date range from string
    pub fn parse_date_range(range_str: &str) -> Result<(DateTime<Utc>, DateTime<Utc>)> {
        let parts: Vec<&str> = range_str.split("..").collect();
        
        if parts.len() != 2 {
            return Err(CcusageError::Validation(
                "Date range must be in format 'start..end'".to_string()
            ));
        }

        let start = utils::parse_date_flexible(parts[0])?;
        let end = utils::parse_date_flexible(parts[1])?;

        if start > end {
            return Err(CcusageError::Validation(
                "Start date must be before end date".to_string()
            ));
        }

        Ok((start, end))
    }

    /// Parse duration from string
    pub fn parse_duration(duration_str: &str) -> Result<chrono::Duration> {
        utils::parse_duration(duration_str)
    }
}

/// Log format types
#[derive(Debug, Clone, PartialEq)]
pub enum LogFormat {
    /// Claude Code log format
    ClaudeCode,
    /// OpenAI API log format
    OpenAI,
    /// Custom log format with regex pattern
    Custom(String),
}

/// Claude export record format
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
    use chrono::NaiveDateTime;

    #[test]
    fn test_parse_json_usage_data() {
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
            }
        ]
        "#;
        
        let result = DataParser::parse_json_usage_data(json_data);
        assert!(result.is_ok());
        
        let records = result.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].model, "claude-3-sonnet");
    }

    #[test]
    fn test_parse_csv_usage_data() {
        let csv_data = "2023-12-25T10:00:00Z,claude-3-sonnet,1000,500,0.015,session1,user1\n";
        
        let result = DataParser::parse_csv_usage_data(csv_data, false);
        assert!(result.is_ok());
        
        let records = result.unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].model, "claude-3-sonnet");
    }

    #[test]
    fn test_parse_jsonl_usage_data() {
        let jsonl_data = r#"
        {"id": "test1", "timestamp": "2023-12-25T10:00:00Z", "model": "claude-3-sonnet", "input_tokens": 1000, "output_tokens": 500, "cost": 0.015}
        {"id": "test2", "timestamp": "2023-12-25T11:00:00Z", "model": "claude-3-opus", "input_tokens": 2000, "output_tokens": 1000, "cost": 0.045}
        "#;
        
        let result = DataParser::parse_jsonl_usage_data(jsonl_data);
        assert!(result.is_ok());
        
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_estimate_cost() {
        let cost = DataParser::estimate_cost("claude-3-sonnet", 1000, 500);
        assert!(cost > 0.0);
        
        let cost = DataParser::estimate_cost("gpt-4", 1000, 500);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_normalize_model_name() {
        assert_eq!(DataParser::normalize_model_name("Claude 3 Sonnet"), "claude-3-sonnet");
        assert_eq!(DataParser::normalize_model_name("GPT_4"), "gpt-4");
    }

    #[test]
    fn test_parse_date_range() {
        let result = DataParser::parse_date_range("2023-12-25..2023-12-26");
        assert!(result.is_ok());
        
        let (start, end) = result.unwrap();
        assert!(start < end);
    }

    #[test]
    fn test_parse_claude_code_log() {
        let log_data = r#"
        2023-12-25T10:00:00Z [INFO] Request: claude-3-sonnet Input: 1000 Output: 500 Cost: $0.015
        2023-12-25T11:00:00Z [INFO] Request: claude-3-opus Input: 2000 Output: 1000 Cost: $0.045
        "#;
        
        let result = DataParser::parse_log_file(log_data, LogFormat::ClaudeCode);
        assert!(result.is_ok());
        
        let records = result.unwrap();
        assert_eq!(records.len(), 2);
    }

    #[test]
    fn test_validate_usage_record() {
        let mut record = UsageRecord::new(
            Utc::now(),
            "claude-3-sonnet".to_string(),
            1000,
            500,
            0.015,
        );
        
        // Valid record
        assert!(DataParser::validate_usage_record(&record).is_ok());
        
        // Invalid record - empty model
        record.model = String::new();
        assert!(DataParser::validate_usage_record(&record).is_err());
        
        // Invalid record - negative cost
        record.model = "claude-3-sonnet".to_string();
        record.cost = -0.015;
        assert!(DataParser::validate_usage_record(&record).is_err());
    }
}