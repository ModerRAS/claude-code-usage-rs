//! Monthly command implementation

use crate::data::models::*;
use crate::analysis::calculator::CostCalculator;
use crate::error::Result;
use chrono::{DateTime, Utc, NaiveDate, Duration, Datelike};

/// Monthly command handler
pub struct MonthlyCommand {
    year: Option<u32>,
    month: Option<u32>,
    compare: bool,
}

impl MonthlyCommand {
    /// Create a new monthly command
    pub fn new(year: Option<u32>, month: Option<u32>, compare: bool) -> Self {
        Self { year, month, compare }
    }

    /// Execute the monthly command
    pub async fn execute(&self, records: &[UsageRecord]) -> Result<MonthlySummary> {
        let target_year = self.year.unwrap_or_else(|| chrono::Utc::now().year() as u32);
        let target_month = self.month.unwrap_or_else(|| chrono::Utc::now().month() as u32);

        let start_date = chrono::NaiveDate::from_ymd_opt(target_year as i32, target_month, 1)
            .ok_or_else(|| crate::error::CcusageError::Validation("Invalid date".to_string()))?;

        let end_date = if target_month == 12 {
            chrono::NaiveDate::from_ymd_opt(target_year as i32 + 1, 1, 1)
                .ok_or_else(|| crate::error::CcusageError::Validation("Invalid date".to_string()))?
                .pred_opt()
                .unwrap()
        } else {
            chrono::NaiveDate::from_ymd_opt(target_year as i32, target_month + 1, 1)
                .ok_or_else(|| crate::error::CcusageError::Validation("Invalid date".to_string()))?
                .pred_opt()
                .unwrap()
        };

        let month_records: Vec<_> = records
            .iter()
            .filter(|r| {
                let date = r.timestamp.date_naive();
                date >= start_date && date <= end_date
            })
            .cloned()
            .collect();

        if month_records.is_empty() {
            return Err(crate::error::CcusageError::Validation(
                format!("No usage data found for {}-{:02}", target_year, target_month)
            ));
        }

        let calculator = CostCalculator::default();
        calculator.calculate_monthly_summary(&month_records)
    }
}