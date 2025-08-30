//! Statistics module for ccusage-rs
//! 
//! This module provides statistical analysis functions for usage data.

use crate::data::models::*;
use crate::error::Result;
use chrono::{DateTime, Utc, NaiveDate, Timelike};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Statistical summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalSummary {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub median: f64,
    pub mode: Option<f64>,
    pub standard_deviation: f64,
    pub variance: f64,
    pub min: f64,
    pub max: f64,
    pub range: f64,
    pub percentiles: HashMap<u8, f64>,
}

impl StatisticalSummary {
    pub fn new() -> Self {
        Self {
            count: 0,
            sum: 0.0,
            mean: 0.0,
            median: 0.0,
            mode: None,
            standard_deviation: 0.0,
            variance: 0.0,
            min: 0.0,
            max: 0.0,
            range: 0.0,
            percentiles: HashMap::new(),
        }
    }
}

/// Usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStatistics {
    pub total_requests: usize,
    pub total_tokens: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cost: f64,
    pub average_tokens_per_request: f64,
    pub average_cost_per_request: f64,
    pub average_cost_per_token: f64,
    pub request_frequency_per_hour: f64,
    pub peak_usage_hour: Option<u8>,
    pub lowest_usage_hour: Option<u8>,
    pub hourly_distribution: HashMap<u8, u32>,
    pub daily_distribution: HashMap<NaiveDate, u32>,
    pub model_usage: HashMap<String, ModelStats>,
}

/// Model-specific statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStats {
    pub request_count: u32,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub average_tokens_per_request: f64,
    pub average_cost_per_request: f64,
    pub usage_percentage: f64,
}

/// Statistics calculator
pub struct StatisticsCalculator;

impl StatisticsCalculator {
    /// Calculate statistical summary for a numeric dataset
    pub fn calculate_summary(data: &[f64]) -> StatisticalSummary {
        if data.is_empty() {
            return StatisticalSummary::new();
        }

        let count = data.len();
        let sum = data.iter().sum();
        let mean = sum / count as f64;
        
        let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let range = max - min;
        
        // Calculate variance and standard deviation
        let mut variance: f64 = 0.0;
        for &x in data {
            let diff: f64 = x - mean;
            variance += diff.powi(2);
        }
        variance /= count as f64;
        let standard_deviation = variance.sqrt();
        
        // Calculate median
        let mut sorted_data = data.to_vec();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = if count % 2 == 0 {
            (sorted_data[count / 2 - 1] + sorted_data[count / 2]) / 2.0
        } else {
            sorted_data[count / 2]
        };
        
        // Calculate mode - use a different approach since f64 can't be HashMap key
        let mut mode = None;
        let mut max_count = 0;
        
        for &value in data {
            let count = data.iter().filter(|&&x| (x - value).abs() < f64::EPSILON).count();
            if count > max_count {
                max_count = count;
                mode = Some(value);
            }
        }
        
        // Calculate percentiles
        let mut percentiles = HashMap::new();
        for &p in &[25, 50, 75, 90, 95, 99] {
            let index = ((p as f64 / 100.0) * (count as f64 - 1.0)) as usize;
            percentiles.insert(p, sorted_data[index]);
        }
        
        StatisticalSummary {
            count,
            sum,
            mean,
            median,
            mode,
            standard_deviation,
            variance,
            min,
            max,
            range,
            percentiles,
        }
    }

