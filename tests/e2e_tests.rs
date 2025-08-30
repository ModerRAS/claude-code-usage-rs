//! 端到端测试
//! 
//! 测试完整的用户工作流程和实际使用场景

mod common;

use common::*;
use ccusage_rs::*;
use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;
use std::fs;
use std::path::Path;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_complete_user_workflow() {
        // 模拟完整的用户工作流程
        
        let temp_dir = TempDir::new().unwrap();
        let mut generator = TestDataGenerator::new();
        
        // 1. 用户创建配置文件
        let config = generator.generate_config();
        let config_file = temp_dir.path().join("config.toml");
        let toml_content = generate_toml_from_json(&config);
        fs::write(&config_file, toml_content).unwrap();
        
        // 2. 用户创建测试数据
        let test_data = generator.generate_usage_records_for_date_range(
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
            10, // 每天10条记录
        );
        
        let data_file = temp_dir.path().join("usage_data.json");
        let json_data = serde_json::to_string_pretty(&json!(test_data)).unwrap();
        fs::write(&data_file, json_data).unwrap();
        
        // 3. 用户设置预算
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        let output = cmd
            .arg("--config")
            .arg(&config_file)
            .arg("budget")
            .arg("set")
            .arg("--limit")
            .arg("200.0")
            .arg("--currency")
            .arg("USD")
            .arg("--warning")
            .arg("80")
            .arg("--alert")
            .arg("95")
            .output()
            .expect("Failed to execute budget set command");
        
        assert!(output.status.success());
        assert!(String::from_utf8(output.stdout).unwrap().contains("Budget set"));
        
        // 4. 用户验证数据
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        let output = cmd
            .arg("--config")
            .arg(&config_file)
            .arg("data")
            .arg("validate")
            .arg("--data-file")
            .arg(&data_file)
            .output()
            .expect("Failed to execute data validate command");
        
        assert!(output.status.success());
        let output_str = String::from_utf8(output.stdout).unwrap();
        assert!(output_str.contains("valid"));
        assert!(output_str.contains("310")); // 31 days * 10 records
        
        // 5. 用户查看数据信息
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        let output = cmd
            .arg("--config")
            .arg(&config_file)
            .arg("data")
            .arg("info")
            .arg("--data-file")
            .arg(&data_file)
            .output()
            .expect("Failed to execute data info command");
        
        assert!(output.status.success());
        let output_str = String::from_utf8(output.stdout).unwrap();
        assert!(output_str.contains("Total records"));
        assert!(output_str.contains("Total cost"));
        assert!(output_str.contains("Total tokens"));
        
        // 6. 用户进行综合分析
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        let output = cmd
            .arg("--config")
            .arg(&config_file)
            .arg("--data")
            .arg(&data_file)
            .arg("--format")
            .arg("json")
            .arg("analyze")
            .arg("--analysis-type")
            .arg("comprehensive")
            .arg("--date-range")
            .arg("2024-01-01..2024-01-31")
            .arg("--detailed")
            .output()
            .expect("Failed to execute analyze command");
        
        assert!(output.status.success());
        let output_str = String::from_utf8(output.stdout).unwrap();
        let analysis_result: serde_json::Value = serde_json::from_str(&output_str).unwrap();
        assert!(analysis_result.is_object());
        
        // 7. 用户查看预算状态
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        let output = cmd
            .arg("--config")
            .arg(&config_file)
            .arg("budget")
            .arg("status")
            .output()
            .expect("Failed to execute budget status command");
        
        assert!(output.status.success());
        let output_str = String::from_utf8(output.stdout).unwrap();
        assert!(output_str.contains("Budget"));
        
        // 8. 用户生成日报
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        let output = cmd
            .arg("--config")
            .arg(&config_file)
            .arg("--data")
            .arg(&data_file)
            .arg("daily")
            .arg("--date")
            .arg("2024-01-15")
            .arg("--compare")
            .output()
            .expect("Failed to execute daily command");
        
        assert!(output.status.success());
        let output_str = String::from_utf8(output.stdout).unwrap();
        assert!(output_str.contains("Daily Report"));
        
        // 9. 用户生成周报
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        let output = cmd
            .arg("--config")
            .arg(&config_file)
            .arg("--data")
            .arg(&data_file)
            .arg("weekly")
            .arg("--week-start")
            .arg("2024-01-15")
            .arg("--weeks")
            .arg("2")
            .output()
            .expect("Failed to execute weekly command");
        
        assert!(output.status.success());
        let output_str = String::from_utf8(output.stdout).unwrap();
        assert!(output_str.contains("Weekly Report"));
        
        // 10. 用户生成月报
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        let output = cmd
            .arg("--config")
            .arg(&config_file)
            .arg("--data")
            .arg(&data_file)
            .arg("monthly")
            .arg("--year")
            .arg("2024")
            .arg("--month")
            .arg("1")
            .arg("--compare")
            .output()
            .expect("Failed to execute monthly command");
        
        assert!(output.status.success());
        let output_str = String::from_utf8(output.stdout).unwrap();
        assert!(output_str.contains("Monthly Report"));
        
        // 11. 用户获取洞见
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        let output = cmd
            .arg("--config")
            .arg(&config_file)
            .arg("--data")
            .arg(&data_file)
            .arg("insights")
            .arg("--count")
            .arg("15")
            .output()
            .expect("Failed to execute insights command");
        
        assert!(output.status.success());
        let output_str = String::from_utf8(output.stdout).unwrap();
        assert!(output_str.contains("Insights"));
        
        // 12. 用户查看统计信息
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        let output = cmd
            .arg("--config")
            .arg(&config_file)
            .arg("--data")
            .arg(&data_file)
            .arg("stats")
            .arg("--stats-type")
            .arg("detailed")
            .arg("--group-by")
            .arg("model")
            .output()
            .expect("Failed to execute stats command");
        
        assert!(output.status.success());
        let output_str = String::from_utf8(output.stdout).unwrap();
        assert!(output_str.contains("Statistics"));
        
        // 13. 用户导出数据
        let export_file = temp_dir.path().join("export.json");
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        let output = cmd
            .arg("--config")
            .arg(&config_file)
            .arg("--data")
            .arg(&data_file)
            .arg("export")
            .arg("--format")
            .arg("json")
            .arg("--start-date")
            .arg("2024-01-01")
            .arg("--end-date")
            .arg("2024-01-31")
            .arg("--output")
            .arg(&export_file)
            .output()
            .expect("Failed to execute export command");
        
        assert!(output.status.success());
        let output_str = String::from_utf8(output.stdout).unwrap();
        assert!(output_str.contains("Exported"));
        assert!(export_file.exists());
        
        // 14. 用户导出配置
        let config_export_file = temp_dir.path().join("config_export.toml");
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        let output = cmd
            .arg("--config")
            .arg(&config_file)
            .arg("config")
            .arg("export")
            .arg("--output")
            .arg(&config_export_file)
            .output()
            .expect("Failed to execute config export command");
        
        assert!(output.status.success());
        let output_str = String::from_utf8(output.stdout).unwrap();
        assert!(output_str.contains("Configuration exported"));
        assert!(config_export_file.exists());
        
        // 15. 验证所有文件都存在且包含预期内容
        assert!(data_file.exists());
        assert!(config_file.exists());
        assert!(export_file.exists());
        assert!(config_export_file.exists());
        
        // 验证导出文件内容
        let export_content = fs::read_to_string(&export_file).unwrap();
        let export_data: serde_json::Value = serde_json::from_str(&export_content).unwrap();
        assert!(export_data.is_array());
        assert!(export_data.as_array().unwrap().len() > 0);
        
        // 验证配置导出文件内容
        let config_export_content = fs::read_to_string(&config_export_file).unwrap();
        assert!(config_export_content.contains("[app]"));
        assert!(config_export_content.contains("[data]"));
        assert!(config_export_content.contains("[output]"));
    }

    #[tokio::test]
    async fn test_real_world_scenario() {
        // 模拟真实世界使用场景：用户分析一个月的使用情况
        
        let temp_dir = TempDir::new().unwrap();
        let mut generator = TestDataGenerator::new();
        
        // 生成真实的月度数据，包含不同的使用模式
        let mut realistic_data = Vec::new();
        
        // 工作日模式：高使用量
        for day in 1..32 {
            let date = chrono::NaiveDate::from_ymd_opt(2024, 1, day).unwrap();
            let weekday = date.weekday();
            
            let records_per_day = match weekday {
                chrono::Weekday::Sat | chrono::Weekday::Sun => 5,  // 周末较少
                _ => 15,  // 工作日较多
            };
            
            for hour in 9..18 {  // 工作时间
                for _ in 0..(records_per_day / 9) {
                    let mut record = generator.generate_usage_record();
                    
                    // 设置具体时间
                    let timestamp = chrono::DateTime::from_naive_utc_and_offset(
                        date.and_hms_opt(hour, 0, 0).unwrap(),
                        chrono::Utc,
                    );
                    record["timestamp"] = json!(timestamp.to_rfc3339());
                    
                    // 根据时间段调整模型使用
                    let model = match hour {
                        9..11 => "claude-3-opus",  // 早晨使用高级模型
                        12..14 => "claude-3-sonnet",  // 中午使用标准模型
                        15..17 => "claude-3-haiku",  // 下午使用快速模型
                        _ => "claude-3-sonnet",
                    };
                    record["model"] = json!(model);
                    
                    realistic_data.push(record);
                }
            }
        }
        
        let data_file = temp_dir.path().join("realistic_data.json");
        let json_data = serde_json::to_string_pretty(&json!(realistic_data)).unwrap();
        fs::write(&data_file, json_data).unwrap();
        
        // 设置合理的预算
        let budget = json!({
            "monthly_limit": 500.0,
            "currency": "USD",
            "warning_threshold": 80.0,
            "alert_threshold": 95.0,
            "enable_alerts": true
        });
        
        let config = generator.generate_config();
        let merged_config = {
            let mut config = config;
            config["budget"] = budget;
            config
        };
        
        let config_file = temp_dir.path().join("realistic_config.toml");
        let toml_content = generate_toml_from_json(&merged_config);
        fs::write(&config_file, toml_content).unwrap();
        
        // 用户执行完整的月度分析流程
        
        // 1. 设置预算
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        let output = cmd
            .arg("--config")
            .arg(&config_file)
            .arg("budget")
            .arg("set")
            .arg("--limit")
            .arg("500")
            .arg("--currency")
            .arg("USD")
            .output()
            .expect("Failed to set budget");
        
        assert!(output.status.success());
        
        // 2. 执行月度分析
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        let output = cmd
            .arg("--config")
            .arg(&config_file)
            .arg("--data")
            .arg(&data_file)
            .arg("--format")
            .arg("json")
            .arg("monthly")
            .arg("--year")
            .arg("2024")
            .arg("--month")
            .arg("1")
            .arg("--compare")
            .output()
            .expect("Failed to execute monthly analysis");
        
        assert!(output.status.success());
        let output_str = String::from_utf8(output.stdout).unwrap();
        let monthly_result: serde_json::Value = serde_json::from_str(&output_str).unwrap();
        
        // 验证月度分析结果
        assert!(monthly_result.is_object());
        if let Some(total_cost) = monthly_result.get("total_cost") {
            assert!(total_cost.is_number());
            assert!(total_cost.as_f64().unwrap() > 0.0);
        }
        
        // 3. 检查预算状态
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        let output = cmd
            .arg("--config")
            .arg(&config_file)
            .arg("--data")
            .arg(&data_file)
            .arg("budget")
            .arg("status")
            .output()
            .expect("Failed to check budget status");
        
        assert!(output.status.success());
        let output_str = String::from_utf8(output.stdout).unwrap();
        
        // 4. 获取使用洞见
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        let output = cmd
            .arg("--config")
            .arg(&config_file)
            .arg("--data")
            .arg(&data_file)
            .arg("insights")
            .arg("--count")
            .arg("20")
            .output()
            .expect("Failed to get insights");
        
        assert!(output.status.success());
        let output_str = String::from_utf8(output.stdout).unwrap();
        assert!(output_str.contains("Insights"));
        
        // 5. 分析使用趋势
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        let output = cmd
            .arg("--config")
            .arg(&config_file)
            .arg("--data")
            .arg(&data_file)
            .arg("analyze")
            .arg("--analysis-type")
            .arg("trends")
            .arg("--date-range")
            .arg("2024-01-01..2024-01-31")
            .output()
            .expect("Failed to analyze trends");
        
        assert!(output.status.success());
        let output_str = String::from_utf8(output.stdout).unwrap();
        assert!(output_str.contains("Trends"));
        
        // 6. 按模型分析使用情况
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        let output = cmd
            .arg("--config")
            .arg(&config_file)
            .arg("--data")
            .arg(&data_file)
            .arg("stats")
            .arg("--stats-type")
            .arg("models")
            .output()
            .expect("Failed to get model stats");
        
        assert!(output.status.success());
        let output_str = String::from_utf8(output.stdout).unwrap();
        assert!(output_str.contains("claude-3-opus"));
        assert!(output_str.contains("claude-3-sonnet"));
        assert!(output_str.contains("claude-3-haiku"));
        
        // 7. 导出完整报告
        let report_file = temp_dir.path().join("monthly_report.json");
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        let output = cmd
            .arg("--config")
            .arg(&config_file)
            .arg("--data")
            .arg(&data_file)
            .arg("export")
            .arg("--format")
            .arg("json")
            .arg("--start-date")
            .arg("2024-01-01")
            .arg("--end-date")
            .arg("2024-01-31")
            .arg("--output")
            .arg(&report_file)
            .output()
            .expect("Failed to export report");
        
        assert!(output.status.success());
        assert!(report_file.exists());
        
        // 验证报告内容
        let report_content = fs::read_to_string(&report_file).unwrap();
        let report_data: serde_json::Value = serde_json::from_str(&report_content).unwrap();
        assert!(report_data.is_array());
        assert!(report_data.as_array().unwrap().len() > 0);
        
        // 8. 生成每日详细报告（选择几个关键日期）
        let key_dates = vec!["2024-01-01", "2024-01-15", "2024-01-31"];
        for date in key_dates {
            let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
            let output = cmd
                .arg("--config")
                .arg(&config_file)
                .arg("--data")
                .arg(&data_file)
                .arg("daily")
                .arg("--date")
                .arg(date)
                .arg("--compare")
                .output()
                .expect(&format!("Failed to generate daily report for {}", date));
            
            assert!(output.status.success());
            let output_str = String::from_utf8(output.stdout).unwrap();
            assert!(output_str.contains("Daily"));
            assert!(output_str.contains(date));
        }
        
        // 验证所有生成文件的存在性和内容
        assert!(data_file.exists());
        assert!(config_file.exists());
        assert!(report_file.exists());
        
        println!("Real-world scenario test completed successfully!");
        println!("Generated files:");
        println!("  - Data file: {:?}", data_file);
        println!("  - Config file: {:?}", config_file);
        println!("  - Report file: {:?}", report_file);
    }

    #[tokio::test]
    async fn test_error_recovery_workflow() {
        // 测试错误恢复工作流程
        
        let temp_dir = TempDir::new().unwrap();
        let mut generator = TestDataGenerator::new();
        
        // 1. 尝试使用不存在的配置文件
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        let output = cmd
            .arg("--config")
            .arg("/nonexistent/config.toml")
            .arg("analyze")
            .output()
            .expect("Failed to execute command with nonexistent config");
        
        // 应该使用默认配置或者给出适当的错误
        // 这取决于具体的错误处理策略
        println!("Nonexistent config test: {:?}", output.status);
        
        // 2. 尝试使用损坏的数据文件
        let corrupted_file = temp_dir.path().join("corrupted.json");
        fs::write(&corrupted_file, "this is not valid json").unwrap();
        
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        let output = cmd
            .arg("--data")
            .arg(&corrupted_file)
            .arg("analyze")
            .output()
            .expect("Failed to execute command with corrupted data");
        
        // 应该给出适当的错误信息
        assert!(!output.status.success());
        let output_str = String::from_utf8(output.stderr).unwrap();
        assert!(output_str.contains("error") || output_str.contains("Error"));
        
        // 3. 尝试使用空数据文件
        let empty_file = temp_dir.path().join("empty.json");
        fs::write(&empty_file, "[]").unwrap();
        
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        let output = cmd
            .arg("--data")
            .arg(&empty_file)
            .arg("analyze")
            .output()
            .expect("Failed to execute command with empty data");
        
        // 空数据应该被正确处理
        assert!(output.status.success());
        let output_str = String::from_utf8(output.stdout).unwrap();
        // 可能会显示"No data found"之类的信息
        
        // 4. 尝试使用无效的日期范围
        let valid_file = temp_dir.path().join("valid.json");
        let test_data = generator.generate_usage_records(10);
        let json_data = serde_json::to_string_pretty(&json!(test_data)).unwrap();
        fs::write(&valid_file, json_data).unwrap();
        
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        let output = cmd
            .arg("--data")
            .arg(&valid_file)
            .arg("analyze")
            .arg("--date-range")
            .arg("invalid-date-range")
            .output()
            .expect("Failed to execute command with invalid date range");
        
        // 应该给出适当的错误信息
        assert!(!output.status.success());
        let output_str = String::from_utf8(output.stderr).unwrap();
        assert!(output_str.contains("error") || output_str.contains("Error"));
        
        // 5. 尝试使用无效的输出格式
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        let output = cmd
            .arg("--data")
            .arg(&valid_file)
            .arg("--format")
            .arg("invalid_format")
            .arg("analyze")
            .output()
            .expect("Failed to execute command with invalid format");
        
        // 应该给出适当的错误信息
        assert!(!output.status.success());
        let output_str = String::from_utf8(output.stderr).unwrap();
        assert!(output_str.contains("error") || output_str.contains("Error"));
        
        // 6. 尝试导出到不存在的目录
        let non_existent_dir = temp_dir.path().join("non_existent");
        let export_file = non_existent_dir.join("export.json");
        
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        let output = cmd
            .arg("--data")
            .arg(&valid_file)
            .arg("export")
            .arg("--format")
            .arg("json")
            .arg("--start-date")
            .arg("2024-01-01")
            .arg("--end-date")
            .arg("2024-01-31")
            .arg("--output")
            .arg(&export_file)
            .output()
            .expect("Failed to execute export to non-existent directory");
        
        // 应该创建目录或者给出适当的错误
        // 这取决于具体的实现
        println!("Export to non-existent directory test: {:?}", output.status);
        
        // 7. 验证系统在错误后仍能正常工作
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        let output = cmd
            .arg("--data")
            .arg(&valid_file)
            .arg("analyze")
            .output()
            .expect("Failed to execute command after errors");
        
        // 系统应该仍然能够正常工作
        assert!(output.status.success());
        
        println!("Error recovery workflow test completed successfully!");
    }

    #[tokio::test]
    async fn test_performance_bottleneck_workflow() {
        // 测试性能瓶颈工作流程
        
        let temp_dir = TempDir::new().unwrap();
        let mut generator = TestDataGenerator::new();
        
        // 1. 创建大型数据集来测试性能
        println!("Generating large dataset for performance testing...");
        let large_dataset = generator.generate_usage_records(5000);
        let data_file = temp_dir.path().join("large_dataset.json");
        let json_data = serde_json::to_string_pretty(&json!(large_dataset)).unwrap();
        fs::write(&data_file, json_data).unwrap();
        
        // 2. 测试大数据集的分析性能
        println!("Testing analysis performance with large dataset...");
        let start_time = std::time::Instant::now();
        
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        let output = cmd
            .arg("--data")
            .arg(&data_file)
            .arg("--format")
            .arg("json")
            .arg("analyze")
            .arg("--analysis-type")
            .arg("comprehensive")
            .output()
            .expect("Failed to execute performance test");
        
        let duration = start_time.elapsed();
        println!("Analysis completed in {:?}", duration);
        
        assert!(output.status.success());
        
        // 验证性能要求（应该在合理时间内完成）
        assert!(duration.as_secs() < 30, "Analysis should complete within 30 seconds");
        
        // 3. 测试导出性能
        println!("Testing export performance...");
        let start_time = std::time::Instant::now();
        
        let export_file = temp_dir.path().join("large_export.json");
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        let output = cmd
            .arg("--data")
            .arg(&data_file)
            .arg("export")
            .arg("--format")
            .arg("json")
            .arg("--start-date")
            .arg("2024-01-01")
            .arg("--end-date")
            .arg("2024-12-31")
            .arg("--output")
            .arg(&export_file)
            .output()
            .expect("Failed to execute export performance test");
        
        let duration = start_time.elapsed();
        println!("Export completed in {:?}", duration);
        
        assert!(output.status.success());
        assert!(export_file.exists());
        
        // 验证导出性能
        assert!(duration.as_secs() < 60, "Export should complete within 60 seconds");
        
        // 4. 测试内存使用
        println!("Testing memory usage...");
        let start_memory = get_memory_usage();
        
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        let output = cmd
            .arg("--data")
            .arg(&data_file)
            .arg("stats")
            .arg("--stats-type")
            .arg("detailed")
            .output()
            .expect("Failed to execute memory usage test");
        
        let end_memory = get_memory_usage();
        let memory_increase = end_memory.saturating_sub(start_memory);
        
        println!("Memory usage increased by {} KB", memory_increase);
        
        assert!(output.status.success());
        
        // 验证内存使用合理（这里只是一个基本检查）
        assert!(memory_increase < 100 * 1024, "Memory increase should be reasonable"); // 100MB
        
        // 5. 测试并发性能
        println!("Testing concurrent operations...");
        let start_time = std::time::Instant::now();
        
        let mut handles = Vec::new();
        
        // 并发执行多个不同的分析命令
        for i in 0..5 {
            let data_file = data_file.clone();
            let temp_dir = temp_dir.path().to_path_buf();
            let handle = tokio::spawn(async move {
                let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
                let output = cmd
                    .arg("--data")
                    .arg(&data_file)
                    .arg("daily")
                    .arg("--date")
                    .arg(&format!("2024-01-{:02}", i + 1))
                    .output()
                    .expect("Failed to execute concurrent test");
                
                output
            });
            handles.push(handle);
        }
        
        // 等待所有并发操作完成
        for handle in handles {
            let output = handle.await.unwrap();
            assert!(output.status.success());
        }
        
        let duration = start_time.elapsed();
        println!("Concurrent operations completed in {:?}", duration);
        
        // 验证并发性能
        assert!(duration.as_secs() < 60, "Concurrent operations should complete within 60 seconds");
        
        println!("Performance bottleneck workflow test completed successfully!");
    }

    // 辅助函数：获取内存使用情况
    fn get_memory_usage() -> u64 {
        match memory_stats::memory_stats() {
            Ok(stats) => stats.physical_mem / 1024, // 转换为KB
            Err(_) => 0,
        }
    }
}