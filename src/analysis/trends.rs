//! Trends analysis module for ccusage-rs
//! 
//! This module provides trend analysis, forecasting, and pattern detection
//! for usage data over time.

use crate::data::models::*;
use crate::analysis::statistics::StatisticsCalculator;
use crate::error::Result;
use chrono::{DateTime, Utc, NaiveDate, Duration, Timelike, Datelike};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, BTreeMap};

/// Trend analysis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysisConfig {
    /// Number of days to include in trend analysis
    pub analysis_period_days: u32,
    
    /// Smoothing factor for moving averages
    pub smoothing_factor: f64,
    
    /// Enable anomaly detection
    pub enable_anomaly_detection: bool,
    
    /// Anomaly detection threshold (z-score)
    pub anomaly_threshold: f64,
    
    /// Enable forecasting
    pub enable_forecasting: bool,
    
    /// Forecast horizon in days
    pub forecast_horizon_days: u32,
}

impl Default for TrendAnalysisConfig {
    fn default() -> Self {
        Self {
            analysis_period_days: 30,
            smoothing_factor: 0.3,
            enable_anomaly_detection: true,
            anomaly_threshold: 2.5,
            enable_forecasting: true,
            forecast_horizon_days: 7,
        }
    }
}

/// Trend analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub overall_trend: TrendDirection,
    pub cost_trend: TrendMetrics,
    pub token_trend: TrendMetrics,
    pub request_trend: TrendMetrics,
    pub daily_patterns: DailyPatterns,
    pub weekly_patterns: WeeklyPatterns,
    pub anomalies: Vec<Anomaly>,
    pub forecast: Option<Forecast>,
    pub insights: Vec<String>,
}

/// Trend direction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
    Unknown,
}

/// Trend metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendMetrics {
    pub growth_rate: f64,
    pub moving_average: Vec<f64>,
    pub trend_line: Vec<(NaiveDate, f64)>,
    pub correlation_coefficient: f64,
    pub volatility: f64,
}

/// Daily patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyPatterns {
    pub peak_hours: Vec<u8>,
    pub low_hours: Vec<u8>,
    pub hourly_distribution: HashMap<u8, f64>,
    pub average_daily_pattern: Vec<f64>,
}

/// Weekly patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyPatterns {
    pub peak_days: Vec<u8>, // 0 = Monday, 6 = Sunday
    pub low_days: Vec<u8>,
    pub daily_distribution: HashMap<u8, f64>,
    pub weekend_vs_weekday: (f64, f64),
}

/// Anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub date: NaiveDate,
    pub value: f64,
    pub expected_value: f64,
    pub z_score: f64,
    pub anomaly_type: AnomalyType,
    pub severity: AnomalySeverity,
}

/// Anomaly types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnomalyType {
    Spike,
    Drop,
    Outlier,
    PatternViolation,
}

/// Anomaly severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Forecast results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Forecast {
    pub forecast_period: Vec<NaiveDate>,
    pub cost_forecast: Vec<f64>,
    pub token_forecast: Vec<f64>,
    pub request_forecast: Vec<f64>,
    pub confidence_intervals: Vec<(f64, f64)>,
    pub forecast_method: ForecastMethod,
}

/// Forecast methods
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ForecastMethod {
    LinearRegression,
    MovingAverage,
    ExponentialSmoothing,
    SeasonalDecomposition,
}

/// Trend analyzer
pub struct TrendAnalyzer {
    config: TrendAnalysisConfig,
}

impl TrendAnalyzer {
    /// Create a new trend analyzer
    pub fn new(config: TrendAnalysisConfig) -> Self {
        Self { config }
    }

    /// Create a trend analyzer with default configuration
    pub fn default() -> Self {
        Self::new(TrendAnalysisConfig::default())
    }

