//! Analyze command implementation

use crate::data::models::*;
use crate::analysis::{calculator::CostCalculator, statistics::StatisticsCalculator, trends::TrendAnalyzer};
use crate::error::Result;

/// Analyze command handler
pub struct AnalyzeCommand {
    analysis_type: AnalysisType,
    date_range: Option<(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>,
    model_filter: Option<Vec<String>>,
    detailed: bool,
}

/// Analysis types
#[derive(Debug, Clone, PartialEq)]
pub enum AnalysisType {
    Cost,
    Usage,
    Trends,
    Performance,
    Comprehensive,
}

impl AnalyzeCommand {
    /// Create a new analyze command
    pub fn new(
        analysis_type: AnalysisType,
        date_range: Option<String>,
        model_filter: Option<Vec<String>>,
        detailed: bool,
    ) -> Result<Self> {
        let parsed_date_range = if let Some(range_str) = date_range {
            Some(crate::data::parser::DataParser::parse_date_range(&range_str)?)
        } else {
            None
        };

        Ok(Self {
            analysis_type,
            date_range: parsed_date_range,
            model_filter,
            detailed,
        })
    }

    /// Execute the analyze command
    pub async fn execute(&self, records: &[UsageRecord]) -> Result<AnalysisResult> {
        // Apply filters
        let filtered_records = self.apply_filters(records)?;

        match self.analysis_type {
            AnalysisType::Cost => {
                let calculator = CostCalculator::default();
                let breakdown = calculator.calculate_detailed_breakdown(&filtered_records)?;
                Ok(AnalysisResult::CostAnalysis(breakdown))
            },
            AnalysisType::Usage => {
                let stats = StatisticsCalculator::calculate_usage_stats(&filtered_records);
                Ok(AnalysisResult::UsageAnalysis(stats))
            },
            AnalysisType::Trends => {
                let analyzer = TrendAnalyzer::default();
                let trends = analyzer.analyze_trends(&filtered_records)?;
                Ok(AnalysisResult::TrendAnalysis(trends))
            },
            AnalysisType::Performance => {
                let stats = StatisticsCalculator::calculate_usage_stats(&filtered_records);
                Ok(AnalysisResult::PerformanceAnalysis(stats))
            },
            AnalysisType::Comprehensive => {
                let calculator = CostCalculator::default();
                let breakdown = calculator.calculate_detailed_breakdown(&filtered_records)?;

                let stats = StatisticsCalculator::calculate_usage_stats(&filtered_records);

                let analyzer = TrendAnalyzer::default();
                let trends = analyzer.analyze_trends(&filtered_records)?;

                Ok(AnalysisResult::ComprehensiveAnalysis {
                    cost_breakdown: breakdown,
                    usage_stats: stats,
                    trends,
                })
            },
        }
    }

    fn apply_filters(&self, records: &[UsageRecord]) -> Result<Vec<UsageRecord>> {
        let mut filtered = records.to_vec();

        // Apply date range filter
        if let Some((start, end)) = &self.date_range {
            filtered = filtered
                .into_iter()
                .filter(|r| r.is_within_range(*start, *end))
                .collect();
        }

        // Apply model filter
        if let Some(models) = &self.model_filter {
            filtered = filtered
                .into_iter()
                .filter(|r| models.contains(&r.model))
                .collect();
        }

        Ok(filtered)
    }
}

/// Analysis result
pub enum AnalysisResult {
    CostAnalysis(crate::analysis::calculator::DetailedCostBreakdown),
    UsageAnalysis(crate::analysis::statistics::UsageStatistics),
    TrendAnalysis(crate::analysis::trends::TrendAnalysis),
    PerformanceAnalysis(crate::analysis::statistics::UsageStatistics),
    ComprehensiveAnalysis {
        cost_breakdown: crate::analysis::calculator::DetailedCostBreakdown,
        usage_stats: crate::analysis::statistics::UsageStatistics,
        trends: crate::analysis::trends::TrendAnalysis,
    },
}