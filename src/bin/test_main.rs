//! 测试主程序入口
//! 
//! 这个版本专注于最基本的功能测试

use ccusage_rs::test_app::TestAnalyzer;

fn main() {
    println!("Claude Code Usage Analysis Tool (Rust) - Test Version");
    println!("====================================================");
    
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