    /// Calculate comprehensive usage statistics
    pub fn calculate_usage_stats(records: &[UsageRecord]) -> UsageStatistics {
        if records.is_empty() {
            return UsageStatistics {
                total_requests: 0,
                total_tokens: 0,
                total_input_tokens: 0,
                total_output_tokens: 0,
                total_cost: 0.0,
                average_tokens_per_request: 0.0,
                average_cost_per_request: 0.0,
                average_cost_per_token: 0.0,
                request_frequency_per_hour: 0.0,
                peak_usage_hour: None,
                lowest_usage_hour: None,
                hourly_distribution: HashMap::new(),
                daily_distribution: HashMap::new(),
                model_usage: HashMap::new(),
            };
        }

        let total_requests = records.len();
        let total_input_tokens = records.iter().map(|r| r.input_tokens as u64).sum();
        let total_output_tokens = records.iter().map(|r| r.output_tokens as u64).sum();
        let total_tokens = total_input_tokens + total_output_tokens;
        let total_cost = records.iter().map(|r| r.cost).sum();
        
        let average_tokens_per_request = if total_requests > 0 {
            total_tokens as f64 / total_requests as f64
        } else {
            0.0
        };
        
        let average_cost_per_request = if total_requests > 0 {
            total_cost / total_requests as f64
        } else {
            0.0
        };
        
        let average_cost_per_token = if total_tokens > 0 {
            total_cost / total_tokens as f64
        } else {
            0.0
        };

        // Calculate hourly distribution
        let mut hourly_distribution = HashMap::new();
        for record in records {
            let hour = record.timestamp.hour() as u8;
            *hourly_distribution.entry(hour).or_insert(0) += 1;
        }

        // Find peak and lowest usage hours
        let peak_usage_hour = hourly_distribution.iter()
            .max_by_key(|(_, count)| *count)
            .map(|(hour, _)| *hour);
        
        let lowest_usage_hour = hourly_distribution.iter()
            .min_by_key(|(_, count)| *count)
            .map(|(hour, _)| *hour);

        // Calculate daily distribution
        let mut daily_distribution = HashMap::new();
        for record in records {
            let date = record.timestamp.date_naive();
            *daily_distribution.entry(date).or_insert(0) += 1;
        }

        // Calculate request frequency per hour
        let time_span = if records.len() > 1 {
            let earliest = records.iter().map(|r| r.timestamp).min().unwrap();
            let latest = records.iter().map(|r| r.timestamp).max().unwrap();
            (latest - earliest).num_hours() as f64
        } else {
            1.0
        };
        
        let request_frequency_per_hour = if time_span > 0.0 {
            total_requests as f64 / time_span
        } else {
            0.0
        };

        // Calculate model usage statistics
        let model_usage = Self::calculate_model_stats(records);

        UsageStatistics {
            total_requests,
            total_tokens,
            total_input_tokens,
            total_output_tokens,
            total_cost,
            average_tokens_per_request,
            average_cost_per_request,
            average_cost_per_token,
            request_frequency_per_hour,
            peak_usage_hour,
            lowest_usage_hour,
            hourly_distribution,
            daily_distribution,
            model_usage,
        }
    }

    /// Calculate model-specific statistics
    fn calculate_model_stats(records: &[UsageRecord]) -> HashMap<String, ModelStats> {
        let mut model_stats = HashMap::new();
        let total_requests = records.len() as f64;
        
        // First pass: collect basic statistics
        for record in records {
            let stats = model_stats.entry(record.model.clone()).or_insert(ModelStats {
                request_count: 0,
                total_tokens: 0,
                total_cost: 0.0,
                average_tokens_per_request: 0.0,
                average_cost_per_request: 0.0,
                usage_percentage: 0.0,
            });
            
            stats.request_count += 1;
            stats.total_tokens += record.total_tokens() as u64;
            stats.total_cost += record.cost;
        }
        
        // Second pass: calculate averages and percentages
        for stats in model_stats.values_mut() {
            if stats.request_count > 0 {
                stats.average_tokens_per_request = stats.total_tokens as f64 / stats.request_count as f64;
                stats.average_cost_per_request = stats.total_cost / stats.request_count as f64;
            }
            
            if total_requests > 0.0 {
                stats.usage_percentage = (stats.request_count as f64 / total_requests) * 100.0;
            }
        }
        
        model_stats
    }

