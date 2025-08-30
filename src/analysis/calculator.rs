//! Cost calculator for ccusage-rs
//! 
//! This module handles cost calculations, pricing management, and
//! financial analysis of usage data.

use crate::data::models::*;
use crate::error::{Result, CcusageError};
use crate::output::SessionAnalysis;
use chrono::{DateTime, Utc, NaiveDate, Timelike, Datelike};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::cell::RefCell;

/// Cost calculator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostCalculatorConfig {
    /// Enable detailed cost breakdown
    pub enable_breakdown: bool,
    
    /// Include tax in calculations
    pub include_tax: bool,
    
    /// Tax rate (percentage)
    pub tax_rate: f64,
    
    /// Currency code
    pub currency: String,
    
    /// Enable cost optimization suggestions
    pub enable_optimization: bool,
}

impl Default for CostCalculatorConfig {
    fn default() -> Self {
        Self {
            enable_breakdown: true,
            include_tax: false,
            tax_rate: 0.0,
            currency: "USD".to_string(),
            enable_optimization: true,
        }
    }
}

/// Main cost calculator
pub struct CostCalculator {
    config: CostCalculatorConfig,
    pricing_data: HashMap<String, Vec<PricingInfo>>,
    cache: RefCell<HashMap<String, f64>>,
}

impl CostCalculator {
    /// Create a new cost calculator
    pub fn new(config: CostCalculatorConfig) -> Self {
        Self {
            config,
            pricing_data: HashMap::new(),
            cache: RefCell::new(HashMap::new()),
        }
    }

    /// Create a cost calculator with default configuration
    pub fn default() -> Self {
        Self::new(CostCalculatorConfig::default())
    }

    /// Load pricing data
    pub fn load_pricing_data(&mut self, pricing_data: Vec<PricingInfo>) -> Result<()> {
        for pricing in pricing_data {
            self.pricing_data
                .entry(pricing.model.clone())
                .or_insert_with(Vec::new)
                .push(pricing.clone());
        }

        // Sort pricing data by effective date
        for pricing_list in self.pricing_data.values_mut() {
            pricing_list.sort_by(|a, b| a.effective_date.cmp(&b.effective_date));
        }

        Ok(())
    }

    /// Calculate cost for a usage record
    pub fn calculate_cost(&self, record: &UsageRecord) -> Result<f64> {
        let cache_key = format!("{}:{}", record.model, record.timestamp.timestamp());
        let mut cache = self.cache.borrow_mut();
        
        // Check cache first
        if let Some(&cached_cost) = cache.get(&cache_key) {
            return Ok(cached_cost);
        }

        // Find applicable pricing
        let pricing = self.find_applicable_pricing(&record.model, record.timestamp)?;
        
        let cost = pricing.calculate_cost(record.input_tokens, record.output_tokens);
        
        // Apply tax if enabled
        let final_cost = if self.config.include_tax {
            cost * (1.0 + self.config.tax_rate / 100.0)
        } else {
            cost
        };

        // Cache the result
        cache.insert(cache_key, final_cost);

        Ok(final_cost)
    }

    /// Find applicable pricing for a model and date
    fn find_applicable_pricing(&self, model: &str, date: DateTime<Utc>) -> Result<&PricingInfo> {
        let pricing_list = self.pricing_data.get(model).ok_or_else(|| {
            CcusageError::Validation(format!("No pricing data found for model: {}", model))
        })?;

        // Find the most recent pricing that is valid for the date
        let applicable_pricing = pricing_list
            .iter()
            .rev()
            .find(|pricing| pricing.is_valid_for_date(date))
            .ok_or_else(|| {
                CcusageError::Validation(format!(
                    "No applicable pricing found for model {} at date {}",
                    model, date
                ))
            })?;

        Ok(applicable_pricing)
    }