    /// Analyze trends in usage data
    pub fn analyze_trends(&self, records: &[UsageRecord]) -> Result<TrendAnalysis> {
        if records.is_empty() {
            return Ok(TrendAnalysis {
                overall_trend: TrendDirection::Unknown,
                cost_trend: TrendMetrics::default(),
                token_trend: TrendMetrics::default(),
                request_trend: TrendMetrics::default(),
                daily_patterns: DailyPatterns::default(),
                weekly_patterns: WeeklyPatterns::default(),
                anomalies: Vec::new(),
                forecast: None,
                insights: Vec::new(),
            });
        }

        // Group records by date
        let daily_data = self.group_records_by_date(records);
        let mut sorted_dates: Vec<NaiveDate> = daily_data.keys().cloned().collect();
        sorted_dates.sort();

        // Calculate trend metrics
        let cost_trend = self.calculate_trend_metrics(&daily_data, |records| {
            records.iter().map(|r| r.cost).sum()
        })?;

        let token_trend = self.calculate_trend_metrics(&daily_data, |records| {
            records.iter().map(|r| r.total_tokens() as f64).sum()
        })?;

        let request_trend = self.calculate_trend_metrics(&daily_data, |records| {
            records.len() as f64
        })?;

        // Determine overall trend direction
        let overall_trend = self.determine_overall_trend(&[&cost_trend, &token_trend, &request_trend]);

        // Analyze patterns
        let daily_patterns = self.analyze_daily_patterns(records);
        let weekly_patterns = self.analyze_weekly_patterns(records);

        // Detect anomalies
        let anomalies = if self.config.enable_anomaly_detection {
            self.detect_anomalies(&daily_data, &cost_trend)?
        } else {
            Vec::new()
        };

        // Generate forecast
        let forecast = if self.config.enable_forecasting {
            self.generate_forecast(&daily_data, &cost_trend)?
        } else {
            None
        };

        // Generate insights
        let insights = self.generate_insights(&overall_trend, &cost_trend, &daily_patterns, &anomalies);

        Ok(TrendAnalysis {
            overall_trend,
            cost_trend,
            token_trend,
            request_trend,
            daily_patterns,
            weekly_patterns,
            anomalies,
            forecast,
            insights,
        })
    }

    /// Group records by date
    fn group_records_by_date(&self, records: &[UsageRecord]) -> BTreeMap<NaiveDate, Vec<UsageRecord>> {
        let mut daily_data = BTreeMap::new();
        
        for record in records {
            let date = record.timestamp.date_naive();
            daily_data.entry(date).or_insert_with(Vec::new).push(record.clone());
        }
        
        daily_data
    }

