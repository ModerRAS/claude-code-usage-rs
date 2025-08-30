//! Output formatting module for ccusage-rs
//! 
//! This module provides various output formats and formatters for
//! displaying analysis results and reports.

use crate::data::models::*;
use crate::analysis::*;
use crate::error::{Result, CcusageError};
use comfy_table::{Table, Cell, Row, Attribute};
use serde_json;
use std::path::Path;
use std::collections::HashMap;

/// Output formats
#[derive(Debug, Clone, PartialEq, clap::ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Csv,
    Markdown,
    Html,
}

/// Export formats
#[derive(Debug, Clone, PartialEq)]
pub enum ExportFormat {
    Json,
    Csv,
    Parquet,
}

/// Output formatter
pub struct OutputFormatter {
    format: OutputFormat,
}

impl OutputFormatter {
    /// Create a new output formatter
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    /// Output cost breakdown
    pub fn output_cost_breakdown(&self, breakdown: &DetailedCostBreakdown, output_path: Option<&str>) -> Result<()> {
        match self.format {
            OutputFormat::Table => self.output_cost_breakdown_table(breakdown),
            OutputFormat::Json => self.output_cost_breakdown_json(breakdown, output_path),
            OutputFormat::Csv => self.output_cost_breakdown_csv(breakdown, output_path),
            OutputFormat::Markdown => self.output_cost_breakdown_markdown(breakdown, output_path),
            OutputFormat::Html => self.output_cost_breakdown_html(breakdown, output_path),
        }
    }

    /// Output usage statistics
    pub fn output_usage_stats(&self, stats: &UsageStatistics, output_path: Option<&str>) -> Result<()> {
        match self.format {
            OutputFormat::Table => self.output_usage_stats_table(stats),
            OutputFormat::Json => self.output_usage_stats_json(stats, output_path),
            OutputFormat::Csv => self.output_usage_stats_csv(stats, output_path),
            OutputFormat::Markdown => self.output_usage_stats_markdown(stats, output_path),
            OutputFormat::Html => self.output_usage_stats_html(stats, output_path),
        }
    }

    /// Output trends analysis
    pub fn output_trends(&self, trends: &TrendAnalysis, output_path: Option<&str>) -> Result<()> {
        match self.format {
            OutputFormat::Table => self.output_trends_table(trends),
            OutputFormat::Json => self.output_trends_json(trends, output_path),
            OutputFormat::Csv => self.output_trends_csv(trends, output_path),
            OutputFormat::Markdown => self.output_trends_markdown(trends, output_path),
            OutputFormat::Html => self.output_trends_html(trends, output_path),
        }
    }

    /// Output comprehensive analysis
    pub fn output_comprehensive_analysis(
        &self,
        breakdown: &DetailedCostBreakdown,
        stats: &UsageStatistics,
        trends: &TrendAnalysis,
        output_path: Option<&str>,
    ) -> Result<()> {
        match self.format {
            OutputFormat::Table => self.output_comprehensive_analysis_table(breakdown, stats, trends),
            OutputFormat::Json => self.output_comprehensive_analysis_json(breakdown, stats, trends, output_path),
            OutputFormat::Csv => self.output_comprehensive_analysis_csv(breakdown, stats, trends, output_path),
            OutputFormat::Markdown => self.output_comprehensive_analysis_markdown(breakdown, stats, trends, output_path),
            OutputFormat::Html => self.output_comprehensive_analysis_html(breakdown, stats, trends, output_path),
        }
    }

    /// Output daily report
    pub fn output_daily_report(
        &self,
        summary: &DailySummary,
        comparison: Option<(&DailySummary, &DailySummary)>,
        output_path: Option<&str>,
    ) -> Result<()> {
        match self.format {
            OutputFormat::Table => self.output_daily_report_table(summary, comparison),
            OutputFormat::Json => self.output_daily_report_json(summary, comparison, output_path),
            OutputFormat::Csv => self.output_daily_report_csv(summary, comparison, output_path),
            OutputFormat::Markdown => self.output_daily_report_markdown(summary, comparison, output_path),
            OutputFormat::Html => self.output_daily_report_html(summary, comparison, output_path),
        }
    }

    /// Output weekly report
    pub fn output_weekly_report(&self, summaries: &[WeeklySummary], output_path: Option<&str>) -> Result<()> {
        match self.format {
            OutputFormat::Table => self.output_weekly_report_table(summaries),
            OutputFormat::Json => self.output_weekly_report_json(summaries, output_path),
            OutputFormat::Csv => self.output_weekly_report_csv(summaries, output_path),
            OutputFormat::Markdown => self.output_weekly_report_markdown(summaries, output_path),
            OutputFormat::Html => self.output_weekly_report_html(summaries, output_path),
        }
    }

    /// Output monthly report
    pub fn output_monthly_report(
        &self,
        summary: &MonthlySummary,
        comparison: Option<(&MonthlySummary, &MonthlySummary)>,
        output_path: Option<&str>,
    ) -> Result<()> {
        match self.format {
            OutputFormat::Table => self.output_monthly_report_table(summary, comparison),
            OutputFormat::Json => self.output_monthly_report_json(summary, comparison, output_path),
            OutputFormat::Csv => self.output_monthly_report_csv(summary, comparison, output_path),
            OutputFormat::Markdown => self.output_monthly_report_markdown(summary, comparison, output_path),
            OutputFormat::Html => self.output_monthly_report_html(summary, comparison, output_path),
        }
    }