    /// Calculate total cost for multiple records
    pub fn calculate_total_cost(&self, records: &[UsageRecord]) -> Result<f64> {
        let mut total_cost = 0.0;
        
        for record in records {
            total_cost += self.calculate_cost(record)?;
        }

        Ok(total_cost)
    }

    /// Calculate cost breakdown by model
    pub fn calculate_cost_by_model(&self, records: &[UsageRecord]) -> Result<HashMap<String, f64>> {
        let mut cost_by_model = HashMap::new();
        
        for record in records {
            let cost = self.calculate_cost(record)?;
            *cost_by_model.entry(record.model.clone()).or_insert(0.0) += cost;
        }

        Ok(cost_by_model)
    }

    /// Calculate cost breakdown by date
    pub fn calculate_cost_by_date(&self, records: &[UsageRecord]) -> Result<HashMap<NaiveDate, f64>> {
        let mut cost_by_date = HashMap::new();
        
        for record in records {
            let cost = self.calculate_cost(record)?;
            let date = record.timestamp.date_naive();
            *cost_by_date.entry(date).or_insert(0.0) += cost;
        }

        Ok(cost_by_date)
    }

    /// Calculate cost breakdown by session
    pub fn calculate_cost_by_session(&self, records: &[UsageRecord]) -> Result<HashMap<String, f64>> {
        let mut cost_by_session = HashMap::new();
        
        for record in records {
            if let Some(session_id) = &record.session_id {
                let cost = self.calculate_cost(record)?;
                *cost_by_session.entry(session_id.clone()).or_insert(0.0) += cost;
            }
        }

        Ok(cost_by_session)
    }

    /// Calculate cost breakdown by user
    pub fn calculate_cost_by_user(&self, records: &[UsageRecord]) -> Result<HashMap<String, f64>> {
        let mut cost_by_user = HashMap::new();
        
        for record in records {
            if let Some(user_id) = &record.user_id {
                let cost = self.calculate_cost(record)?;
                *cost_by_user.entry(user_id.clone()).or_insert(0.0) += cost;
            }
        }

        Ok(cost_by_user)
    }

    /// Calculate detailed cost breakdown
    pub fn calculate_detailed_breakdown(&self, records: &[UsageRecord]) -> Result<DetailedCostBreakdown> {
        let mut breakdown = DetailedCostBreakdown::new();
        
        breakdown.total_cost = self.calculate_total_cost(records)?;
        breakdown.total_records = records.len();
        breakdown.total_input_tokens = records.iter().map(|r| r.input_tokens).sum();
        breakdown.total_output_tokens = records.iter().map(|r| r.output_tokens).sum();
        breakdown.total_tokens = breakdown.total_input_tokens + breakdown.total_output_tokens;
        
        breakdown.cost_by_model = self.calculate_cost_by_model(records)?;
        breakdown.cost_by_date = self.calculate_cost_by_date(records)?;
        breakdown.cost_by_session = self.calculate_cost_by_session(records)?;
        breakdown.cost_by_user = self.calculate_cost_by_user(records)?;
        
        // Calculate averages
        if breakdown.total_records > 0 {
            breakdown.avg_cost_per_request = breakdown.total_cost / breakdown.total_records as f64;
            breakdown.avg_tokens_per_request = breakdown.total_tokens as f64 / breakdown.total_records as f64;
        }
        
        if breakdown.total_tokens > 0 {
            breakdown.avg_cost_per_token = breakdown.total_cost / breakdown.total_tokens as f64;
        }
        
        // Calculate model efficiency
        for (model, &cost) in &breakdown.cost_by_model {
            let model_records: Vec<_> = records
                .iter()
                .filter(|r| r.model == *model)
                .collect();
            
            let model_tokens: u32 = model_records.iter().map(|r| r.total_tokens()).sum();
            let efficiency = if model_tokens > 0 {
                cost / model_tokens as f64
            } else {
                0.0
            };
            
            breakdown.model_efficiency.insert(model.clone(), efficiency);
        }
        
        // Find most expensive model
        breakdown.most_expensive_model = breakdown.cost_by_model
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(model, _)| model.clone());
        
