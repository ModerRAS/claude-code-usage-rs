//! Core data models for ccusage-rs
//! 
//! This module defines the main data structures used throughout the application
//! for representing usage data, costs, sessions, and reports.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, NaiveDate, Timelike};
use std::collections::HashMap;
use std::path::PathBuf;

/// Main usage data record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    /// Unique identifier for the record
    pub id: String,
    
    /// Timestamp of the usage
    pub timestamp: DateTime<Utc>,
    
    /// Model used (e.g., "claude-3-sonnet-20240229")
    pub model: String,
    
    /// Number of tokens input
    pub input_tokens: u32,
    
    /// Number of tokens output
    pub output_tokens: u32,
    
    /// Calculated cost for this record
    pub cost: f64,
    
    /// Session identifier
    pub session_id: Option<String>,
    
    /// User identifier
    pub user_id: Option<String>,
    
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl UsageRecord {
    /// Create a new usage record
    pub fn new(
        timestamp: DateTime<Utc>,
        model: String,
        input_tokens: u32,
        output_tokens: u32,
        cost: f64,
    ) -> Self {
        Self {
            id: generate_record_id(&timestamp, &model),
            timestamp,
            model,
            input_tokens,
            output_tokens,
            cost,
            session_id: None,
            user_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Calculate total tokens used
    pub fn total_tokens(&self) -> u32 {
        self.input_tokens + self.output_tokens
    }

    /// Calculate cost per token
    pub fn cost_per_token(&self) -> f64 {
        let total = self.total_tokens();
        if total == 0 {
            0.0
        } else {
            self.cost / total as f64
        }
    }

    /// Check if this record belongs to a specific date
    pub fn is_on_date(&self, date: NaiveDate) -> bool {
        self.timestamp.date_naive() == date
    }

    /// Check if this record is within a date range
    pub fn is_within_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> bool {
        self.timestamp >= start && self.timestamp <= end
    }
}

/// Session data grouping multiple usage records
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier
    pub id: String,
    
    /// Session start time
    pub start_time: DateTime<Utc>,
    
    /// Session end time
    pub end_time: Option<DateTime<Utc>>,
    
    /// User identifier
    pub user_id: Option<String>,
    
    /// Total cost for the session
    pub total_cost: f64,
    
    /// Total input tokens
    pub total_input_tokens: u32,
    
    /// Total output tokens
    pub total_output_tokens: u32,
    
    /// Number of requests in the session
    pub request_count: u32,
    
    /// Session duration in seconds
    pub duration_seconds: Option<u64>,
    
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Session {
    /// Create a new session
    pub fn new(
        id: String,
        start_time: DateTime<Utc>,
        user_id: Option<String>,
    ) -> Self {
        Self {
            id,
            start_time,
            end_time: None,
            user_id,
            total_cost: 0.0,
            total_input_tokens: 0,
            total_output_tokens: 0,
            request_count: 0,
            duration_seconds: None,
            metadata: HashMap::new(),
        }
    }

    /// Add a usage record to this session
    pub fn add_record(&mut self, record: &UsageRecord) {
        self.total_cost += record.cost;
        self.total_input_tokens += record.input_tokens;
        self.total_output_tokens += record.output_tokens;
        self.request_count += 1;
        
        // Update end time if this record is later
        if let Some(end_time) = self.end_time {
            if record.timestamp > end_time {
                self.end_time = Some(record.timestamp);
            }
        } else {
            self.end_time = Some(record.timestamp);
        }
    }

    /// Calculate session duration
    pub fn calculate_duration(&mut self) {
        if let Some(end_time) = self.end_time {
            self.duration_seconds = Some(
                (end_time - self.start_time).num_seconds() as u64
            );
        }
    }

    /// Calculate average cost per request
    pub fn avg_cost_per_request(&self) -> f64 {
        if self.request_count == 0 {
            0.0
        } else {
            self.total_cost / self.request_count as f64
        }
    }

    /// Calculate average tokens per request
    pub fn avg_tokens_per_request(&self) -> f64 {
        if self.request_count == 0 {
            0.0
        } else {
            (self.total_input_tokens + self.total_output_tokens) as f64 / self.request_count as f64
        }
    }
}

/// Pricing information for models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingInfo {
    /// Model identifier
    pub model: String,
    
    /// Cost per 1K input tokens
    pub input_cost_per_1k: f64,
    
    /// Cost per 1K output tokens
    pub output_cost_per_1k: f64,
    
    /// Currency code
    pub currency: String,
    
    /// Effective date for this pricing
    pub effective_date: DateTime<Utc>,
    
    /// Whether this pricing is currently active
    pub is_active: bool,
}

impl PricingInfo {
    /// Calculate cost for a usage record
    pub fn calculate_cost(&self, input_tokens: u32, output_tokens: u32) -> f64 {
        let input_cost = (input_tokens as f64 / 1000.0) * self.input_cost_per_1k;
        let output_cost = (output_tokens as f64 / 1000.0) * self.output_cost_per_1k;
        input_cost + output_cost
    }

    /// Check if this pricing is valid for a given date
    pub fn is_valid_for_date(&self, date: DateTime<Utc>) -> bool {
        self.effective_date <= date && self.is_active
    }
}

/// Daily usage summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailySummary {
    /// Date of the summary
    pub date: NaiveDate,
    
    /// Total cost for the day
    pub total_cost: f64,
    
    /// Total input tokens
    pub total_input_tokens: u32,
    
    /// Total output tokens
    pub total_output_tokens: u32,
    
    /// Number of requests
    pub request_count: u32,
    
    /// Number of unique sessions
    pub session_count: u32,
    
    /// Most used model
    pub most_used_model: Option<String>,
    
    /// Peak usage hour (0-23)
    pub peak_hour: Option<u8>,
    
    /// Average cost per request
    pub avg_cost_per_request: f64,
    
    /// Breakdown by model
    pub model_breakdown: HashMap<String, ModelUsage>,
}

