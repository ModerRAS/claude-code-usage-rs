//! 简化版主程序，专注于核心功能
//! 
//! 这是简化实现，专注于基本的成本计算和分析

use crate::data::simple_models::*;
use crate::error::Result;

/// 简化的应用程序
pub struct SimpleApp {
    calculator: CostCalculator,
}

impl SimpleApp {
    pub fn new() -> Self {
        Self {
            calculator: CostCalculator::default(),
        }
    }

    /// 分析使用数据
    pub fn analyze_usage(&self, records: &[UsageRecord]) -> Result<AnalysisResult> {
        Ok(AnalysisResult::new(records))
    }

    /// 计算总成本
    pub fn calculate_total_cost(&self, records: &[UsageRecord]) -> Result<f64> {
        Ok(self.calculator.calculate_total_cost(records))
    }

    /// 生成简单的文本报告
    pub fn generate_text_report(&self, result: &AnalysisResult) -> String {
        let mut report = String::new();
        
        report.push_str("=== Claude Code Usage Analysis ===\n\n");
        report.push_str(&format!("Total Cost: ${:.6}\n", result.total_cost));
        report.push_str(&format!("Total Records: {}\n", result.total_records));
        report.push_str(&format!("Total Input Tokens: {}\n", result.total_input_tokens));
        report.push_str(&format!("Total Output Tokens: {}\n", result.total_output_tokens));
        
        if let Some((start, end)) = &result.date_range {
            report.push_str(&format!("Date Range: {} to {}\n", start.format("%Y-%m-%d"), end.format("%Y-%m-%d")));
        }
        
        if result.total_records > 0 {
            let avg_cost_per_record = result.total_cost / result.total_records as f64;
            let avg_input_per_record = result.total_input_tokens / result.total_records as u64;
            let avg_output_per_record = result.total_output_tokens / result.total_records as u64;
            
            report.push_str(&format!("Average Cost per Record: ${:.6}\n", avg_cost_per_record));
            report.push_str(&format!("Average Input Tokens per Record: {}\n", avg_input_per_record));
            report.push_str(&format!("Average Output Tokens per Record: {}\n", avg_output_per_record));
        }
        
        report
    }
}

/// 创建示例数据用于测试
pub fn create_sample_data() -> Vec<UsageRecord> {
    let mut records = Vec::new();
    
    // 创建一些示例记录
    for i in 0..10 {
        let timestamp = chrono::Utc::now() - chrono::Duration::days(i);
        let model = if i % 3 == 0 {
            "claude-3-sonnet-20240229"
        } else if i % 3 == 1 {
            "claude-3-opus-20240229"
        } else {
            "claude-3-haiku-20240307"
        };
        
        let input_tokens = (1000 + (i * 500)) as u32;
        let output_tokens = (500 + (i * 250)) as u32;
        
        let calculator = CostCalculator::default();
        let cost = calculator.calculate_cost(model, input_tokens, output_tokens);
        
        records.push(UsageRecord::new(timestamp, model.to_string(), input_tokens, output_tokens, cost));
    }
    
    records
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_app() {
        let app = SimpleApp::new();
        let sample_data = create_sample_data();
        
        let result = app.analyze_usage(&sample_data).unwrap();
        assert!(result.total_cost > 0.0);
        assert_eq!(result.total_records, 10);
        
        let total_cost = app.calculate_total_cost(&sample_data).unwrap();
        assert_eq!(total_cost, result.total_cost);
        
        let report = app.generate_text_report(&result);
        assert!(report.contains("Total Cost:"));
        assert!(report.contains("Total Records: 10"));
    }
}