    /// Output session list
    pub fn output_session_list(&self, sessions: &[Session], output_path: Option<&str>) -> Result<()> {
        match self.format {
            OutputFormat::Table => self.output_session_list_table(sessions),
            OutputFormat::Json => self.output_session_list_json(sessions, output_path),
            OutputFormat::Csv => self.output_session_list_csv(sessions, output_path),
            OutputFormat::Markdown => self.output_session_list_markdown(sessions, output_path),
            OutputFormat::Html => self.output_session_list_html(sessions, output_path),
        }
    }

    /// Output session analysis
    pub fn output_session_analysis(&self, analysis: &SessionAnalysis, output_path: Option<&str>) -> Result<()> {
        match self.format {
            OutputFormat::Table => self.output_session_analysis_table(analysis),
            OutputFormat::Json => self.output_session_analysis_json(analysis, output_path),
            OutputFormat::Csv => self.output_session_analysis_csv(analysis, output_path),
            OutputFormat::Markdown => self.output_session_analysis_markdown(analysis, output_path),
            OutputFormat::Html => self.output_session_analysis_html(analysis, output_path),
        }
    }

    /// Output budget status
    pub fn output_budget_status(&self, budget: &BudgetInfo, analysis: &BudgetAnalysis, output_path: Option<&str>) -> Result<()> {
        match self.format {
            OutputFormat::Table => self.output_budget_status_table(budget, analysis),
            OutputFormat::Json => self.output_budget_status_json(budget, analysis, output_path),
            OutputFormat::Csv => self.output_budget_status_csv(budget, analysis, output_path),
            OutputFormat::Markdown => self.output_budget_status_markdown(budget, analysis, output_path),
            OutputFormat::Html => self.output_budget_status_html(budget, analysis, output_path),
        }
    }

    /// Output model statistics
    pub fn output_model_stats(&self, model_stats: &HashMap<String, ModelStats>, output_path: Option<&str>) -> Result<()> {
        match self.format {
            OutputFormat::Table => self.output_model_stats_table(model_stats),
            OutputFormat::Json => self.output_model_stats_json(model_stats, output_path),
            OutputFormat::Csv => self.output_model_stats_csv(model_stats, output_path),
            OutputFormat::Markdown => self.output_model_stats_markdown(model_stats, output_path),
            OutputFormat::Html => self.output_model_stats_html(model_stats, output_path),
        }
    }

    /// Output performance statistics
    pub fn output_performance_stats(&self, stats: &UsageStatistics, output_path: Option<&str>) -> Result<()> {
        match self.format {
            OutputFormat::Table => self.output_performance_stats_table(stats),
            OutputFormat::Json => self.output_performance_stats_json(stats, output_path),
            OutputFormat::Csv => self.output_performance_stats_csv(stats, output_path),
            OutputFormat::Markdown => self.output_performance_stats_markdown(stats, output_path),
            OutputFormat::Html => self.output_performance_stats_html(stats, output_path),
        }
    }

    /// Output insights
    pub fn output_insights(&self, insights: &[crate::analysis::insights::Insight], output_path: Option<&str>) -> Result<()> {
        match self.format {
            OutputFormat::Table => self.output_insights_table(insights),
            OutputFormat::Json => self.output_insights_json(insights, output_path),
            OutputFormat::Csv => self.output_insights_csv(insights, output_path),
            OutputFormat::Markdown => self.output_insights_markdown(insights, output_path),
            OutputFormat::Html => self.output_insights_html(insights, output_path),
        }
    }

    /// Export data
    pub fn export_data(&self, records: &[UsageRecord], format: ExportFormat, output_path: &Path) -> Result<()> {
        match format {
            ExportFormat::Json => self.export_data_json(records, output_path),
            ExportFormat::Csv => self.export_data_csv(records, output_path),
            ExportFormat::Parquet => self.export_data_parquet(records, output_path),
        }
    }

    // Table output methods
    fn output_cost_breakdown_table(&self, breakdown: &DetailedCostBreakdown) -> Result<()> {
        let mut table = Table::new();
        table.set_header(vec![
            Cell::new("Metric").add_attribute(Attribute::Bold),
            Cell::new("Value").add_attribute(Attribute::Bold),
        ]);

        table.add_row(vec![
            Cell::new("Total Cost"),
            Cell::new(format!("${:.6}", breakdown.total_cost)),
        ]);

        table.add_row(vec![
            Cell::new("Total Requests"),
            Cell::new(breakdown.total_records),
        ]);

        table.add_row(vec![
            Cell::new("Total Tokens"),
            Cell::new(breakdown.total_tokens),
        ]);

        table.add_row(vec![
            Cell::new("Avg Cost per Request"),
            Cell::new(format!("${:.6}", breakdown.avg_cost_per_request)),
        ]);

        table.add_row(vec![
            Cell::new("Avg Tokens per Request"),
            Cell::new(format!("{:.1}", breakdown.avg_tokens_per_request)),
        ]);

        println!("{}", table);
        Ok(())
    }

