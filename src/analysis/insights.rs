//! Insights module for ccusage-rs
//! 
//! This module provides intelligent insights, recommendations, and
//! actionable intelligence based on usage data analysis.

use crate::data::models::*;
use crate::analysis::{calculator::*, statistics::*, trends::*};
use crate::error::Result;
use chrono::{DateTime, Utc, NaiveDate, Timelike};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Insight types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InsightType {
    CostOptimization,
    UsagePattern,
    AnomalyDetection,
    TrendAnalysis,
    BudgetAlert,
    Performance,
    Recommendation,
    Warning,
}

/// Insight severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InsightSeverity {
    Informational,
    Low,
    Medium,
    High,
    Critical,
}

/// Insight categories
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InsightCategory {
    Cost,
    Usage,
    Performance,
    Security,
    Compliance,
    Optimization,
}

/// Insight data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    pub id: String,
    pub insight_type: InsightType,
    pub severity: InsightSeverity,
    pub category: InsightCategory,
    pub title: String,
    pub description: String,
    pub recommendation: String,
    pub impact: ImpactAssessment,
    pub confidence: f64, // 0.0 to 1.0
    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Impact assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactAssessment {
    pub potential_savings: Option<f64>,
    pub potential_efficiency_gain: Option<f64>,
    pub risk_level: RiskLevel,
    pub implementation_difficulty: DifficultyLevel,
}

/// Risk levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RiskLevel {
    None,
    Low,
    Medium,
    High,
}

/// Difficulty levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DifficultyLevel {
    Easy,
    Moderate,
    Hard,
    Expert,
}

/// Insights engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsEngineConfig {
    /// Enable cost optimization insights
    pub enable_cost_insights: bool,
    
    /// Enable usage pattern insights
    pub enable_usage_insights: bool,
    
    /// Enable anomaly detection insights
    pub enable_anomaly_insights: bool,
    
    /// Enable trend analysis insights
    pub enable_trend_insights: bool,
    
    /// Enable budget alert insights
    pub enable_budget_insights: bool,
    
    /// Minimum confidence threshold for insights
    pub min_confidence: f64,
    
    /// Maximum number of insights to generate
    pub max_insights: usize,
}

impl Default for InsightsEngineConfig {
    fn default() -> Self {
        Self {
            enable_cost_insights: true,
            enable_usage_insights: true,
            enable_anomaly_insights: true,
            enable_trend_insights: true,
            enable_budget_insights: true,
            min_confidence: 0.7,
            max_insights: 20,
        }
    }
}

/// Insights engine
pub struct InsightsEngine {
    config: InsightsEngineConfig,
}

impl InsightsEngine {
    /// Create a new insights engine
    pub fn new(config: InsightsEngineConfig) -> Self {
        Self { config }
    }

    /// Create an insights engine with default configuration
    pub fn default() -> Self {
        Self::new(InsightsEngineConfig::default())
    }

    /// Generate insights from usage data
    pub fn generate_insights(&self, records: &[UsageRecord], budget: Option<&BudgetInfo>) -> Result<Vec<Insight>> {
        let mut insights = Vec::new();

        if records.is_empty() {
            return Ok(insights);
        }

        // Generate cost optimization insights
        if self.config.enable_cost_insights {
            insights.extend(self.generate_cost_insights(records)?);
        }

        // Generate usage pattern insights
        if self.config.enable_usage_insights {
            insights.extend(self.generate_usage_pattern_insights(records)?);
        }

        // Generate trend analysis insights
        if self.config.enable_trend_insights {
            insights.extend(self.generate_trend_insights(records)?);
        }

        // Generate budget alert insights
        if self.config.enable_budget_insights {
            if let Some(budget) = budget {
                insights.extend(self.generate_budget_insights(records, budget)?);
            }
        }

        // Generate performance insights
        insights.extend(self.generate_performance_insights(records)?);

        // Filter insights by confidence and limit count
        insights = insights
            .into_iter()
            .filter(|insight| insight.confidence >= self.config.min_confidence)
            .take(self.config.max_insights)
            .collect();

        // Sort insights by severity and confidence
        insights.sort_by(|a, b| {
            let severity_cmp = self.severity_priority(&b.severity).cmp(&self.severity_priority(&a.severity));
            if severity_cmp == std::cmp::Ordering::Equal {
                b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal)
            } else {
                severity_cmp
            }
        });

