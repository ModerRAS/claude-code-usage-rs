//! 简化版主程序入口
//! 
//! 这是简化实现，专注于核心功能

use ccusage_rs::simple_app::{SimpleApp, create_sample_data};
use ccusage_rs::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建简化应用
    let app = SimpleApp::new();
    
    // 创建示例数据
    let sample_data = create_sample_data();
    
    // 分析数据
    let result = app.analyze_usage(&sample_data)?;
    
    // 生成报告
    let report = app.generate_text_report(&result);
    
    // 输出报告
    println!("{}", report);
    
    Ok(())
}