    fn output_usage_stats_table(&self, stats: &UsageStatistics) -> Result<()> {
        let mut table = Table::new();
        table.set_header(vec![
            Cell::new("Metric").add_attribute(Attribute::Bold),
            Cell::new("Value").add_attribute(Attribute::Bold),
        ]);

        table.add_row(vec![
            Cell::new("Total Requests"),
            Cell::new(stats.total_requests),
        ]);

        table.add_row(vec![
            Cell::new("Total Tokens"),
            Cell::new(stats.total_tokens),
        ]);

        table.add_row(vec![
            Cell::new("Total Cost"),
            Cell::new(format!("${:.6}", stats.total_cost)),
        ]);

        table.add_row(vec![
            Cell::new("Avg Tokens per Request"),
            Cell::new(format!("{:.1}", stats.average_tokens_per_request)),
        ]);

        table.add_row(vec![
            Cell::new("Avg Cost per Request"),
            Cell::new(format!("${:.6}", stats.average_cost_per_request)),
        ]);

        println!("{}", table);
        Ok(())
    }

    fn output_trends_table(&self, trends: &TrendAnalysis) -> Result<()> {
        let mut table = Table::new();
        table.set_header(vec![
            Cell::new("Trend Metric").add_attribute(Attribute::Bold),
            Cell::new("Value").add_attribute(Attribute::Bold),
        ]);

        table.add_row(vec![
            Cell::new("Overall Trend"),
            Cell::new(format!("{:?}", trends.overall_trend)),
        ]);

        table.add_row(vec![
            Cell::new("Cost Growth Rate"),
            Cell::new(format!("{:.1}%", trends.cost_trend.growth_rate)),
        ]);

        table.add_row(vec![
            Cell::new("Token Growth Rate"),
            Cell::new(format!("{:.1}%", trends.token_trend.growth_rate)),
        ]);

        table.add_row(vec![
            Cell::new("Request Growth Rate"),
            Cell::new(format!("{:.1}%", trends.request_trend.growth_rate)),
        ]);

        println!("{}", table);
        Ok(())
    }

    fn output_comprehensive_analysis_table(
        &self,
        breakdown: &DetailedCostBreakdown,
        stats: &UsageStatistics,
        trends: &TrendAnalysis,
    ) -> Result<()> {
        println!("=== Comprehensive Usage Analysis ===\n");
        
        println!("Cost Analysis:");
        self.output_cost_breakdown_table(breakdown)?;
        
        println!("\nUsage Statistics:");
        self.output_usage_stats_table(stats)?;
        
        println!("\nTrend Analysis:");
        self.output_trends_table(trends)?;
        
        Ok(())
    }

    fn output_daily_report_table(&self, summary: &DailySummary, comparison: Option<(&DailySummary, &DailySummary)>) -> Result<()> {
        let mut table = Table::new();
        table.set_header(vec![
            Cell::new("Daily Summary").add_attribute(Attribute::Bold),
            Cell::new("Value").add_attribute(Attribute::Bold),
        ]);

        table.add_row(vec![
            Cell::new("Date"),
            Cell::new(summary.date),
        ]);

        table.add_row(vec![
            Cell::new("Total Cost"),
            Cell::new(format!("${:.6}", summary.total_cost)),
        ]);

        table.add_row(vec![
            Cell::new("Total Requests"),
            Cell::new(summary.request_count),
        ]);

        table.add_row(vec![
            Cell::new("Total Tokens"),
            Cell::new(summary.total_input_tokens + summary.total_output_tokens),
        ]);

        println!("{}", table);

        if let Some((prev, current)) = comparison {
            println!("\nComparison with previous day:");
            let mut comp_table = Table::new();
            comp_table.set_header(vec![
                Cell::new("Metric").add_attribute(Attribute::Bold),
                Cell::new("Previous").add_attribute(Attribute::Bold),
                Cell::new("Current").add_attribute(Attribute::Bold),
                Cell::new("Change").add_attribute(Attribute::Bold),
            ]);

            let cost_change = if prev.total_cost > 0.0 {
                ((current.total_cost - prev.total_cost) / prev.total_cost) * 100.0
            } else {
                0.0
            };

            comp_table.add_row(vec![
                Cell::new("Total Cost"),
                Cell::new(format!("${:.6}", prev.total_cost)),
                Cell::new(format!("${:.6}", current.total_cost)),
                Cell::new(format!("{:.1}%", cost_change)),
            ]);

            println!("{}", comp_table);
        }

        Ok(())
    }

