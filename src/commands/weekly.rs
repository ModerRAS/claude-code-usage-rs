//! Weekly command implementation

use crate::data::models::*;
use crate::analysis::calculator::CostCalculator;
use crate::error::Result;
use chrono::{Datelike, Weekday};

/// Weekly command handler
pub struct WeeklyCommand {
    week_start: Option<String>,
    weeks: u32,
}

impl WeeklyCommand {
    /// Create a new weekly command
    pub fn new(week_start: Option<String>, weeks: u32) -> Self {
        Self { week_start, weeks }
    }

    /// Execute the weekly command
    pub async fn execute(&self, records: &[UsageRecord]) -> Result<Vec<WeeklySummary>> {
        let week_start_date = if let Some(date_str) = &self.week_start {
            crate::utils::parse_date_flexible(date_str)?.date_naive()
        } else {
            // Get current week's Monday
            let today = chrono::Utc::now().date_naive();
            let days_since_monday = today.weekday() as i64;
            today - chrono::Duration::days(days_since_monday)
        };

        let mut weekly_reports = Vec::new();

        for week in 0..self.weeks {
            let start_date = week_start_date + chrono::Duration::weeks(week as i64);
            let end_date = start_date + chrono::Duration::days(6);

            let week_records: Vec<_> = records
                .iter()
                .filter(|r| {
                    let date = r.timestamp.date_naive();
                    date >= start_date && date <= end_date
                })
                .cloned()
                .collect();

            if !week_records.is_empty() {
                let calculator = CostCalculator::default();
                let weekly_summary = calculator.calculate_weekly_summary(&week_records)?;
                weekly_reports.push(weekly_summary);
            }
        }

        Ok(weekly_reports)
    }
}