impl DailySummary {
    /// Create a new daily summary
    pub fn new(date: NaiveDate) -> Self {
        Self {
            date,
            total_cost: 0.0,
            total_input_tokens: 0,
            total_output_tokens: 0,
            request_count: 0,
            session_count: 0,
            most_used_model: None,
            peak_hour: None,
            avg_cost_per_request: 0.0,
            model_breakdown: HashMap::new(),
        }
    }

    /// Add a usage record to this summary
    pub fn add_record(&mut self, record: &UsageRecord) {
        self.total_cost += record.cost;
        self.total_input_tokens += record.input_tokens;
        self.total_output_tokens += record.output_tokens;
        self.request_count += 1;

        // Update model breakdown
        let model_usage = self.model_breakdown
            .entry(record.model.clone())
            .or_insert_with(|| ModelUsage::new(record.model.clone()));
        model_usage.add_record(record);

        // Update most used model
        self.most_used_model = self.model_breakdown
            .iter()
            .max_by_key(|(_, usage)| usage.request_count)
            .map(|(model, _)| model.clone());
    }

    /// Calculate average cost per request
    pub fn calculate_avg_cost(&mut self) {
        if self.request_count > 0 {
            self.avg_cost_per_request = self.total_cost / self.request_count as f64;
        }
    }

    /// Calculate peak usage hour
    pub fn calculate_peak_hour(&self, records: &[UsageRecord]) {
        let mut hour_counts = [0u32; 24];
        
        for record in records {
            if record.is_on_date(self.date) {
                let hour = record.timestamp.hour() as usize;
                hour_counts[hour] += 1;
            }
        }
        
        self.peak_hour = hour_counts
            .iter()
            .enumerate()
            .max_by_key(|(_, count)| *count)
            .map(|(hour, _)| hour as u8);
    }
}

/// Usage breakdown by model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsage {
    /// Model identifier
    pub model: String,
    
    /// Total cost for this model
    pub total_cost: f64,
    
    /// Total input tokens
    pub total_input_tokens: u32,
    
    /// Total output tokens
    pub total_output_tokens: u32,
    
    /// Number of requests
    pub request_count: u32,
    
    /// Average cost per request
    pub avg_cost_per_request: f64,
}