        // Find most cost-effective model
        breakdown.most_cost_effective_model = breakdown.model_efficiency
            .iter()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(model, _)| model.clone());
        
        Ok(breakdown)
    }

    /// Calculate cost projection
    pub fn calculate_cost_projection(&self, records: &[UsageRecord], days_to_project: u32) -> Result<CostProjection> {
        if records.is_empty() {
            return Ok(CostProjection::default());
        }

        // Calculate daily average cost
        let daily_breakdown = self.calculate_cost_by_date(records)?;
        let daily_avg_cost = daily_breakdown.values().sum::<f64>() / daily_breakdown.len() as f64;
        
        // Calculate trend
        let mut costs: Vec<_> = daily_breakdown.values().cloned().collect();
        costs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        let trend = if costs.len() >= 2 {
            let first_half = &costs[..costs.len() / 2];
            let second_half = &costs[costs.len() / 2..];
            
            let first_avg = first_half.iter().sum::<f64>() / first_half.len() as f64;
            let second_avg = second_half.iter().sum::<f64>() / second_half.len() as f64;
            
            if first_avg > 0.0 {
                ((second_avg - first_avg) / first_avg) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Calculate projection
        let projected_cost = daily_avg_cost * days_to_project as f64;
        let projected_with_trend = projected_cost * (1.0 + trend / 100.0);

        Ok(CostProjection {
            daily_average_cost: daily_avg_cost,
            projected_cost,
            projected_cost_with_trend: projected_with_trend,
            trend_percentage: trend,
            days_projected: days_to_project,
            currency: self.config.currency.clone(),
        })
    }

    /// Calculate budget analysis
    pub fn calculate_budget_analysis(&self, records: &[UsageRecord], budget: &BudgetInfo) -> Result<BudgetAnalysis> {
        let total_cost = self.calculate_total_cost(records)?;
        let current_date = Utc::now().date_naive();
        
        // Get first and last record dates
        let (first_date, last_date) = records.iter()
            .fold((None, None), |(first, last), record| {
                let date = record.timestamp.date_naive();
                (
                    first.map_or(Some(date), |f: NaiveDate| Some(f.min(date))),
                    last.map_or(Some(date), |l: NaiveDate| Some(l.max(date)))
                )
            });

        let usage_period_days = if let (Some(first), Some(last)) = (first_date, last_date) {
            (last - first).num_days() as u32
        } else {
            0
        };

        let budget_usage_percentage = if budget.monthly_limit > 0.0 {
            (total_cost / budget.monthly_limit) * 100.0
        } else {
            0.0
        };

        let daily_average_cost = if usage_period_days > 0 {
            total_cost / usage_period_days as f64
        } else {
            0.0
        };

        let days_remaining_in_month = Self::days_remaining_in_month(current_date);
        let projected_monthly_cost = total_cost + (daily_average_cost * days_remaining_in_month as f64);

        let is_budget_exceeded = total_cost > budget.monthly_limit;
        let is_warning_exceeded = budget.is_warning_exceeded(total_cost);
        let is_alert_exceeded = budget.is_alert_exceeded(total_cost);

        Ok(BudgetAnalysis {
            budget_limit: budget.monthly_limit,
            current_usage: total_cost,
            budget_usage_percentage,
            is_budget_exceeded,
            is_warning_exceeded,
            is_alert_exceeded,
            daily_average_cost,
            projected_monthly_cost,
            usage_period_days,
            days_remaining_in_month,
            currency: budget.currency.clone(),
        })
    }

    /// Generate cost optimization suggestions
    pub fn generate_optimization_suggestions(&self, records: &[UsageRecord]) -> Result<Vec<OptimizationSuggestion>> {
        if !self.config.enable_optimization {
            return Ok(Vec::new());
        }

        let mut suggestions = Vec::new();
        
        // Analyze model usage patterns
        let cost_by_model = self.calculate_cost_by_model(records)?;
        let _model_usage: HashMap<String, u32> = records
            .iter()
            .fold(HashMap::new(), |mut acc, record| {
                *acc.entry(record.model.clone()).or_insert(0) += 1;
                acc
            });

        // Suggest switching to cheaper models for simple tasks
        for (model, cost) in cost_by_model {
            if model.contains("claude-3-opus") && cost > 10.0 {
                suggestions.push(OptimizationSuggestion {
                    suggestion_type: OptimizationType::ModelSwitch,
                    title: "Consider using Claude 3 Sonnet for simpler tasks".to_string(),
                    description: format!(
                        "You've spent ${:.2} on Claude 3 Opus. For simpler tasks, Claude 3 Sonnet could save you money.",
                        cost
                    ),
                    potential_savings: cost * 0.6, // Estimate 60% savings
                    priority: Priority::Medium,
                });
            }
        }

        // Suggest batching requests
        let short_sessions: Vec<_> = records
            .iter()
            .filter(|r| {
                if let Some(session_id) = &r.session_id {
                    // Count records per session
                    records.iter()
                        .filter(|r2| r2.session_id.as_ref() == Some(session_id))
                        .count() < 3
                } else {
                    false
                }
            })
            .collect();

        if !short_sessions.is_empty() {
            let cost = short_sessions.iter()
                .map(|r| self.calculate_cost(r).unwrap_or(0.0))
                .sum::<f64>();
            
            suggestions.push(OptimizationSuggestion {
                suggestion_type: OptimizationType::Batching,
                title: "Batch similar requests together".to_string(),
                description: format!(
                    "You have {} short sessions that could be batched. Batching could reduce overhead costs.",
                    short_sessions.len()
                ),
                potential_savings: cost * 0.1, // Estimate 10% savings
                priority: Priority::Low,
            });
        }

        // Suggest caching results for repeated requests
        let repeated_requests = Self::find_repeated_requests(records);
        if !repeated_requests.is_empty() {
            suggestions.push(OptimizationSuggestion {
                suggestion_type: OptimizationType::Caching,
                title: "Implement result caching for repeated requests".to_string(),
                description: format!(
                    "Found {} potentially repeated requests that could benefit from caching.",
                    repeated_requests.len()
                ),
                potential_savings: repeated_requests.iter()
                    .map(|&(_, cost)| cost)
                    .sum::<f64>(),
                priority: Priority::High,
            });
        }

        Ok(suggestions)
    }

    /// Find potentially repeated requests
    fn find_repeated_requests(records: &[UsageRecord]) -> Vec<(String, f64)> {
        let mut request_patterns = HashMap::new();
        
        for record in records {
            // Simple pattern matching based on model and token counts
            let pattern = format!("{}:{}:{}", record.model, record.input_tokens, record.output_tokens);
            let cost = record.cost; // Use pre-calculated cost
            
            let count = request_patterns.entry(pattern.clone()).or_insert(0);
            *count += 1;
            
            if *count > 1 {
                // This is a repeated pattern
                return vec![(pattern, cost)];
            }
        }
        
        Vec::new()
    }

    /// Calculate days remaining in month
    fn days_remaining_in_month(_date: NaiveDate) -> u32 {
        // TODO: Fix chrono API usage
        30
    }

    /// Clear cost cache
    pub fn clear_cache(&self) {
        self.cache.borrow_mut().clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        let cache = self.cache.borrow();
        CacheStats {
            cache_size: cache.len(),
            hit_count: 0, // Would need to track this separately
        }
    }

    /// Calculate daily summary
    pub fn calculate_daily_summary(&self, records: &[UsageRecord]) -> Result<DailySummary> {
        if records.is_empty() {
            return Err(CcusageError::Validation("No records provided for daily summary".to_string()));
        }

        let date = records[0].timestamp.date_naive();
        let mut summary = DailySummary::new(date);

        // Group records by hour for peak hour calculation
        let mut hour_counts = [0u32; 24];
        let mut sessions = std::collections::HashSet::new();

        for record in records {
            // Calculate cost if not already calculated
            let cost = if record.cost > 0.0 {
                record.cost
            } else {
                self.calculate_cost(record)?
            };

            // Update summary fields
            summary.total_cost += cost;
            summary.total_input_tokens += record.input_tokens;
            summary.total_output_tokens += record.output_tokens;
            summary.request_count += 1;

            // Track sessions
            if let Some(session_id) = &record.session_id {
                sessions.insert(session_id);
            }

            // Track hour usage
            let hour = record.timestamp.hour() as usize;
            hour_counts[hour] += 1;

            // Update model breakdown
            let model_usage = summary.model_breakdown
                .entry(record.model.clone())
                .or_insert_with(|| ModelUsage::new(record.model.clone()));
            
            // Create a temporary record with the calculated cost
            let mut temp_record = record.clone();
            temp_record.cost = cost;
            model_usage.add_record(&temp_record);
        }

        // Calculate derived fields
        summary.session_count = sessions.len() as u32;
        summary.calculate_avg_cost();

        // Find peak hour
        summary.peak_hour = hour_counts
            .iter()
            .enumerate()
            .max_by_key(|(_, count)| *count)
            .map(|(hour, _)| hour as u8);

        // Find most used model
        summary.most_used_model = summary.model_breakdown
            .iter()
            .max_by_key(|(_, usage)| usage.request_count)
            .map(|(model, _)| model.clone());

        Ok(summary)
    }

    /// Calculate weekly summary
    pub fn calculate_weekly_summary(&self, records: &[UsageRecord]) -> Result<WeeklySummary> {
        if records.is_empty() {
            return Err(CcusageError::Validation("No records provided for weekly summary".to_string()));
        }

        // Determine week start (Monday)
        let first_date = records.iter()
            .map(|r| r.timestamp.date_naive())
            .min()
            .unwrap();
        
        // TODO: Fix weekday() API usage
        let week_start = first_date;
        let week_end = week_start + chrono::Duration::days(6);

        let mut summary = WeeklySummary::new(week_start);

        // Group records by day
        let mut daily_records: std::collections::HashMap<NaiveDate, Vec<UsageRecord>> = std::collections::HashMap::new();
        let mut sessions = std::collections::HashSet::new();

        for record in records {
            let date = record.timestamp.date_naive();
            daily_records.entry(date).or_insert_with(Vec::new).push(record.clone());

            if let Some(session_id) = &record.session_id {
                sessions.insert(session_id);
            }
        }

        // Create daily summaries
        for (date, day_records) in daily_records {
            let daily_summary = self.calculate_daily_summary(&day_records)?;
            summary.add_daily_summary(daily_summary);
        }

        summary.session_count = sessions.len() as u32;
        summary.calculate_avg_daily_cost();

        Ok(summary)
    }

    /// Calculate monthly summary
    pub fn calculate_monthly_summary(&self, records: &[UsageRecord]) -> Result<MonthlySummary> {
        if records.is_empty() {
            return Err(CcusageError::Validation("No records provided for monthly summary".to_string()));
        }

        let first_date = records.iter()
            .map(|r| r.timestamp.date_naive())
            .min()
            .unwrap();
        
        let year = first_date.year() as u32;
        let month = first_date.month() as u32;

        let mut summary = MonthlySummary::new(year, month);

        // Group records by week
        let mut weekly_records: std::collections::HashMap<NaiveDate, Vec<UsageRecord>> = std::collections::HashMap::new();
        let mut sessions = std::collections::HashSet::new();

        for record in records {
            let date = record.timestamp.date_naive();
            // TODO: Fix weekday() API usage
            let week_start = date;
            weekly_records.entry(week_start).or_insert_with(Vec::new).push(record.clone());

            if let Some(session_id) = &record.session_id {
                sessions.insert(session_id);
            }
        }

        // Create weekly summaries
        for (week_start, week_records) in weekly_records {
            let weekly_summary = self.calculate_weekly_summary(&week_records)?;
            summary.add_weekly_summary(weekly_summary);
        }

        summary.session_count = sessions.len() as u32;
        summary.calculate_avg_weekly_cost();

        Ok(summary)
    }

    /// Calculate session analysis
    pub fn calculate_session_analysis(&self, records: &[UsageRecord]) -> Result<SessionAnalysis> {
        if records.is_empty() {
            return Err(CcusageError::Validation("No records provided for session analysis".to_string()));
        }

        let session_id = records[0].session_id.as_ref()
            .ok_or_else(|| CcusageError::Validation("Session ID missing from records".to_string()))?;

        let mut total_cost = 0.0;
        let mut total_tokens = 0u64;
        let mut start_time = None;
        let mut end_time = None;

        for record in records {
            // Calculate cost if not already calculated
            let cost = if record.cost > 0.0 {
                record.cost
            } else {
                self.calculate_cost(record)?
            };

            total_cost += cost;
            total_tokens += record.total_tokens() as u64;

            // Track time range
            if start_time.is_none() || record.timestamp < start_time.unwrap() {
                start_time = Some(record.timestamp);
            }
            if end_time.is_none() || record.timestamp > end_time.unwrap() {
                end_time = Some(record.timestamp);
            }
        }

        let duration = if let (Some(start), Some(end)) = (start_time, end_time) {
            (end - start).num_seconds() as u64
        } else {
            0
        };

        Ok(SessionAnalysis {
            session_id: session_id.clone(),
            total_cost,
            request_count: records.len() as u32,
            total_tokens,
            duration,
        })
    }
}

/// Detailed cost breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedCostBreakdown {
    pub total_cost: f64,
    pub total_records: usize,
    pub total_input_tokens: u32,
    pub total_output_tokens: u32,
    pub total_tokens: u32,
    pub avg_cost_per_request: f64,
    pub avg_tokens_per_request: f64,
    pub avg_cost_per_token: f64,
    pub cost_by_model: HashMap<String, f64>,
    pub cost_by_date: HashMap<NaiveDate, f64>,
    pub cost_by_session: HashMap<String, f64>,
    pub cost_by_user: HashMap<String, f64>,
    pub model_efficiency: HashMap<String, f64>,
    pub most_expensive_model: Option<String>,
    pub most_cost_effective_model: Option<String>,
}