    /// Calculate correlation between two variables
    pub fn calculate_correlation(x: &[f64], y: &[f64]) -> Result<f64> {
        if x.len() != y.len() {
            return Err(crate::error::CcusageError::Validation(
                "Input arrays must have the same length".to_string()
            ));
        }
        
        if x.is_empty() {
            return Ok(0.0);
        }
        
        let n = x.len() as f64;
        let sum_x = x.iter().sum::<f64>();
        let sum_y = y.iter().sum::<f64>();
        let sum_xy = x.iter().zip(y.iter()).map(|(a, b)| a * b).sum::<f64>();
        let sum_x2 = x.iter().map(|a| a * a).sum::<f64>();
        let sum_y2 = y.iter().map(|b| b * b).sum::<f64>();
        
        let numerator = n * sum_xy - sum_x * sum_y;
        let denominator = ((n * sum_x2 - sum_x * sum_x) * (n * sum_y2 - sum_y * sum_y)).sqrt();
        
        if denominator == 0.0 {
            return Ok(0.0);
        }
        
        Ok(numerator / denominator)
    }

    /// Calculate moving average
    pub fn calculate_moving_average(data: &[f64], window_size: usize) -> Vec<f64> {
        if data.is_empty() || window_size == 0 {
            return Vec::new();
        }
        
        let mut result = Vec::new();
        let window_size = window_size.min(data.len());
        
        for i in 0..=data.len() - window_size {
            let window = &data[i..i + window_size];
            let average = window.iter().sum::<f64>() / window_size as f64;
            result.push(average);
        }
        
        result
    }

    /// Calculate growth rate
    pub fn calculate_growth_rate(current: f64, previous: f64) -> f64 {
        if previous == 0.0 {
            return 0.0;
        }
        ((current - previous) / previous) * 100.0
    }

    /// Calculate z-score for outlier detection
    pub fn calculate_z_score(value: f64, mean: f64, std_dev: f64) -> f64 {
        if std_dev == 0.0 {
            return 0.0;
        }
        (value - mean) / std_dev
    }

    /// Detect outliers using z-score method
    pub fn detect_outliers(data: &[f64], threshold: f64) -> Vec<usize> {
        if data.is_empty() {
            return Vec::new();
        }
        
        let summary = Self::calculate_summary(data);
        let mut outliers = Vec::new();
        
        for (i, &value) in data.iter().enumerate() {
            let z_score = Self::calculate_z_score(value, summary.mean, summary.standard_deviation);
            if z_score.abs() > threshold {
                outliers.push(i);
            }
        }
        
        outliers
    }

    /// Calculate confidence interval
    pub fn calculate_confidence_interval(data: &[f64], confidence_level: f64) -> Result<(f64, f64)> {
        if data.is_empty() {
            return Err(crate::error::CcusageError::Validation(
                "Data array is empty".to_string()
            ));
        }
        
        let summary = Self::calculate_summary(data);
        let n = data.len() as f64;
        
        // For simplicity, using normal distribution approximation
        // In practice, you might want to use t-distribution for small samples
        let z_score = match confidence_level {
            0.90 => 1.645,
            0.95 => 1.96,
            0.99 => 2.576,
            _ => 1.96, // Default to 95% confidence
        };
        
        let margin_of_error = z_score * (summary.standard_deviation / n.sqrt());
        let lower_bound = summary.mean - margin_of_error;
        let upper_bound = summary.mean + margin_of_error;
        
        Ok((lower_bound, upper_bound))
    }