impl ModelUsage {
    /// Create new model usage
    pub fn new(model: String) -> Self {
        Self {
            model,
            total_cost: 0.0,
            total_input_tokens: 0,
            total_output_tokens: 0,
            request_count: 0,
            avg_cost_per_request: 0.0,
        }
    }

    /// Add a usage record to this model usage
    pub fn add_record(&mut self, record: &UsageRecord) {
        self.total_cost += record.cost;
        self.total_input_tokens += record.input_tokens;
        self.total_output_tokens += record.output_tokens;
        self.request_count += 1;
    }

    /// Calculate average cost per request
    pub fn calculate_avg_cost(&mut self) {
        if self.request_count > 0 {
            self.avg_cost_per_request = self.total_cost / self.request_count as f64;
        }
    }
}

/// Weekly usage summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklySummary {
    /// Week start date (Monday)
    pub week_start: NaiveDate,
    
    /// Week end date (Sunday)
    pub week_end: NaiveDate,
    
    /// Total cost for the week
    pub total_cost: f64,
    
    /// Total input tokens
    pub total_input_tokens: u32,
    
    /// Total output tokens
    pub total_output_tokens: u32,
    
    /// Number of requests
    pub request_count: u32,
    
    /// Number of unique sessions
    pub session_count: u32,
    
    /// Daily breakdown
    pub daily_breakdown: Vec<DailySummary>,
    
    /// Average daily cost
    pub avg_daily_cost: f64,
    
    /// Most expensive day
    pub most_expensive_day: Option<NaiveDate>,
}

impl WeeklySummary {
    /// Create a new weekly summary
    pub fn new(week_start: NaiveDate) -> Self {
        let week_end = week_start + chrono::Duration::days(6);
        Self {
            week_start,
            week_end,
            total_cost: 0.0,
            total_input_tokens: 0,
            total_output_tokens: 0,
            request_count: 0,
            session_count: 0,
            daily_breakdown: Vec::new(),
            avg_daily_cost: 0.0,
            most_expensive_day: None,
        }
    }

    /// Add a daily summary to this weekly summary
    pub fn add_daily_summary(&mut self, daily: DailySummary) {
        self.total_cost += daily.total_cost;
        self.total_input_tokens += daily.total_input_tokens;
        self.total_output_tokens += daily.total_output_tokens;
        self.request_count += daily.request_count;
        self.session_count += daily.session_count;
        self.daily_breakdown.push(daily);
    }

    /// Calculate average daily cost
    pub fn calculate_avg_daily_cost(&mut self) {
        if !self.daily_breakdown.is_empty() {
            self.avg_daily_cost = self.total_cost / self.daily_breakdown.len() as f64;
            
            // Find most expensive day
            self.most_expensive_day = self.daily_breakdown
                .iter()
                .max_by_key(|day| day.total_cost as u64)
                .map(|day| day.date);
        }
    }
}

/// Monthly usage summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlySummary {
    /// Year and month
    pub year: u32,
    pub month: u32,
    
    /// Total cost for the month
    pub total_cost: f64,
    
    /// Total input tokens
    pub total_input_tokens: u32,
    
    /// Total output tokens
    pub total_output_tokens: u32,
    
    /// Number of requests
    pub request_count: u32,
    
    /// Number of unique sessions
    pub session_count: u32,
    
    /// Weekly breakdown
    pub weekly_breakdown: Vec<WeeklySummary>,
    
    /// Average weekly cost
    pub avg_weekly_cost: f64,
    
    /// Most expensive week
    pub most_expensive_week: Option<NaiveDate>,
    
    /// Budget information
    pub budget: Option<BudgetInfo>,
}

impl MonthlySummary {
    /// Create a new monthly summary
    pub fn new(year: u32, month: u32) -> Self {
        Self {
            year,
            month,
            total_cost: 0.0,
            total_input_tokens: 0,
            total_output_tokens: 0,
            request_count: 0,
            session_count: 0,
            weekly_breakdown: Vec::new(),
            avg_weekly_cost: 0.0,
            most_expensive_week: None,
            budget: None,
        }
    }

    /// Add a weekly summary to this monthly summary
    pub fn add_weekly_summary(&mut self, weekly: WeeklySummary) {
        self.total_cost += weekly.total_cost;
        self.total_input_tokens += weekly.total_input_tokens;
        self.total_output_tokens += weekly.total_output_tokens;
        self.request_count += weekly.request_count;
        self.session_count += weekly.session_count;
        self.weekly_breakdown.push(weekly);
    }