    fn output_weekly_report_table(&self, summaries: &[WeeklySummary]) -> Result<()> {
        let mut table = Table::new();
        table.set_header(vec![
            Cell::new("Week Start").add_attribute(Attribute::Bold),
            Cell::new("Total Cost").add_attribute(Attribute::Bold),
            Cell::new("Requests").add_attribute(Attribute::Bold),
            Cell::new("Avg Daily Cost").add_attribute(Attribute::Bold),
        ]);

        for summary in summaries {
            table.add_row(vec![
                Cell::new(summary.week_start),
                Cell::new(format!("${:.6}", summary.total_cost)),
                Cell::new(summary.request_count),
                Cell::new(format!("${:.6}", summary.avg_daily_cost)),
            ]);
        }

        println!("{}", table);
        Ok(())
    }

    fn output_monthly_report_table(&self, summary: &MonthlySummary, _comparison: Option<(&MonthlySummary, &MonthlySummary)>) -> Result<()> {
        let mut table = Table::new();
        table.set_header(vec![
            Cell::new("Monthly Summary").add_attribute(Attribute::Bold),
            Cell::new("Value").add_attribute(Attribute::Bold),
        ]);

        table.add_row(vec![
            Cell::new("Period"),
            Cell::new(format!("{}-{:02}", summary.year, summary.month)),
        ]);

        table.add_row(vec![
            Cell::new("Total Cost"),
            Cell::new(format!("${:.6}", summary.total_cost)),
        ]);

        table.add_row(vec![
            Cell::new("Total Requests"),
            Cell::new(summary.request_count),
        ]);

        table.add_row(vec![
            Cell::new("Total Tokens"),
            Cell::new(summary.total_input_tokens + summary.total_output_tokens),
        ]);

        println!("{}", table);
        Ok(())
    }

    fn output_session_list_table(&self, sessions: &[Session]) -> Result<()> {
        let mut table = Table::new();
        table.set_header(vec![
            Cell::new("Session ID").add_attribute(Attribute::Bold),
            Cell::new("Start Time").add_attribute(Attribute::Bold),
            Cell::new("Duration").add_attribute(Attribute::Bold),
            Cell::new("Requests").add_attribute(Attribute::Bold),
            Cell::new("Total Cost").add_attribute(Attribute::Bold),
        ]);

        for session in sessions {
            let duration = session.duration_seconds
                .map(|d| format!("{}s", d))
                .unwrap_or_else(|| "N/A".to_string());

            table.add_row(vec![
                Cell::new(&session.id),
                Cell::new(session.start_time.format("%Y-%m-%d %H:%M:%S")),
                Cell::new(duration),
                Cell::new(session.request_count),
                Cell::new(format!("${:.6}", session.total_cost)),
            ]);
        }

        println!("{}", table);
        Ok(())
    }

    fn output_session_analysis_table(&self, analysis: &SessionAnalysis) -> Result<()> {
        let mut table = Table::new();
        table.set_header(vec![
            Cell::new("Session Analysis").add_attribute(Attribute::Bold),
            Cell::new("Value").add_attribute(Attribute::Bold),
        ]);

        table.add_row(vec![
            Cell::new("Session ID"),
            Cell::new(&analysis.session_id),
        ]);

        table.add_row(vec![
            Cell::new("Total Cost"),
            Cell::new(format!("${:.6}", analysis.total_cost)),
        ]);

        table.add_row(vec![
            Cell::new("Total Requests"),
            Cell::new(analysis.request_count),
        ]);

        table.add_row(vec![
            Cell::new("Total Tokens"),
            Cell::new(analysis.total_tokens),
        ]);

        table.add_row(vec![
            Cell::new("Duration"),
            Cell::new(format!("{}s", analysis.duration)),
        ]);

        println!("{}", table);
        Ok(())
    }

    fn output_budget_status_table(&self, budget: &BudgetInfo, analysis: &BudgetAnalysis) -> Result<()> {
        let mut table = Table::new();
        table.set_header(vec![
            Cell::new("Budget Status").add_attribute(Attribute::Bold),
            Cell::new("Value").add_attribute(Attribute::Bold),
        ]);

        table.add_row(vec![
            Cell::new("Budget Limit"),
            Cell::new(format!("{} {:.2}", budget.currency, budget.monthly_limit)),
        ]);

        table.add_row(vec![
            Cell::new("Current Usage"),
            Cell::new(format!("{} {:.2}", budget.currency, analysis.current_usage)),
        ]);

        table.add_row(vec![
            Cell::new("Usage Percentage"),
            Cell::new(format!("{:.1}%", analysis.budget_usage_percentage)),
        ]);

        table.add_row(vec![
            Cell::new("Status"),
            Cell::new(if analysis.is_budget_exceeded { "EXCEEDED" } else { "OK" }),
        ]);

        println!("{}", table);
        Ok(())
    }

    fn output_model_stats_table(&self, model_stats: &HashMap<String, ModelStats>) -> Result<()> {
        let mut table = Table::new();
        table.set_header(vec![
            Cell::new("Model").add_attribute(Attribute::Bold),
            Cell::new("Requests").add_attribute(Attribute::Bold),
            Cell::new("Total Cost").add_attribute(Attribute::Bold),
            Cell::new("Usage %").add_attribute(Attribute::Bold),
        ]);

        for (model, stats) in model_stats {
            table.add_row(vec![
                Cell::new(model),
                Cell::new(stats.request_count),
                Cell::new(format!("${:.6}", stats.total_cost)),
                Cell::new(format!("{:.1}%", stats.usage_percentage)),
            ]);
        }

        println!("{}", table);
        Ok(())
    }