        Ok(insights)
    }

    /// Generate cost optimization insights
    fn generate_cost_insights(&self, records: &[UsageRecord]) -> Result<Vec<Insight>> {
        let mut insights = Vec::new();

        // Analyze cost by model
        let cost_by_model: HashMap<String, f64> = records
            .iter()
            .fold(HashMap::new(), |mut acc, record| {
                *acc.entry(record.model.clone()).or_insert(0.0) += record.cost;
                acc
            });

        let total_cost = cost_by_model.values().sum::<f64>();
        
        // Find expensive models
        for (model, cost) in cost_by_model {
            let cost_percentage = (cost / total_cost) * 100.0;
            
            if cost_percentage > 50.0 && model.contains("claude-3-opus") {
                insights.push(Insight {
                    id: format!("cost_model_{}", uuid::Uuid::new_v4()),
                    insight_type: InsightType::CostOptimization,
                    severity: InsightSeverity::Medium,
                    category: InsightCategory::Cost,
                    title: "High usage of expensive model detected".to_string(),
                    description: format!(
                        "{} accounts for {:.1}% of your total costs. Consider using cheaper models for simpler tasks.",
                        model, cost_percentage
                    ),
                    recommendation: format!(
                        "Consider using Claude 3 Sonnet for simpler tasks to potentially save 40-60% on costs."
                    ),
                    impact: ImpactAssessment {
                        potential_savings: Some(cost * 0.5),
                        potential_efficiency_gain: None,
                        risk_level: RiskLevel::Low,
                        implementation_difficulty: DifficultyLevel::Easy,
                    },
                    confidence: 0.85,
                    created_at: Utc::now(),
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert("model".to_string(), serde_json::Value::String(model.clone()));
                        meta.insert("cost_percentage".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(cost_percentage).unwrap()));
                        meta
                    },
                });
            }
        }

        // Analyze token efficiency
        let token_efficiency: HashMap<String, f64> = records
            .iter()
            .fold(HashMap::new(), |mut acc, record| {
                let efficiency = if record.total_tokens() > 0 {
                    record.cost / record.total_tokens() as f64
                } else {
                    0.0
                };
                *acc.entry(record.model.clone()).or_insert(0.0) += efficiency;
                acc
            });

        // Find inefficient models
        for (model, efficiency) in token_efficiency {
            if efficiency > 0.00002 { // High cost per token
                insights.push(Insight {
                    id: format!("efficiency_model_{}", uuid::Uuid::new_v4()),
                    insight_type: InsightType::CostOptimization,
                    severity: InsightSeverity::Low,
                    category: InsightCategory::Cost,
                    title: "Low token efficiency detected".to_string(),
                    description: format!(
                        "{} has high cost per token (${:.6}). Consider optimizing prompts.",
                        model, efficiency
                    ),
                    recommendation: "Review and optimize prompts to reduce token usage and improve efficiency.".to_string(),
                    impact: ImpactAssessment {
                        potential_savings: Some(total_cost * 0.1),
                        potential_efficiency_gain: Some(20.0),
                        risk_level: RiskLevel::None,
                        implementation_difficulty: DifficultyLevel::Moderate,
                    },
                    confidence: 0.75,
                    created_at: Utc::now(),
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert("model".to_string(), serde_json::Value::String(model.clone()));
                        meta.insert("efficiency".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(efficiency).unwrap()));
                        meta
                    },
                });
            }
        }

        Ok(insights)
    }

    /// Generate usage pattern insights
    fn generate_usage_pattern_insights(&self, records: &[UsageRecord]) -> Result<Vec<Insight>> {
        let mut insights = Vec::new();

        // Analyze usage frequency
        let usage_by_hour = self.analyze_usage_by_hour(records);
        
        // Find unusual usage patterns
        let total_usage = usage_by_hour.values().sum::<u32>() as f64;
        let avg_usage = total_usage / 24.0;
        
        for (hour, count) in usage_by_hour {
            let percentage = (count as f64 / total_usage) * 100.0;
            
            if percentage > 15.0 && (hour >= 22 || hour <= 6) {
                insights.push(Insight {
                    id: format!("pattern_off_hours_{}", uuid::Uuid::new_v4()),
                    insight_type: InsightType::UsagePattern,
                    severity: InsightSeverity::Low,
                    category: InsightCategory::Usage,
                    title: "High off-hours usage detected".to_string(),
                    description: format!(
                        "Significant usage at {}:00 ({}% of total). Consider scheduling tasks during business hours.",
                        hour, percentage
                    ),
                    recommendation: "Review if off-hours usage is necessary or can be scheduled during regular hours.".to_string(),
                    impact: ImpactAssessment {
                        potential_savings: None,
                        potential_efficiency_gain: Some(10.0),
                        risk_level: RiskLevel::Low,
                        implementation_difficulty: DifficultyLevel::Easy,
                    },
                    confidence: 0.80,
                    created_at: Utc::now(),
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert("hour".to_string(), serde_json::Value::Number(serde_json::Number::from(hour)));
                        meta.insert("percentage".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(percentage).unwrap()));
                        meta
                    },
                });
            }
        }

        // Analyze session patterns
        let sessions = self.group_records_by_session(records);
        let session_lengths: Vec<u64> = sessions.iter()
            .filter_map(|session| session.duration_seconds)
            .collect();

        if !session_lengths.is_empty() {
            let avg_session_length = session_lengths.iter().sum::<u64>() as f64 / session_lengths.len() as f64;
            
            if avg_session_length < 300.0 { // Less than 5 minutes
                insights.push(Insight {
                    id: format!("pattern_short_sessions_{}", uuid::Uuid::new_v4()),
                    insight_type: InsightType::UsagePattern,
                    severity: InsightSeverity::Low,
                    category: InsightCategory::Usage,
                    title: "Short sessions detected".to_string(),
                    description: format!(
                        "Average session length is {:.1} minutes. Consider batching related requests.",
                        avg_session_length / 60.0
                    ),
                    recommendation: "Batch related requests together to reduce session overhead and improve efficiency.".to_string(),
                    impact: ImpactAssessment {
                        potential_savings: Some(total_usage * 0.05),
                        potential_efficiency_gain: Some(15.0),
                        risk_level: RiskLevel::None,
                        implementation_difficulty: DifficultyLevel::Easy,
                    },
                    confidence: 0.75,
                    created_at: Utc::now(),
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert("avg_session_length".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(avg_session_length).unwrap()));
                        meta
                    },
                });
            }
        }

        Ok(insights)
    }

    /// Generate trend analysis insights
    fn generate_trend_insights(&self, records: &[UsageRecord]) -> Result<Vec<Insight>> {
        let mut insights = Vec::new();

        // Group by date for trend analysis
        let daily_data = self.group_records_by_date(records);
        let daily_costs: Vec<f64> = daily_data.values()
            .map(|records| records.iter().map(|r| r.cost).sum())
            .collect();

        if daily_costs.len() >= 7 {
            // Calculate growth rate
            let first_week_avg = daily_costs.iter().take(7).sum::<f64>() / 7.0;
            let last_week_avg = daily_costs.iter().rev().take(7).sum::<f64>() / 7.0;
            
            if first_week_avg > 0.0 {
                let growth_rate = ((last_week_avg - first_week_avg) / first_week_avg) * 100.0;
                
                if growth_rate > 50.0 {
                    insights.push(Insight {
                        id: format!("trend_rapid_growth_{}", uuid::Uuid::new_v4()),
                        insight_type: InsightType::TrendAnalysis,
                        severity: InsightSeverity::Medium,
                        category: InsightCategory::Cost,
                        title: "Rapid cost growth detected".to_string(),
                        description: format!(
                            "Costs have increased by {:.1}% over the analysis period. Monitor usage closely.",
                            growth_rate
                        ),
                        recommendation: "Review recent usage patterns and implement cost controls if necessary.".to_string(),
                        impact: ImpactAssessment {
                            potential_savings: Some(last_week_avg * 0.2),
                            potential_efficiency_gain: None,
                            risk_level: RiskLevel::Medium,
                            implementation_difficulty: DifficultyLevel::Moderate,
                        },
                        confidence: 0.90,
                        created_at: Utc::now(),
                        metadata: {
                            let mut meta = HashMap::new();
                            meta.insert("growth_rate".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(growth_rate).unwrap()));
                            meta
                        },
                    });
                }
            }
        }

        Ok(insights)
    }

    /// Generate budget alert insights
    fn generate_budget_insights(&self, records: &[UsageRecord], budget: &BudgetInfo) -> Result<Vec<Insight>> {
        let mut insights = Vec::new();

        let total_cost = records.iter().map(|r| r.cost).sum();
        let budget_usage = (total_cost / budget.monthly_limit) * 100.0;

        if budget_usage >= budget.alert_threshold {
            insights.push(Insight {
                id: format!("budget_alert_critical_{}", uuid::Uuid::new_v4()),
                insight_type: InsightType::BudgetAlert,
                severity: InsightSeverity::Critical,
                category: InsightCategory::Cost,
                title: "Budget alert threshold exceeded".to_string(),
                description: format!(
                    "You have used {:.1}% of your monthly budget (${:.2} of ${:.2}).",
                    budget_usage, total_cost, budget.monthly_limit
                ),
                recommendation: "Immediately review usage patterns and implement cost reduction measures.".to_string(),
                impact: ImpactAssessment {
                    potential_savings: Some(budget.monthly_limit - total_cost),
                    potential_efficiency_gain: None,
                    risk_level: RiskLevel::High,
                    implementation_difficulty: DifficultyLevel::Hard,
                },
                confidence: 1.0,
                created_at: Utc::now(),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("budget_usage".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(budget_usage).unwrap()));
                    meta.insert("total_cost".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(total_cost).unwrap()));
                    meta.insert("budget_limit".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(budget.monthly_limit).unwrap()));
                    meta
                },
            });
        } else if budget_usage >= budget.warning_threshold {
            insights.push(Insight {
                id: format!("budget_alert_warning_{}", uuid::Uuid::new_v4()),
                insight_type: InsightType::BudgetAlert,
                severity: InsightSeverity::Medium,
                category: InsightCategory::Cost,
                title: "Budget warning threshold exceeded".to_string(),
                description: format!(
                    "You have used {:.1}% of your monthly budget (${:.2} of ${:.2}).",
                    budget_usage, total_cost, budget.monthly_limit
                ),
                recommendation: "Monitor usage closely and consider cost optimization strategies.".to_string(),
                impact: ImpactAssessment {
                    potential_savings: Some(budget.monthly_limit - total_cost),
                    potential_efficiency_gain: None,
                    risk_level: RiskLevel::Medium,
                    implementation_difficulty: DifficultyLevel::Moderate,
                },
                confidence: 1.0,
                created_at: Utc::now(),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("budget_usage".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(budget_usage).unwrap()));
                    meta.insert("total_cost".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(total_cost).unwrap()));
                    meta.insert("budget_limit".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(budget.monthly_limit).unwrap()));
                    meta
                },
            });
        }

        Ok(insights)
    }

    /// Generate performance insights
    fn generate_performance_insights(&self, records: &[UsageRecord]) -> Result<Vec<Insight>> {
        let mut insights = Vec::new();

        // Analyze response patterns
        let response_times: Vec<f64> = records.iter()
            .map(|r| {
                // Simulate response time based on tokens
                (r.input_tokens + r.output_tokens) as f64 * 0.001
            })
            .collect();

        if !response_times.is_empty() {
            let avg_response_time = response_times.iter().sum::<f64>() / response_times.len() as f64;
            let summary = StatisticsCalculator::calculate_summary(&response_times);
            
            // High response time variability
            if summary.standard_deviation > avg_response_time * 0.5 {
                insights.push(Insight {
                    id: format!("performance_variability_{}", uuid::Uuid::new_v4()),
                    insight_type: InsightType::Performance,
                    severity: InsightSeverity::Low,
                    category: InsightCategory::Performance,
                    title: "High response time variability detected".to_string(),
                    description: format!(
                        "Response times vary significantly (std dev: {:.2}s). This may indicate inconsistent performance.",
                        summary.standard_deviation
                    ),
                    recommendation: "Monitor API performance and consider caching responses for repeated requests.".to_string(),
                    impact: ImpactAssessment {
                        potential_savings: None,
                        potential_efficiency_gain: Some(25.0),
                        risk_level: RiskLevel::Low,
                        implementation_difficulty: DifficultyLevel::Moderate,
                    },
                    confidence: 0.70,
                    created_at: Utc::now(),
                    metadata: {
                        let mut meta = HashMap::new();
                        meta.insert("avg_response_time".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(avg_response_time).unwrap()));
                        meta.insert("std_dev".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(summary.standard_deviation).unwrap()));
                        meta
                    },
                });
            }
        }

        Ok(insights)
    }

    // Helper methods

    fn analyze_usage_by_hour(&self, records: &[UsageRecord]) -> HashMap<u8, u32> {
        let mut usage_by_hour = HashMap::new();
        
        for record in records {
            let hour = record.timestamp.hour() as u8;
            *usage_by_hour.entry(hour).or_insert(0) += 1;
        }
        
        usage_by_hour
    }

    fn group_records_by_session(&self, records: &[UsageRecord]) -> Vec<Session> {
        let mut sessions: HashMap<String, Session> = HashMap::new();
        
        for record in records {
            if let Some(session_id) = &record.session_id {
                let session = sessions.entry(session_id.clone())
                    .or_insert_with(|| Session::new(
                        session_id.clone(),
                        record.timestamp,
                        record.user_id.clone(),
                    ));
                
                session.add_record(record);
            }
        }
        
        sessions.into_values().collect()
    }

    fn group_records_by_date(&self, records: &[UsageRecord]) -> HashMap<NaiveDate, Vec<UsageRecord>> {
        let mut daily_data = HashMap::new();
        
        for record in records {
            let date = record.timestamp.date_naive();
            daily_data.entry(date).or_insert_with(Vec::new).push(record.clone());
        }
        
        daily_data
    }

    fn severity_priority(&self, severity: &InsightSeverity) -> u8 {
        match severity {
            InsightSeverity::Critical => 5,
            InsightSeverity::High => 4,
            InsightSeverity::Medium => 3,
            InsightSeverity::Low => 2,
            InsightSeverity::Informational => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;

    #[test]
    fn test_insights_engine_creation() {
        let engine = InsightsEngine::default();
        assert_eq!(engine.config.max_insights, 20);
    }

    #[test]
    fn test_generate_insights() {
        let engine = InsightsEngine::default();
        
        let records = vec![
            UsageRecord::new(Utc::now(), "claude-3-opus".to_string(), 1000, 500, 0.045),
            UsageRecord::new(Utc::now(), "claude-3-opus".to_string(), 2000, 1000, 0.090),
        ];
        
        let insights = engine.generate_insights(&records, None).unwrap();
        assert!(!insights.is_empty());
    }

    #[test]
    fn test_analyze_usage_by_hour() {
        let engine = InsightsEngine::default();
        
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
        ];
        
        let usage_by_hour = engine.analyze_usage_by_hour(&records);
        assert_eq!(usage_by_hour.get(&9), Some(&1));
    }

    #[test]
    fn test_group_records_by_session() {
        let engine = InsightsEngine::default();
        
        let records = vec![
            UsageRecord::new(Utc::now(), "claude-3-sonnet".to_string(), 1000, 500, 0.015),
        ];
        
        let sessions = engine.group_records_by_session(&records);
        assert_eq!(sessions.len(), 0); // No session ID
    }
}