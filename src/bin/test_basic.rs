//! 简化的测试程序 - 测试基本功能

use chrono::Utc;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 测试 ccusage-rs 基本功能");
    
    // 1. 测试数据模型
    println!("📊 测试数据模型...");
    let record = create_sample_record();
    println!("✅ 创建了示例记录: {}", record.id);
    
    // 2. 测试成本计算
    println!("💰 测试成本计算...");
    let mut calculator = claude_code_usage_rs::analysis::calculator::CostCalculator::default();
    
    // 添加默认定价数据
    let pricing_data = create_sample_pricing_data();
    calculator.load_pricing_data(pricing_data)?;
    
    let cost = calculator.calculate_cost(&record)?;
    println!("✅ 计算成本: ${:.6}", cost);
    
    // 3. 测试序列化
    println!("📝 测试序列化...");
    let json = serde_json::to_string(&record)?;
    println!("✅ 序列化成功: {} 字符", json.len());
    
    // 4. 测试统计分析
    println!("📈 测试统计分析...");
    let stats = claude_code_usage_rs::analysis::statistics::StatisticsCalculator::calculate_usage_stats(&[record.clone()]);
    println!("✅ 统计分析完成: {} 条记录", stats.total_requests);
    
    println!("🎉 所有基本功能测试通过！");
    Ok(())
}

fn create_sample_record() -> claude_code_usage_rs::data::models::UsageRecord {
    let mut metadata = HashMap::new();
    metadata.insert("api_endpoint".to_string(), serde_json::Value::String("/v1/messages".to_string()));
    metadata.insert("response_time_ms".to_string(), serde_json::Value::Number(serde_json::Number::from(1200)));
    metadata.insert("status".to_string(), serde_json::Value::String("success".to_string()));
    
    claude_code_usage_rs::data::models::UsageRecord {
        id: "test-record-001".to_string(),
        timestamp: Utc::now(),
        model: "claude-3-sonnet-20240229".to_string(),
        input_tokens: 1500,
        output_tokens: 800,
        cost: 0.023,
        session_id: Some("test-session-001".to_string()),
        user_id: Some("user123".to_string()),
        metadata,
    }
}

fn create_sample_pricing_data() -> Vec<claude_code_usage_rs::data::models::PricingInfo> {
    vec![
        claude_code_usage_rs::data::models::PricingInfo {
            model: "claude-3-sonnet-20240229".to_string(),
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.015,
            currency: "USD".to_string(),
            effective_date: Utc::now() - chrono::Duration::days(1),
            is_active: true,
        }
    ]
}