    /// Calculate average weekly cost
    pub fn calculate_avg_weekly_cost(&mut self) {
        if !self.weekly_breakdown.is_empty() {
            self.avg_weekly_cost = self.total_cost / self.weekly_breakdown.len() as f64;
            
            // Find most expensive week
            self.most_expensive_week = self.weekly_breakdown
                .iter()
                .max_by_key(|week| week.total_cost as u64)
                .map(|week| week.week_start);
        }
    }

    /// Check if monthly budget is exceeded
    pub fn is_budget_exceeded(&self) -> bool {
        if let Some(budget) = &self.budget {
            self.total_cost > budget.monthly_limit
        } else {
            false
        }
    }

    /// Get budget usage percentage
    pub fn budget_usage_percentage(&self) -> Option<f64> {
        self.budget.as_ref().map(|budget| {
            (self.total_cost / budget.monthly_limit) * 100.0
        })
    }
}

/// Budget information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetInfo {
    /// Monthly budget limit
    pub monthly_limit: f64,
    
    /// Currency code
    pub currency: String,
    
    /// Warning threshold (percentage)
    pub warning_threshold: f64,
    
    /// Alert threshold (percentage)
    pub alert_threshold: f64,
}

impl BudgetInfo {
    /// Create new budget info
    pub fn new(monthly_limit: f64, currency: String) -> Self {
        Self {
            monthly_limit,
            currency,
            warning_threshold: 80.0,
            alert_threshold: 95.0,
        }
    }

    /// Check if usage exceeds warning threshold
    pub fn is_warning_exceeded(&self, usage: f64) -> bool {
        let percentage = (usage / self.monthly_limit) * 100.0;
        percentage >= self.warning_threshold
    }

    /// Check if usage exceeds alert threshold
    pub fn is_alert_exceeded(&self, usage: f64) -> bool {
        let percentage = (usage / self.monthly_limit) * 100.0;
        percentage >= self.alert_threshold
    }
}

/// Analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResults {
    /// Analysis timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Time period analyzed
    pub period: AnalysisPeriod,
    
    /// Total cost
    pub total_cost: f64,
    
    /// Total tokens
    pub total_tokens: u64,
    
    /// Number of requests
    pub request_count: u32,
    
    /// Number of sessions
    pub session_count: u32,
    
    /// Average cost per request
    pub avg_cost_per_request: f64,
    
    /// Average tokens per request
    pub avg_tokens_per_request: f64,
    
    /// Cost breakdown by model
    pub cost_by_model: HashMap<String, f64>,
    
    /// Usage trends
    pub trends: UsageTrends,
    
    /// Insights and recommendations
    pub insights: Vec<String>,
}

impl AnalysisResults {
    /// Create new analysis results
    pub fn new(period: AnalysisPeriod) -> Self {
        Self {
            timestamp: Utc::now(),
            period,
            total_cost: 0.0,
            total_tokens: 0,
            request_count: 0,
            session_count: 0,
            avg_cost_per_request: 0.0,
            avg_tokens_per_request: 0.0,
            cost_by_model: HashMap::new(),
            trends: UsageTrends::new(),
            insights: Vec::new(),
        }
    }
}

/// Analysis time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalysisPeriod {
    Daily { date: NaiveDate },
    Weekly { week_start: NaiveDate },
    Monthly { year: u32, month: u32 },
    Custom { start: DateTime<Utc>, end: DateTime<Utc> },
}

/// Usage trends data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageTrends {
    /// Daily cost trend
    pub daily_costs: Vec<(NaiveDate, f64)>,
    
    /// Daily token trend
    pub daily_tokens: Vec<(NaiveDate, u64)>,
    
    /// Model usage trend
    pub model_trends: HashMap<String, Vec<(NaiveDate, u32)>>,
    
    /// Cost growth rate (percentage)
    pub cost_growth_rate: Option<f64>,
    
    /// Token growth rate (percentage)
    pub token_growth_rate: Option<f64>,
}