    fn output_performance_stats_table(&self, stats: &UsageStatistics) -> Result<()> {
        let mut table = Table::new();
        table.set_header(vec![
            Cell::new("Performance Metric").add_attribute(Attribute::Bold),
            Cell::new("Value").add_attribute(Attribute::Bold),
        ]);

        table.add_row(vec![
            Cell::new("Total Requests"),
            Cell::new(stats.total_requests),
        ]);

        table.add_row(vec![
            Cell::new("Average Tokens per Request"),
            Cell::new(format!("{:.1}", stats.average_tokens_per_request)),
        ]);

        table.add_row(vec![
            Cell::new("Average Cost per Request"),
            Cell::new(format!("${:.6}", stats.average_cost_per_request)),
        ]);

        table.add_row(vec![
            Cell::new("Request Frequency (per hour)"),
            Cell::new(format!("{:.1}", stats.request_frequency_per_hour)),
        ]);

        println!("{}", table);
        Ok(())
    }

    fn output_insights_table(&self, insights: &[crate::analysis::insights::Insight]) -> Result<()> {
        let mut table = Table::new();
        table.set_header(vec![
            Cell::new("Type").add_attribute(Attribute::Bold),
            Cell::new("Severity").add_attribute(Attribute::Bold),
            Cell::new("Title").add_attribute(Attribute::Bold),
            Cell::new("Confidence").add_attribute(Attribute::Bold),
        ]);

        for insight in insights {
            table.add_row(vec![
                Cell::new(format!("{:?}", insight.insight_type)),
                Cell::new(format!("{:?}", insight.severity)),
                Cell::new(&insight.title),
                Cell::new(format!("{:.1}%", insight.confidence * 100.0)),
            ]);
        }

        println!("{}", table);
        Ok(())
    }

    // JSON output methods (simplified implementations)
    fn output_cost_breakdown_json(&self, breakdown: &DetailedCostBreakdown, output_path: Option<&str>) -> Result<()> {
        let json = serde_json::to_string_pretty(breakdown)?;
        self.write_output(&json, output_path)
    }

    fn output_usage_stats_json(&self, stats: &UsageStatistics, output_path: Option<&str>) -> Result<()> {
        let json = serde_json::to_string_pretty(stats)?;
        self.write_output(&json, output_path)
    }

    fn output_trends_json(&self, trends: &TrendAnalysis, output_path: Option<&str>) -> Result<()> {
        let json = serde_json::to_string_pretty(trends)?;
        self.write_output(&json, output_path)
    }

    fn output_comprehensive_analysis_json(
        &self,
        breakdown: &DetailedCostBreakdown,
        stats: &UsageStatistics,
        trends: &TrendAnalysis,
        output_path: Option<&str>,
    ) -> Result<()> {
        let analysis = serde_json::json!({
            "cost_breakdown": breakdown,
            "usage_stats": stats,
            "trends": trends
        });
        let json = serde_json::to_string_pretty(&analysis)?;
        self.write_output(&json, output_path)
    }

    fn output_daily_report_json(
        &self,
        summary: &DailySummary,
        comparison: Option<(&DailySummary, &DailySummary)>,
        output_path: Option<&str>,
    ) -> Result<()> {
        let report = serde_json::json!({
            "summary": summary,
            "comparison": comparison.map(|(p, c)| serde_json::json!({"previous": p, "current": c}))
        });
        let json = serde_json::to_string_pretty(&report)?;
        self.write_output(&json, output_path)
    }

    fn output_weekly_report_json(&self, summaries: &[WeeklySummary], output_path: Option<&str>) -> Result<()> {
        let json = serde_json::to_string_pretty(summaries)?;
        self.write_output(&json, output_path)
    }

    fn output_monthly_report_json(
        &self,
        summary: &MonthlySummary,
        comparison: Option<(&MonthlySummary, &MonthlySummary)>,
        output_path: Option<&str>,
    ) -> Result<()> {
        let report = serde_json::json!({
            "summary": summary,
            "comparison": comparison.map(|(p, c)| serde_json::json!({"previous": p, "current": c}))
        });
        let json = serde_json::to_string_pretty(&report)?;
        self.write_output(&json, output_path)
    }

    fn output_session_list_json(&self, sessions: &[Session], output_path: Option<&str>) -> Result<()> {
        let json = serde_json::to_string_pretty(sessions)?;
        self.write_output(&json, output_path)
    }

    fn output_session_analysis_json(&self, analysis: &SessionAnalysis, output_path: Option<&str>) -> Result<()> {
        let json = serde_json::to_string_pretty(analysis)?;
        self.write_output(&json, output_path)
    }

    fn output_budget_status_json(&self, budget: &BudgetInfo, analysis: &BudgetAnalysis, output_path: Option<&str>) -> Result<()> {
        let status = serde_json::json!({
            "budget": budget,
            "analysis": analysis
        });
        let json = serde_json::to_string_pretty(&status)?;
        self.write_output(&json, output_path)
    }