    /// Calculate session statistics
    pub fn calculate_session_stats(sessions: &[Session]) -> SessionStatistics {
        if sessions.is_empty() {
            return SessionStatistics {
                total_sessions: 0,
                average_session_duration: 0.0,
                average_requests_per_session: 0.0,
                average_cost_per_session: 0.0,
                longest_session: None,
                shortest_session: None,
                most_expensive_session: None,
                least_expensive_session: None,
            };
        }

        let total_sessions = sessions.len();
        let total_duration = sessions.iter()
            .filter_map(|s| s.duration_seconds)
            .sum::<u64>() as f64;
        
        let total_requests = sessions.iter().map(|s| s.request_count).sum::<u32>() as f64;
        let total_cost = sessions.iter().map(|s| s.total_cost).sum::<f64>();
        
        let average_session_duration = if total_sessions > 0 {
            total_duration / total_sessions as f64
        } else {
            0.0
        };
        
        let average_requests_per_session = if total_sessions > 0 {
            total_requests / total_sessions as f64
        } else {
            0.0
        };
        
        let average_cost_per_session = if total_sessions > 0 {
            total_cost / total_sessions as f64
        } else {
            0.0
        };

        let longest_session = sessions.iter()
            .filter_map(|s| s.duration_seconds.map(|d| (s.id.clone(), d)))
            .max_by_key(|(_, d)| *d);
        
        let shortest_session = sessions.iter()
            .filter_map(|s| s.duration_seconds.map(|d| (s.id.clone(), d)))
            .min_by_key(|(_, d)| *d);
        
        let most_expensive_session = sessions.iter()
            .max_by(|a, b| a.total_cost.partial_cmp(&b.total_cost).unwrap_or(std::cmp::Ordering::Equal))
            .map(|s| (s.id.clone(), s.total_cost));
        
        let least_expensive_session = sessions.iter()
            .min_by(|a, b| a.total_cost.partial_cmp(&b.total_cost).unwrap_or(std::cmp::Ordering::Equal))
            .map(|s| (s.id.clone(), s.total_cost));

        SessionStatistics {
            total_sessions,
            average_session_duration,
            average_requests_per_session,
            average_cost_per_session,
            longest_session,
            shortest_session,
            most_expensive_session,
            least_expensive_session,
        }
    }
}

/// Session statistics
#[derive(Debug, Clone)]
pub struct SessionStatistics {
    pub total_sessions: usize,
    pub average_session_duration: f64,
    pub average_requests_per_session: f64,
    pub average_cost_per_session: f64,
    pub longest_session: Option<(String, u64)>,
    pub shortest_session: Option<(String, u64)>,
    pub most_expensive_session: Option<(String, f64)>,
    pub least_expensive_session: Option<(String, f64)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    #[test]
    fn test_calculate_summary() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let summary = StatisticsCalculator::calculate_summary(&data);
        
        assert_eq!(summary.count, 5);
        assert_eq!(summary.sum, 15.0);
        assert_eq!(summary.mean, 3.0);
        assert_eq!(summary.median, 3.0);
        assert_eq!(summary.min, 1.0);
        assert_eq!(summary.max, 5.0);
        assert_eq!(summary.range, 4.0);
    }

    #[test]
    fn test_calculate_usage_stats() {
        let records = vec![
            UsageRecord::new(Utc::now(), "claude-3-sonnet".to_string(), 1000, 500, 0.015),
            UsageRecord::new(Utc::now(), "claude-3-sonnet".to_string(), 2000, 1000, 0.030),
            UsageRecord::new(Utc::now(), "claude-3-opus".to_string(), 1500, 750, 0.045),
        ];
        
        let stats = StatisticsCalculator::calculate_usage_stats(&records);
        
        assert_eq!(stats.total_requests, 3);
        assert_eq!(stats.total_input_tokens, 4500);
        assert_eq!(stats.total_output_tokens, 2250);
        assert_eq!(stats.total_tokens, 6750);
        assert_eq!(stats.total_cost, 0.09);
        assert_eq!(stats.model_usage.len(), 2);
    }

    #[test]
    fn test_calculate_correlation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        
        let correlation = StatisticsCalculator::calculate_correlation(&x, &y).unwrap();
        assert!((correlation - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_moving_average() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let moving_avg = StatisticsCalculator::calculate_moving_average(&data, 3);
        
        assert_eq!(moving_avg, vec![2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_calculate_growth_rate() {
        let growth = StatisticsCalculator::calculate_growth_rate(150.0, 100.0);
        assert!((growth - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_detect_outliers() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 100.0];
        let outliers = StatisticsCalculator::detect_outliers(&data, 2.0);
        
        assert_eq!(outliers.len(), 1);
        assert_eq!(outliers[0], 4);
    }

    #[test]
    fn test_calculate_confidence_interval() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let (lower, upper) = StatisticsCalculator::calculate_confidence_interval(&data, 0.95).unwrap();
        
        assert!(lower < 3.0);
        assert!(upper > 3.0);
    }
}