impl UsageTrends {
    /// Create new usage trends
    pub fn new() -> Self {
        Self {
            daily_costs: Vec::new(),
            daily_tokens: Vec::new(),
            model_trends: HashMap::new(),
            cost_growth_rate: None,
            token_growth_rate: None,
        }
    }
}

/// Report configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    /// Report type
    pub report_type: ReportType,
    
    /// Output format
    pub output_format: OutputFormat,
    
    /// Time period
    pub period: AnalysisPeriod,
    
    /// Include detailed breakdown
    pub include_breakdown: bool,
    
    /// Include charts (if supported)
    pub include_charts: bool,
    
    /// Output file path
    pub output_path: Option<PathBuf>,
    
    /// Filter criteria
    pub filters: ReportFilters,
}

/// Report types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportType {
    Daily,
    Weekly,
    Monthly,
    Session { session_id: String },
    Custom,
}

/// Output formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Table,
    Json,
    Csv,
    Markdown,
    Html,
}

/// Report filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportFilters {
    /// Model filter
    pub models: Vec<String>,
    
    /// Date range filter
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    
    /// Cost range filter
    pub cost_range: Option<(f64, f64)>,
    
    /// Session filter
    pub session_ids: Vec<String>,
    
    /// User filter
    pub user_ids: Vec<String>,
}

impl Default for ReportFilters {
    fn default() -> Self {
        Self {
            models: Vec::new(),
            date_range: None,
            cost_range: None,
            session_ids: Vec::new(),
            user_ids: Vec::new(),
        }
    }
}

/// Generate a unique record ID
fn generate_record_id(timestamp: &DateTime<Utc>, model: &str) -> String {
    format!("{}_{}_{}", timestamp.timestamp(), model, uuid::Uuid::new_v4())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    #[test]
    fn test_usage_record() {
        let timestamp = Utc::now();
        let record = UsageRecord::new(
            timestamp,
            "claude-3-sonnet".to_string(),
            1000,
            500,
            0.015,
        );

        assert_eq!(record.total_tokens(), 1500);
        assert!(record.cost_per_token() > 0.0);
        assert!(record.id.starts_with(&timestamp.timestamp().to_string()));
    }

    #[test]
    fn test_session() {
        let start_time = Utc::now();
        let mut session = Session::new(
            "test_session".to_string(),
            start_time,
            Some("test_user".to_string()),
        );

        let record = UsageRecord::new(
            start_time,
            "claude-3-sonnet".to_string(),
            1000,
            500,
            0.015,
        );

        session.add_record(&record);
        assert_eq!(session.request_count, 1);
        assert_eq!(session.total_cost, 0.015);
        assert_eq!(session.total_input_tokens, 1000);
        assert_eq!(session.total_output_tokens, 500);
    }

    #[test]
    fn test_pricing_info() {
        let pricing = PricingInfo {
            model: "claude-3-sonnet".to_string(),
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.015,
            currency: "USD".to_string(),
            effective_date: Utc::now(),
            is_active: true,
        };

        let cost = pricing.calculate_cost(1000, 500);
        assert_eq!(cost, 0.003 + 0.0075); // 0.003 * 1 + 0.015 * 0.5
    }

    #[test]
    fn test_daily_summary() {
        let date = NaiveDate::from_ymd_opt(2023, 12, 25).unwrap();
        let mut summary = DailySummary::new(date);

        let record = UsageRecord::new(
            DateTime::from_naive_utc_and_offset(
                NaiveDateTime::new(date, chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
                chrono::Utc,
            ),
            "claude-3-sonnet".to_string(),
            1000,
            500,
            0.015,
        );

        summary.add_record(&record);
        assert_eq!(summary.total_cost, 0.015);
        assert_eq!(summary.request_count, 1);
        assert_eq!(summary.total_input_tokens, 1000);
    }

    #[test]
    fn test_budget_info() {
        let budget = BudgetInfo::new(100.0, "USD".to_string());

        assert!(!budget.is_warning_exceeded(70.0));
        assert!(budget.is_warning_exceeded(85.0));
        assert!(!budget.is_alert_exceeded(90.0));
        assert!(budget.is_alert_exceeded(98.0));
    }
}