    fn output_model_stats_json(&self, model_stats: &HashMap<String, ModelStats>, output_path: Option<&str>) -> Result<()> {
        let json = serde_json::to_string_pretty(model_stats)?;
        self.write_output(&json, output_path)
    }

    fn output_performance_stats_json(&self, stats: &UsageStatistics, output_path: Option<&str>) -> Result<()> {
        let json = serde_json::to_string_pretty(stats)?;
        self.write_output(&json, output_path)
    }

    fn output_insights_json(&self, insights: &[crate::analysis::insights::Insight], output_path: Option<&str>) -> Result<()> {
        let json = serde_json::to_string_pretty(insights)?;
        self.write_output(&json, output_path)
    }

    // Export methods
    fn export_data_json(&self, records: &[UsageRecord], output_path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(records)?;
        std::fs::write(output_path, json).map_err(|e| {
            CcusageError::FileSystem(format!("Failed to write export file: {}", e))
        })?;
        Ok(())
    }

    fn export_data_csv(&self, records: &[UsageRecord], output_path: &Path) -> Result<()> {
        let mut writer = csv::Writer::from_path(output_path).map_err(|e| {
            CcusageError::Csv(e)
        })?;

        // Write header
        writer.write_record(&[
            "id", "timestamp", "model", "input_tokens", "output_tokens", 
            "cost", "session_id", "user_id"
        ]).map_err(|e| {
            CcusageError::Csv(e)
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
                CcusageError::Csv(e)
            })?;
        }

        writer.flush().map_err(|e| {
            CcusageError::Csv(e.into())
        })?;

