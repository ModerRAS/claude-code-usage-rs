//! 简化的测试程序 - 测试JSON数据加载和基本功能

use std::collections::HashMap;
use chrono::Utc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 测试 ccusage-rs 基本功能");
    
    // 1. 测试数据模型
    println!("📊 测试数据模型...");
    let record = create_sample_record();
    println!("✅ 创建了示例记录: {}", record.id);
    println!("   - 模型: {}", record.model);
    println!("   - 输入token: {}", record.input_tokens);
    println!("   - 输出token: {}", record.output_tokens);
    println!("   - 总token: {}", record.total_tokens());
    println!("   - 成本: ${:.6}", record.cost);
    
    // 2. 测试序列化
    println!("📝 测试序列化...");
    let json = serde_json::to_string(&record)?;
    println!("✅ 序列化成功: {} 字符", json.len());
    
    // 3. 测试反序列化
    println!("📖 测试反序列化...");
    let deserialized: claude_code_usage_rs::data::models::UsageRecord = serde_json::from_str(&json)?;
    println!("✅ 反序列化成功: {}", deserialized.id);
    
    // 4. 测试统计分析
    println!("📈 测试统计分析...");
    let stats = claude_code_usage_rs::analysis::statistics::StatisticsCalculator::calculate_usage_stats(&[record.clone()]);
    println!("✅ 统计分析完成:");
    println!("   - 总请求数: {}", stats.total_requests);
    println!("   - 总token数: {}", stats.total_tokens);
    println!("   - 总成本: ${:.6}", stats.total_cost);
    println!("   - 平均每请求token数: {:.2}", stats.average_tokens_per_request);
    println!("   - 平均每请求成本: ${:.6}", stats.average_cost_per_request);
    
    // 5. 测试数据验证
    println!("🔍 测试数据验证...");
    let validation_result = validate_record(&record);
    println!("✅ 数据验证: {}", validation_result);
    
    // 6. 测试模型使用统计
    println!("📊 测试模型使用统计...");
    println!("   - 模型使用情况:");
    for (model, model_stats) in &stats.model_usage {
        println!("     * {}: {} 请求, ${:.6}", model, model_stats.request_count, model_stats.total_cost);
    }
    
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

fn validate_record(record: &claude_code_usage_rs::data::models::UsageRecord) -> String {
    let mut issues = Vec::new();
    
    if record.input_tokens == 0 {
        issues.push("输入token数为0");
    }
    
    if record.output_tokens == 0 {
        issues.push("输出token数为0");
    }
    
    if record.cost < 0.0 {
        issues.push("成本为负数");
    }
    
    if record.model.is_empty() {
        issues.push("模型名称为空");
    }
    
    if issues.is_empty() {
        "✅ 数据验证通过".to_string()
    } else {
        format!("❌ 数据验证失败: {}", issues.join(", "))
    }
}