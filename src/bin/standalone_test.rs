//! 独立测试版本
//! 
//! 这个版本完全独立，不依赖于复杂的库结构

use std::collections::HashMap;

/// 超简化的使用记录
#[derive(Debug, Clone)]
pub struct TestUsageRecord {
    pub id: String,
    pub timestamp: String,
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cost: f64,
}

impl TestUsageRecord {
    pub fn new(
        timestamp: String,
        model: String,
        input_tokens: u32,
        output_tokens: u32,
        cost: f64,
    ) -> Self {
        Self {
            id: format!("{}-{}", model, timestamp),
            timestamp,
            model,
            input_tokens,
            output_tokens,
            cost,
        }
    }
}

/// 超简化的分析结果
#[derive(Debug, Clone)]
pub struct TestAnalysisResult {
    pub total_cost: f64,
    pub total_records: usize,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
}

impl TestAnalysisResult {
    pub fn new(records: &[TestUsageRecord]) -> Self {
        let total_cost = records.iter().map(|r| r.cost).sum();
        let total_records = records.len();
        let total_input_tokens = records.iter().map(|r| r.input_tokens as u64).sum();
        let total_output_tokens = records.iter().map(|r| r.output_tokens as u64).sum();

        Self {
            total_cost,
            total_records,
            total_input_tokens,
            total_output_tokens,
        }
    }
}

/// 超简化的分析器
pub struct TestAnalyzer;

impl TestAnalyzer {
    pub fn analyze(records: &[TestUsageRecord]) -> TestAnalysisResult {
        TestAnalysisResult::new(records)
    }

    pub fn create_sample_data() -> Vec<TestUsageRecord> {
        let mut records = Vec::new();
        
        for i in 0..5 {
            let timestamp = format!("2024-01-{:02}", i + 1);
            let model = "claude-3-sonnet-20240229";
            let input_tokens = 1000 + (i * 200) as u32;
            let output_tokens = 500 + (i * 100) as u32;
            let cost = (input_tokens as f64 * 0.003 / 1000.0) + (output_tokens as f64 * 0.015 / 1000.0);
            
            records.push(TestUsageRecord::new(timestamp, model.to_string(), input_tokens, output_tokens, cost));
        }
        
        records
    }

    pub fn generate_report(result: &TestAnalysisResult) -> String {
        format!(
            "=== Claude Code Usage Analysis ===\n\n\
             Total Cost: ${:.6}\n\
             Total Records: {}\n\
             Total Input Tokens: {}\n\
             Total Output Tokens: {}\n\
             Average Cost per Record: ${:.6}\n\
             Average Input per Record: {}\n\
             Average Output per Record: {}",
            result.total_cost,
            result.total_records,
            result.total_input_tokens,
            result.total_output_tokens,
            if result.total_records > 0 { result.total_cost / result.total_records as f64 } else { 0.0 },
            if result.total_records > 0 { result.total_input_tokens / result.total_records as u64 } else { 0 },
            if result.total_records > 0 { result.total_output_tokens / result.total_records as u64 } else { 0 }
        )
    }
}

fn main() {
    println!("Claude Code Usage Analysis Tool (Rust) - Standalone Test");
    println!("==========================================================");
    
    // 创建示例数据
    let sample_data = TestAnalyzer::create_sample_data();
    
    // 分析数据
    let result = TestAnalyzer::analyze(&sample_data);
    
    // 生成报告
    let report = TestAnalyzer::generate_report(&result);
    
    // 输出报告
    println!("{}", report);
    
    println!("\n测试成功完成！");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        let sample_data = TestAnalyzer::create_sample_data();
        assert_eq!(sample_data.len(), 5);
        
        let result = TestAnalyzer::analyze(&sample_data);
        assert_eq!(result.total_records, 5);
        assert!(result.total_cost > 0.0);
        
        let report = TestAnalyzer::generate_report(&result);
        assert!(report.contains("Total Cost:"));
        assert!(report.contains("Total Records: 5"));
    }
}