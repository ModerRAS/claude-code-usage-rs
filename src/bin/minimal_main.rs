//! 最小化主程序入口
//! 
//! 这是简化实现，专注于编译成功

use ccusage_rs::minimal_app::SimpleAnalyzer;

fn main() {
    println!("Claude Code Usage Analysis Tool (Rust)");
    println!("=====================================");
    
    // 创建示例数据
    let sample_data = SimpleAnalyzer::create_sample_data();
    
    // 分析数据
    let result = SimpleAnalyzer::analyze(&sample_data);
    
    // 生成报告
    let report = SimpleAnalyzer::generate_report(&result);
    
    // 输出报告
    println!("{}", report);
}