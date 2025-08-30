//! 简化版数据模型，专注于核心功能
//! 
//! 这是简化实现，只包含必要的字段和功能

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// 简化的使用记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cost: f64,
    pub session_id: Option<String>,
    pub user_id: Option<String>,
}

impl UsageRecord {
    pub fn new(
        timestamp: DateTime<Utc>,
        model: String,
        input_tokens: u32,
        output_tokens: u32,
        cost: f64,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp,
            model,
            input_tokens,
            output_tokens,
            cost,
            session_id: None,
            user_id: None,
        }
    }
}

/// 简化的成本计算器
pub struct CostCalculator {
    pub model_costs: std::collections::HashMap<String, (f64, f64)>,
}

impl Default for CostCalculator {
    fn default() -> Self {
        let mut model_costs = std::collections::HashMap::new();
        model_costs.insert("claude-3-sonnet-20240229".to_string(), (0.003, 0.015));
        model_costs.insert("claude-3-opus-20240229".to_string(), (0.015, 0.075));
        model_costs.insert("claude-3-haiku-20240307".to_string(), (0.00025, 0.00125));
        Self { model_costs }
    }
}

impl CostCalculator {
    pub fn calculate_cost(&self, model: &str, input_tokens: u32, output_tokens: u32) -> f64 {
        if let Some((input_cost, output_cost)) = self.model_costs.get(model) {
            (input_tokens as f64 * input_cost / 1000.0) + (output_tokens as f64 * output_cost / 1000.0)
        } else {
            // 默认费率
            (input_tokens as f64 * 0.001 / 1000.0) + (output_tokens as f64 * 0.002 / 1000.0)
        }
    }

    pub fn calculate_total_cost(&self, records: &[UsageRecord]) -> f64 {
        records.iter().map(|r| r.cost).sum()
    }
}

/// 简化的分析结果
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    pub total_cost: f64,
    pub total_records: usize,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
}

impl AnalysisResult {
    pub fn new(records: &[UsageRecord]) -> Self {
        let total_cost = records.iter().map(|r| r.cost).sum();
        let total_records = records.len();
        let total_input_tokens = records.iter().map(|r| r.input_tokens as u64).sum();
        let total_output_tokens = records.iter().map(|r| r.output_tokens as u64).sum();
        
        let date_range = if records.is_empty() {
            None
        } else {
            let min_date = records.iter().map(|r| r.timestamp).min().unwrap();
            let max_date = records.iter().map(|r| r.timestamp).max().unwrap();
            Some((min_date, max_date))
        };

        Self {
            total_cost,
            total_records,
            total_input_tokens,
            total_output_tokens,
            date_range,
        }
    }
}