        Ok(())
    }

    fn export_data_parquet(&self, _records: &[UsageRecord], _output_path: &Path) -> Result<()> {
        // Parquet export not implemented in this simplified version
        Err(CcusageError::Application(
            "Parquet export not yet implemented".to_string()
        ))
    }

    // Helper method to write output
    fn write_output(&self, content: &str, output_path: Option<&str>) -> Result<()> {
        if let Some(path) = output_path {
            std::fs::write(path, content).map_err(|e| {
                CcusageError::FileSystem(format!("Failed to write output file: {}", e))
            })?;
        } else {
            println!("{}", content);
        }
        Ok(())
    }

    // Placeholder implementations for other formats
    fn output_cost_breakdown_csv(&self, _breakdown: &DetailedCostBreakdown, _output_path: Option<&str>) -> Result<()> {
        Ok(())
    }

    fn output_usage_stats_csv(&self, _stats: &UsageStatistics, _output_path: Option<&str>) -> Result<()> {
        Ok(())
    }

    fn output_trends_csv(&self, _trends: &TrendAnalysis, _output_path: Option<&str>) -> Result<()> {
        Ok(())
    }

    fn output_comprehensive_analysis_csv(
        &self,
        _breakdown: &DetailedCostBreakdown,
        _stats: &UsageStatistics,
        _trends: &TrendAnalysis,
        _output_path: Option<&str>,
    ) -> Result<()> {
        Ok(())
    }

    fn output_daily_report_csv(
        &self,
        _summary: &DailySummary,
        _comparison: Option<(&DailySummary, &DailySummary)>,
        _output_path: Option<&str>,
    ) -> Result<()> {
        Ok(())
    }

    fn output_weekly_report_csv(&self, _summaries: &[WeeklySummary], _output_path: Option<&str>) -> Result<()> {
        Ok(())
    }

    fn output_monthly_report_csv(
        &self,
        _summary: &MonthlySummary,
        _comparison: Option<(&MonthlySummary, &MonthlySummary)>,
        _output_path: Option<&str>,
    ) -> Result<()> {
        Ok(())
    }

    fn output_session_list_csv(&self, _sessions: &[Session], _output_path: Option<&str>) -> Result<()> {
        Ok(())
    }

    fn output_session_analysis_csv(&self, _analysis: &SessionAnalysis, _output_path: Option<&str>) -> Result<()> {
        Ok(())
    }

    fn output_budget_status_csv(&self, _budget: &BudgetInfo, _analysis: &BudgetAnalysis, _output_path: Option<&str>) -> Result<()> {
        Ok(())
    }

    fn output_model_stats_csv(&self, _model_stats: &HashMap<String, ModelStats>, _output_path: Option<&str>) -> Result<()> {
        Ok(())
    }

    fn output_performance_stats_csv(&self, _stats: &UsageStatistics, _output_path: Option<&str>) -> Result<()> {
        Ok(())
    }

    fn output_insights_csv(&self, _insights: &[crate::analysis::insights::Insight], _output_path: Option<&str>) -> Result<()> {
        Ok(())
    }

    // Markdown implementations (simplified)
    fn output_cost_breakdown_markdown(&self, breakdown: &DetailedCostBreakdown, output_path: Option<&str>) -> Result<()> {
        let content = format!(
            "# Cost Breakdown\n\n\
            | Metric | Value |\n\
            |--------|-------|\n\
            | Total Cost | ${:.6} |\n\
            | Total Requests | {} |\n\
            | Total Tokens | {} |\n\
            | Avg Cost per Request | ${:.6} |\n\
            | Avg Tokens per Request | {:.1} |\n",
            breakdown.total_cost,
            breakdown.total_records,
            breakdown.total_tokens,
            breakdown.avg_cost_per_request,
            breakdown.avg_tokens_per_request
        );
        self.write_output(&content, output_path)
    }

    fn output_usage_stats_markdown(&self, stats: &UsageStatistics, output_path: Option<&str>) -> Result<()> {
        let content = format!(
            "# Usage Statistics\n\n\
            | Metric | Value |\n\
            |--------|-------|\n\
            | Total Requests | {} |\n\
            | Total Tokens | {} |\n\
            | Total Cost | ${:.6} |\n\
            | Avg Tokens per Request | {:.1} |\n\
            | Avg Cost per Request | ${:.6} |\n",
            stats.total_requests,
            stats.total_tokens,
            stats.total_cost,
            stats.average_tokens_per_request,
            stats.average_cost_per_request
        );
        self.write_output(&content, output_path)
    }

    fn output_trends_markdown(&self, trends: &TrendAnalysis, output_path: Option<&str>) -> Result<()> {
        let content = format!(
            "# Trends Analysis\n\n\
            | Trend Metric | Value |\n\
            |--------------|-------|\n\
            | Overall Trend | {:?} |\n\
            | Cost Growth Rate | {:.1}% |\n\
            | Token Growth Rate | {:.1}% |\n\
            | Request Growth Rate | {:.1}% |\n",
            trends.overall_trend,
            trends.cost_trend.growth_rate,
            trends.token_trend.growth_rate,
            trends.request_trend.growth_rate
        );
        self.write_output(&content, output_path)
    }

    fn output_comprehensive_analysis_markdown(
        &self,
        breakdown: &DetailedCostBreakdown,
        stats: &UsageStatistics,
        trends: &TrendAnalysis,
        output_path: Option<&str>,
    ) -> Result<()> {
        let mut content = String::from("# Comprehensive Usage Analysis\n\n");
        
        content.push_str("## Cost Analysis\n\n");
        content.push_str(&format!(
            "| Metric | Value |\n\
             |--------|-------|\n\
             | Total Cost | ${:.6} |\n\
             | Total Requests | {} |\n\
             | Total Tokens | {} |\n\
             | Avg Cost per Request | ${:.6} |\n\
             | Avg Tokens per Request | {:.1} |\n\n",
            breakdown.total_cost,
            breakdown.total_records,
            breakdown.total_tokens,
            breakdown.avg_cost_per_request,
            breakdown.avg_tokens_per_request
        ));
        
        content.push_str("## Usage Statistics\n\n");
        content.push_str(&format!(
            "| Metric | Value |\n\
             |--------|-------|\n\
             | Total Requests | {} |\n\
             | Total Tokens | {} |\n\
             | Total Cost | ${:.6} |\n\
             | Avg Tokens per Request | {:.1} |\n\
             | Avg Cost per Request | ${:.6} |\n\n",
            stats.total_requests,
            stats.total_tokens,
            stats.total_cost,
            stats.average_tokens_per_request,
            stats.average_cost_per_request
        ));
        
        content.push_str("## Trends Analysis\n\n");
        content.push_str(&format!(
            "| Trend Metric | Value |\n\
             |--------------|-------|\n\
             | Overall Trend | {:?} |\n\
             | Cost Growth Rate | {:.1}% |\n\
             | Token Growth Rate | {:.1}% |\n\
             | Request Growth Rate | {:.1}% |\n",
            trends.overall_trend,
            trends.cost_trend.growth_rate,
            trends.token_trend.growth_rate,
            trends.request_trend.growth_rate
        ));
        
        self.write_output(&content, output_path)
    }

    // Placeholder implementations for other markdown outputs
    fn output_daily_report_markdown(
        &self,
        _summary: &DailySummary,
        _comparison: Option<(&DailySummary, &DailySummary)>,
        _output_path: Option<&str>,
    ) -> Result<()> {
        Ok(())
    }

    fn output_weekly_report_markdown(&self, _summaries: &[WeeklySummary], _output_path: Option<&str>) -> Result<()> {
        Ok(())
    }

    fn output_monthly_report_markdown(
        &self,
        _summary: &MonthlySummary,
        _comparison: Option<(&MonthlySummary, &MonthlySummary)>,
        _output_path: Option<&str>,
    ) -> Result<()> {
        Ok(())
    }

    fn output_session_list_markdown(&self, _sessions: &[Session], _output_path: Option<&str>) -> Result<()> {
        Ok(())
    }

    fn output_session_analysis_markdown(&self, _analysis: &SessionAnalysis, _output_path: Option<&str>) -> Result<()> {
        Ok(())
    }

    fn output_budget_status_markdown(&self, _budget: &BudgetInfo, _analysis: &BudgetAnalysis, _output_path: Option<&str>) -> Result<()> {
        Ok(())
    }

    fn output_model_stats_markdown(&self, _model_stats: &HashMap<String, ModelStats>, _output_path: Option<&str>) -> Result<()> {
        Ok(())
    }

    fn output_performance_stats_markdown(&self, _stats: &UsageStatistics, _output_path: Option<&str>) -> Result<()> {
        Ok(())
    }

    fn output_insights_markdown(&self, _insights: &[crate::analysis::insights::Insight], _output_path: Option<&str>) -> Result<()> {
        Ok(())
    }

    // HTML implementations (simplified - just return JSON for now)
    fn output_cost_breakdown_html(&self, breakdown: &DetailedCostBreakdown, output_path: Option<&str>) -> Result<()> {
        self.output_cost_breakdown_json(breakdown, output_path)
    }

    fn output_usage_stats_html(&self, stats: &UsageStatistics, output_path: Option<&str>) -> Result<()> {
        self.output_usage_stats_json(stats, output_path)
    }

    fn output_trends_html(&self, trends: &TrendAnalysis, output_path: Option<&str>) -> Result<()> {
        self.output_trends_json(trends, output_path)
    }

    fn output_comprehensive_analysis_html(
        &self,
        breakdown: &DetailedCostBreakdown,
        stats: &UsageStatistics,
        trends: &TrendAnalysis,
        output_path: Option<&str>,
    ) -> Result<()> {
        self.output_comprehensive_analysis_json(breakdown, stats, trends, output_path)
    }

    fn output_daily_report_html(
        &self,
        summary: &DailySummary,
        comparison: Option<(&DailySummary, &DailySummary)>,
        output_path: Option<&str>,
    ) -> Result<()> {
        self.output_daily_report_json(summary, comparison, output_path)
    }

    fn output_weekly_report_html(&self, summaries: &[WeeklySummary], output_path: Option<&str>) -> Result<()> {
        self.output_weekly_report_json(summaries, output_path)
    }

    fn output_monthly_report_html(
        &self,
        summary: &MonthlySummary,
        comparison: Option<(&MonthlySummary, &MonthlySummary)>,
        output_path: Option<&str>,
    ) -> Result<()> {
        self.output_monthly_report_json(summary, comparison, output_path)
    }

    fn output_session_list_html(&self, sessions: &[Session], output_path: Option<&str>) -> Result<()> {
        self.output_session_list_json(sessions, output_path)
    }

    fn output_session_analysis_html(&self, analysis: &SessionAnalysis, output_path: Option<&str>) -> Result<()> {
        self.output_session_analysis_json(analysis, output_path)
    }

    fn output_budget_status_html(&self, budget: &BudgetInfo, analysis: &BudgetAnalysis, output_path: Option<&str>) -> Result<()> {
        self.output_budget_status_json(budget, analysis, output_path)
    }

    fn output_model_stats_html(&self, model_stats: &HashMap<String, ModelStats>, output_path: Option<&str>) -> Result<()> {
        self.output_model_stats_json(model_stats, output_path)
    }

    fn output_performance_stats_html(&self, stats: &UsageStatistics, output_path: Option<&str>) -> Result<()> {
        self.output_performance_stats_json(stats, output_path)
    }

    fn output_insights_html(&self, insights: &[crate::analysis::insights::Insight], output_path: Option<&str>) -> Result<()> {
        self.output_insights_json(insights, output_path)
    }

    // Additional placeholder methods for missing output methods
    pub fn output_session_stats(&self, _stats: &SessionStatistics, _output_path: Option<&str>) -> Result<()> {
        Ok(())
    }

    pub fn output_time_stats(&self, _stats: &HashMap<u8, crate::analysis::statistics::ModelStats>, _output_path: Option<&str>) -> Result<()> {
        Ok(())
    }

    pub fn output_cost_stats(&self, _breakdown: &DetailedCostBreakdown, _output_path: Option<&str>) -> Result<()> {
        Ok(())
    }
}