impl DetailedCostBreakdown {
    pub fn new() -> Self {
        Self {
            total_cost: 0.0,
            total_records: 0,
            total_input_tokens: 0,
            total_output_tokens: 0,
            total_tokens: 0,
            avg_cost_per_request: 0.0,
            avg_tokens_per_request: 0.0,
            avg_cost_per_token: 0.0,
            cost_by_model: HashMap::new(),
            cost_by_date: HashMap::new(),
            cost_by_session: HashMap::new(),
            cost_by_user: HashMap::new(),
            model_efficiency: HashMap::new(),
            most_expensive_model: None,
            most_cost_effective_model: None,
        }
    }
}

/// Cost projection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostProjection {
    pub daily_average_cost: f64,
    pub projected_cost: f64,
    pub projected_cost_with_trend: f64,
    pub trend_percentage: f64,
    pub days_projected: u32,
    pub currency: String,
}

impl Default for CostProjection {
    fn default() -> Self {
        Self {
            daily_average_cost: 0.0,
            projected_cost: 0.0,
            projected_cost_with_trend: 0.0,
            trend_percentage: 0.0,
            days_projected: 0,
            currency: "USD".to_string(),
        }
    }
}

/// Budget analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAnalysis {
    pub budget_limit: f64,
    pub current_usage: f64,
    pub budget_usage_percentage: f64,
    pub is_budget_exceeded: bool,
    pub is_warning_exceeded: bool,
    pub is_alert_exceeded: bool,
    pub daily_average_cost: f64,
    pub projected_monthly_cost: f64,
    pub usage_period_days: u32,
    pub days_remaining_in_month: u32,
    pub currency: String,
}

/// Optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    pub suggestion_type: OptimizationType,
    pub title: String,
    pub description: String,
    pub potential_savings: f64,
    pub priority: Priority,
}

/// Optimization types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OptimizationType {
    ModelSwitch,
    Batching,
    Caching,
    TokenOptimization,
    TimingOptimization,
}

/// Priority levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub cache_size: usize,
    pub hit_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    #[test]
    fn test_cost_calculator_creation() {
        let calculator = CostCalculator::default();
        assert_eq!(calculator.config.currency, "USD");
    }

    #[test]
    fn test_calculate_cost_with_pricing() {
        let mut calculator = CostCalculator::default();
        
        let pricing = PricingInfo {
            model: "claude-3-sonnet".to_string(),
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.015,
            currency: "USD".to_string(),
            effective_date: Utc::now(),
            is_active: true,
        };
        
        calculator.load_pricing_data(vec![pricing]).unwrap();
        
        let record = UsageRecord::new(
            Utc::now(),
            "claude-3-sonnet".to_string(),
            1000,
            500,
            0.015,
        );
        
        let cost = calculator.calculate_cost(&record).unwrap();
        assert_eq!(cost, 0.0105); // 0.003 * 1 + 0.015 * 0.5
    }

    #[test]
    fn test_calculate_total_cost() {
        let mut calculator = CostCalculator::default();
        
        let pricing = PricingInfo {
            model: "claude-3-sonnet".to_string(),
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.015,
            currency: "USD".to_string(),
            effective_date: Utc::now(),
            is_active: true,
        };
        
        calculator.load_pricing_data(vec![pricing]).unwrap();
        
        let records = vec![
            UsageRecord::new(Utc::now(), "claude-3-sonnet".to_string(), 1000, 500, 0.015),
            UsageRecord::new(Utc::now(), "claude-3-sonnet".to_string(), 2000, 1000, 0.030),
        ];
        
        let total_cost = calculator.calculate_total_cost(&records).unwrap();
        assert_eq!(total_cost, 0.0315); // 0.0105 + 0.021
    }

    #[test]
    fn test_cost_projection() {
        let calculator = CostCalculator::default();
        
        let records = vec![
            UsageRecord::new(Utc::now(), "claude-3-sonnet".to_string(), 1000, 500, 0.015),
            UsageRecord::new(Utc::now(), "claude-3-sonnet".to_string(), 1000, 500, 0.015),
        ];
        
        let projection = calculator.calculate_cost_projection(&records, 30).unwrap();
        assert_eq!(projection.days_projected, 30);
        assert!(projection.daily_average_cost > 0.0);
    }

    #[test]
    fn test_budget_analysis() {
        let calculator = CostCalculator::default();
        
        let budget = BudgetInfo::new(100.0, "USD".to_string());
        
        let records = vec![
            UsageRecord::new(Utc::now(), "claude-3-sonnet".to_string(), 1000, 500, 0.015),
        ];
        
        let analysis = calculator.calculate_budget_analysis(&records, &budget).unwrap();
        assert_eq!(analysis.budget_limit, 100.0);
        assert!(!analysis.is_budget_exceeded);
    }

    #[test]
    fn test_optimization_suggestions() {
        let calculator = CostCalculator::default();
        
        let records = vec![
            UsageRecord::new(Utc::now(), "claude-3-opus".to_string(), 1000, 500, 0.045),
        ];
        
        let suggestions = calculator.generate_optimization_suggestions(&records).unwrap();
        assert!(!suggestions.is_empty());
    }
}