    /// Calculate trend metrics for a specific metric
    fn calculate_trend_metrics<F>(&self, daily_data: &BTreeMap<NaiveDate, Vec<UsageRecord>>, metric_fn: F) -> Result<TrendMetrics>
    where
        F: Fn(&[UsageRecord]) -> f64,
    {
        let values: Vec<f64> = daily_data.values().map(|records| metric_fn(records)).collect();
        
        if values.len() < 2 {
            return Ok(TrendMetrics::default());
        }

        // Calculate growth rate
        let growth_rate = if values.len() >= 2 {
            let first = values[0];
            let last = values[values.len() - 1];
            if first != 0.0 {
                ((last - first) / first) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Calculate moving average
        let window_size = (values.len() as f64 * self.config.smoothing_factor).max(3.0) as usize;
        let moving_average = StatisticsCalculator::calculate_moving_average(&values, window_size);

        // Calculate trend line using simple linear regression
        let trend_line = self.calculate_trend_line(daily_data.keys().cloned().collect(), &values);

        // Calculate correlation coefficient
        let correlation_coefficient = StatisticsCalculator::calculate_correlation(
            &(0..values.len() as i32).map(|i| i as f64).collect::<Vec<_>>(),
            &values
        ).unwrap_or(0.0);

        // Calculate volatility (standard deviation of daily changes)
        let changes: Vec<f64> = values.windows(2).map(|w| w[1] - w[0]).collect();
        let volatility = if changes.is_empty() {
            0.0
        } else {
            let summary = StatisticsCalculator::calculate_summary(&changes);
            summary.standard_deviation
        };

        Ok(TrendMetrics {
            growth_rate,
            moving_average,
            trend_line,
            correlation_coefficient,
            volatility,
        })
    }

    /// Calculate trend line using simple linear regression
    fn calculate_trend_line(&self, dates: Vec<NaiveDate>, values: &[f64]) -> Vec<(NaiveDate, f64)> {
        if values.is_empty() {
            return Vec::new();
        }

        let n = values.len() as f64;
        let sum_x = (0..values.len()).map(|i| i as f64).sum::<f64>();
        let sum_y = values.iter().sum::<f64>();
        let sum_xy = values.iter().enumerate().map(|(i, &y)| i as f64 * y).sum::<f64>();
        let sum_x2 = (0..values.len()).map(|i| (i as f64).powi(2)).sum::<f64>();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;

        dates.into_iter().enumerate().map(|(i, date)| {
            let predicted = slope * i as f64 + intercept;
            (date, predicted)
        }).collect()
    }

    /// Determine overall trend direction
    fn determine_overall_trend(&self, trends: &[&TrendMetrics]) -> TrendDirection {
        let positive_trends = trends.iter().filter(|t| t.growth_rate > 5.0).count();
        let negative_trends = trends.iter().filter(|t| t.growth_rate < -5.0).count();

        if positive_trends > negative_trends {
            TrendDirection::Increasing
        } else if negative_trends > positive_trends {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        }
    }

    /// Analyze daily patterns
    fn analyze_daily_patterns(&self, records: &[UsageRecord]) -> DailyPatterns {
        let mut hourly_counts = [0u32; 24];
        let mut hourly_costs = [0.0; 24];
        let mut hourly_requests = [0u32; 24];

        for record in records {
            let hour = record.timestamp.hour() as usize;
            hourly_counts[hour] += 1;
            hourly_costs[hour] += record.cost;
            hourly_requests[hour] += 1;
        }

        // Find peak and low hours
        let max_count = hourly_counts.iter().max().copied().unwrap_or(0);
        let min_count = hourly_counts.iter().min().copied().unwrap_or(0);

        let peak_hours: Vec<u8> = hourly_counts
            .iter()
            .enumerate()
            .filter(|(_, &count)| count == max_count)
            .map(|(i, _)| i as u8)
            .collect();

        let low_hours: Vec<u8> = hourly_counts
            .iter()
            .enumerate()
            .filter(|(_, &count)| count == min_count)
            .map(|(i, _)| i as u8)
            .collect();

        // Calculate hourly distribution (percentage of total)
        let total_requests: u32 = hourly_requests.iter().sum();
        let hourly_distribution: HashMap<u8, f64> = hourly_counts
            .iter()
            .enumerate()
            .map(|(hour, &count)| {
                let percentage = if total_requests > 0 {
                    (count as f64 / total_requests as f64) * 100.0
                } else {
                    0.0
                };
                (hour as u8, percentage)
            })
            .collect();

        // Calculate average daily pattern
        let average_daily_pattern = hourly_counts.iter().map(|&count| count as f64).collect();

        DailyPatterns {
            peak_hours,
            low_hours,
            hourly_distribution,
            average_daily_pattern,
        }
    }

    /// Analyze weekly patterns
    fn analyze_weekly_patterns(&self, records: &[UsageRecord]) -> WeeklyPatterns {
        let mut weekday_counts = [0u32; 7]; // 0 = Monday, 6 = Sunday
        let mut weekday_costs = [0.0; 7];

        for record in records {
            let weekday = record.timestamp.weekday().num_days_from_monday() as usize;
            weekday_counts[weekday] += 1;
            weekday_costs[weekday] += record.cost;
        }

        // Find peak and low days
        let max_count = weekday_counts.iter().max().copied().unwrap_or(0);
        let min_count = weekday_counts.iter().min().copied().unwrap_or(0);

        let peak_days: Vec<u8> = weekday_counts
            .iter()
            .enumerate()
            .filter(|(_, &count)| count == max_count)
            .map(|(i, _)| i as u8)
            .collect();

        let low_days: Vec<u8> = weekday_counts
            .iter()
            .enumerate()
            .filter(|(_, &count)| count == min_count)
            .map(|(i, _)| i as u8)
            .collect();

        // Calculate daily distribution
        let total_requests: u32 = weekday_counts.iter().sum();
        let daily_distribution: HashMap<u8, f64> = weekday_counts
            .iter()
            .enumerate()
            .map(|(day, &count)| {
                let percentage = if total_requests > 0 {
                    (count as f64 / total_requests as f64) * 100.0
                } else {
                    0.0
                };
                (day as u8, percentage)
            })
            .collect();

        // Calculate weekend vs weekday usage
        let weekend_total: f64 = weekday_costs[5] + weekday_costs[6]; // Saturday + Sunday
        let weekday_total: f64 = weekday_costs[0..5].iter().sum();

        WeeklyPatterns {
            peak_days,
            low_days,
            daily_distribution,
            weekend_vs_weekday: (weekend_total, weekday_total),
        }
    }

    /// Detect anomalies in the data
    fn detect_anomalies(&self, daily_data: &BTreeMap<NaiveDate, Vec<UsageRecord>>, trend: &TrendMetrics) -> Result<Vec<Anomaly>> {
        let mut anomalies = Vec::new();
        let values: Vec<f64> = daily_data.values().map(|records| {
            records.iter().map(|r| r.cost).sum()
        }).collect();

        if values.len() < 3 {
            return Ok(anomalies);
        }

        let summary = StatisticsCalculator::calculate_summary(&values);

        for (i, (date, records)) in daily_data.iter().enumerate() {
            let value = records.iter().map(|r| r.cost).sum();
            let expected_value = if let Some(trend_point) = trend.trend_line.get(i) {
                trend_point.1
            } else {
                summary.mean
            };

            let z_score = StatisticsCalculator::calculate_z_score(value, expected_value, summary.standard_deviation);

            if z_score.abs() > self.config.anomaly_threshold {
                let anomaly_type = if z_score > 0.0 {
                    AnomalyType::Spike
                } else {
                    AnomalyType::Drop
                };

                let severity = if z_score.abs() > 4.0 {
                    AnomalySeverity::Critical
                } else if z_score.abs() > 3.0 {
                    AnomalySeverity::High
                } else if z_score.abs() > 2.5 {
                    AnomalySeverity::Medium
                } else {
                    AnomalySeverity::Low
                };

                anomalies.push(Anomaly {
                    date: *date,
                    value,
                    expected_value,
                    z_score,
                    anomaly_type,
                    severity,
                });
            }
        }

        Ok(anomalies)
    }

    /// Generate forecast
    fn generate_forecast(&self, daily_data: &BTreeMap<NaiveDate, Vec<UsageRecord>>, _trend: &TrendMetrics) -> Result<Option<Forecast>> {
        if daily_data.len() < 7 {
            return Ok(None);
        }

        let last_date = daily_data.keys().max().unwrap();
        let values: Vec<f64> = daily_data.values().map(|records| {
            records.iter().map(|r| r.cost).sum()
        }).collect();

        // Simple linear regression forecast
        let n = values.len() as f64;
        let sum_x = (0..values.len()).map(|i| i as f64).sum::<f64>();
        let sum_y = values.iter().sum::<f64>();
        let sum_xy = values.iter().enumerate().map(|(i, &y)| i as f64 * y).sum::<f64>();
        let sum_x2 = (0..values.len()).map(|i| (i as f64).powi(2)).sum::<f64>();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;

        // Generate forecast
        let forecast_period: Vec<NaiveDate> = (1..=self.config.forecast_horizon_days)
            .map(|i| *last_date + Duration::days(i as i64))
            .collect();

        let cost_forecast: Vec<f64> = (0..self.config.forecast_horizon_days)
            .map(|i| {
                let x = values.len() as f64 + i as f64;
                slope * x + intercept
            })
            .collect();

        // Simple confidence intervals (assuming normal distribution)
        let residuals: Vec<f64> = values.iter().enumerate().map(|(i, &y)| {
            let predicted = slope * i as f64 + intercept;
            y - predicted
        }).collect();

        let residual_std = StatisticsCalculator::calculate_summary(&residuals).standard_deviation;
        let confidence_intervals: Vec<(f64, f64)> = cost_forecast.iter().map(|&forecast| {
            let margin = 1.96 * residual_std; // 95% confidence
            (forecast - margin, forecast + margin)
        }).collect();

        // Generate token and request forecasts (simplified)
        let token_forecast: Vec<f64> = cost_forecast.iter().map(|&cost| cost * 1000.0).collect();
        let request_forecast: Vec<f64> = cost_forecast.iter().map(|&cost| cost / 0.015).collect();

        Ok(Some(Forecast {
            forecast_period,
            cost_forecast,
            token_forecast,
            request_forecast,
            confidence_intervals,
            forecast_method: ForecastMethod::LinearRegression,
        }))
    }

    /// Generate insights based on trend analysis
    fn generate_insights(&self, overall_trend: &TrendDirection, cost_trend: &TrendMetrics, daily_patterns: &DailyPatterns, anomalies: &[Anomaly]) -> Vec<String> {
        let mut insights = Vec::new();

        // Overall trend insights
        match overall_trend {
            TrendDirection::Increasing => {
                insights.push("Your usage costs are increasing. Consider reviewing your usage patterns.".to_string());
            }
            TrendDirection::Decreasing => {
                insights.push("Your usage costs are decreasing. Good cost management!".to_string());
            }
            TrendDirection::Stable => {
                insights.push("Your usage costs are stable. Predictable budgeting is possible.".to_string());
            }
            TrendDirection::Unknown => {}
        }

        // Growth rate insights
        if cost_trend.growth_rate.abs() > 20.0 {
            insights.push(format!(
                "High growth rate detected: {:.1}%. Consider investigating the cause.",
                cost_trend.growth_rate
            ));
        }

        // Volatility insights
        if cost_trend.volatility > cost_trend.moving_average.last().unwrap_or(&0.0) * 0.5 {
            insights.push("High volatility detected in daily costs. Usage patterns are inconsistent.".to_string());
        }

        // Daily pattern insights
        if let Some(peak_hour) = daily_patterns.peak_hours.first() {
            insights.push(format!(
                "Peak usage hour detected at {}:00. Consider scheduling tasks accordingly.",
                peak_hour
            ));
        }

        // Anomaly insights
        if !anomalies.is_empty() {
            let critical_anomalies = anomalies.iter().filter(|a| a.severity == AnomalySeverity::Critical).count();
            if critical_anomalies > 0 {
                insights.push(format!(
                    "{} critical anomalies detected. Review unusual usage patterns.",
                    critical_anomalies
                ));
            }
        }

        insights
    }
}

impl Default for TrendMetrics {
    fn default() -> Self {
        Self {
            growth_rate: 0.0,
            moving_average: Vec::new(),
            trend_line: Vec::new(),
            correlation_coefficient: 0.0,
            volatility: 0.0,
        }
    }
}

impl Default for DailyPatterns {
    fn default() -> Self {
        Self {
            peak_hours: Vec::new(),
            low_hours: Vec::new(),
            hourly_distribution: HashMap::new(),
            average_daily_pattern: Vec::new(),
        }
    }
}

impl Default for WeeklyPatterns {
    fn default() -> Self {
        Self {
            peak_days: Vec::new(),
            low_days: Vec::new(),
            daily_distribution: HashMap::new(),
            weekend_vs_weekday: (0.0, 0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    #[test]
    fn test_trend_analyzer_creation() {
        let analyzer = TrendAnalyzer::default();
        assert_eq!(analyzer.config.analysis_period_days, 30);
    }

    #[test]
    fn test_analyze_trends() {
        let analyzer = TrendAnalyzer::default();
        
        let records = vec![
            UsageRecord::new(Utc::now(), "claude-3-sonnet".to_string(), 1000, 500, 0.015),
            UsageRecord::new(Utc::now(), "claude-3-sonnet".to_string(), 2000, 1000, 0.030),
        ];
        
        let result = analyzer.analyze_trends(&records).unwrap();
        assert_eq!(result.overall_trend, TrendDirection::Increasing);
    }

    #[test]
    fn test_group_records_by_date() {
        let analyzer = TrendAnalyzer::default();
        
        let date1 = Utc::now().date_naive();
        let date2 = Utc::now().date_naive() + Duration::days(1);
        
        let records = vec![
            UsageRecord::new(
                DateTime::from_naive_utc_and_offset(
                    NaiveDateTime::new(date1, chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
                    chrono::Utc,
                ),
                "claude-3-sonnet".to_string(),
                1000,
                500,
                0.015,
            ),
            UsageRecord::new(
                DateTime::from_naive_utc_and_offset(
                    NaiveDateTime::new(date2, chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
                    chrono::Utc,
                ),
                "claude-3-sonnet".to_string(),
                2000,
                1000,
                0.030,
            ),
        ];
        
        let daily_data = analyzer.group_records_by_date(&records);
        assert_eq!(daily_data.len(), 2);
    }

    #[test]
    fn test_analyze_daily_patterns() {
        let analyzer = TrendAnalyzer::default();
        
        let records = vec![
            UsageRecord::new(
                DateTime::from_naive_utc_and_offset(
                    NaiveDateTime::new(
                        Utc::now().date_naive(),
                        chrono::NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
                    ),
                    chrono::Utc,
                ),
                "claude-3-sonnet".to_string(),
                1000,
                500,
                0.015,
            ),
            UsageRecord::new(
                DateTime::from_naive_utc_and_offset(
                    NaiveDateTime::new(
                        Utc::now().date_naive(),
                        chrono::NaiveTime::from_hms_opt(14, 0, 0).unwrap(),
                    ),
                    chrono::Utc,
                ),
                "claude-3-sonnet".to_string(),
                2000,
                1000,
                0.030,
            ),
        ];
        
        let patterns = analyzer.analyze_daily_patterns(&records);
        assert_eq!(patterns.peak_hours.len(), 1);
        assert_eq!(patterns.low_hours.len(), 1);
    }
}