// Placeholder structures for missing types
#[derive(Debug, Clone, serde::Serialize)]
pub struct SessionAnalysis {
    pub session_id: String,
    pub total_cost: f64,
    pub request_count: u32,
    pub total_tokens: u64,
    pub duration: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_formatter_creation() {
        let formatter = OutputFormatter::new(OutputFormat::Table);
        assert_eq!(formatter.format, OutputFormat::Table);
    }

    #[test]
    fn test_cost_breakdown_table_output() {
        let formatter = OutputFormatter::new(OutputFormat::Table);
        let breakdown = DetailedCostBreakdown::new();
        
        // This should not panic
        let result = formatter.output_cost_breakdown_table(&breakdown);
        assert!(result.is_ok());
    }

    #[test]
    fn test_usage_stats_table_output() {
        let formatter = OutputFormatter::new(OutputFormat::Table);
        let stats = StatisticsCalculator::calculate_usage_stats(&[]);
        
        let result = formatter.output_usage_stats_table(&stats);
        assert!(result.is_ok());
    }

    #[test]
    fn test_json_output() {
        let formatter = OutputFormatter::new(OutputFormat::Json);
        let breakdown = DetailedCostBreakdown::new();
        
        let result = formatter.output_cost_breakdown_json(&breakdown, None);
        assert!(result.is_ok());
    }
}