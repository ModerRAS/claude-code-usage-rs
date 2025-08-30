//! Daily command implementation

use crate::data::models::*;
use crate::analysis::calculator::CostCalculator;
use crate::error::Result;

/// Daily command handler
pub struct DailyCommand {
    date: Option<String>,
    compare: bool,
}

impl DailyCommand {
    /// Create a new daily command
    pub fn new(date: Option<String>, compare: bool) -> Self {
        Self { date, compare }
    }

    /// Execute the daily command
    pub async fn execute(&self, records: &[UsageRecord]) -> Result<DailySummary> {
        let target_date = if let Some(date_str) = &self.date {
            crate::utils::parse_date_flexible(date_str)?.date_naive()
        } else {
            chrono::Utc::now().date_naive()
        };

        let daily_records: Vec<_> = records
            .iter()
            .filter(|r| r.is_on_date(target_date))
            .cloned()
            .collect();

        if daily_records.is_empty() {
            return Err(crate::error::CcusageError::Validation(
                format!("No usage data found for {}", target_date)
            ));
        }

        let calculator = CostCalculator::default();
        calculator.calculate_daily_summary(&daily_records)
    }
}