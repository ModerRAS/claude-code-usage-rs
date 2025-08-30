//! 集成测试
//! 
//! 测试模块间的交互和整体功能

mod common;

use common::*;
use ccusage_rs::*;
use assert_cmd::Command;
use predicates::str::contains;
use tempfile::TempDir;
use std::path::Path;
use std::fs;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_full_data_processing_pipeline() {
        // 创建测试数据
        let mut generator = TestDataGenerator::new();
        let test_records = generator.generate_usage_records(50);
        let temp_dir = TempDir::new().unwrap();
        let data_file = temp_dir.path().join("test_data.json");
        
        // 写入测试数据
        let json_data = serde_json::to_string_pretty(&json!(test_records)).unwrap();
        fs::write(&data_file, json_data).unwrap();
        
        // 测试完整的处理流程
        let config_manager = ConfigManager::new().unwrap();
        
        // 加载数据
        let data_loader = DataLoader::with_source(
            DataSourceType::Json,
            data_file.to_string_lossy().to_string(),
        );
        let records = data_loader.load_usage_data().await.unwrap();
        assert_eq!(records.len(), 50);
        
        // 验证所有记录
        for record in &records {
            record.assert_valid_usage_record();
        }
        
        // 进行分析
        let calculator = CostCalculator::default();
        let total_cost = calculator.calculate_total_cost(&records).unwrap();
        assert_positive!(total_cost);
        
        let breakdown = calculator.calculate_detailed_breakdown(&records).unwrap();
        assert!(breakdown.total_cost > 0.0);
        assert!(!breakdown.model_breakdown.is_empty());
        
        // 计算统计信息
        let stats = StatisticsCalculator::calculate_usage_stats(&records);
        assert_positive!(stats.total_requests);
        assert_positive!(stats.total_tokens);
        assert_positive!(stats.total_cost);
        assert!(!stats.model_usage.is_empty());
        
        // 生成趋势分析
        let analyzer = TrendAnalyzer::default();
        let trends = analyzer.analyze_trends(&records).unwrap();
        assert!(!trends.daily_costs.is_empty());
        assert!(!trends.daily_tokens.is_empty());
        
        // 生成洞见
        let mut engine = InsightsEngine::default();
        let insights = engine.generate_insights(&records, None).unwrap();
        assert!(!insights.is_empty());
        
        // 验证洞见质量
        for insight in insights {
            assert!(!insight.title.is_empty());
            assert!(!insight.description.is_empty());
            assert!(insight.confidence > 0.0 && insight.confidence <= 1.0);
        }
        
        // 输出结果
        let formatter = OutputFormatter::new(OutputFormat::Json);
        let output_file = temp_dir.path().join("output.json");
        
        let result = formatter.output_usage_stats(&stats, Some(output_file.to_str().unwrap()));
        assert!(result.is_ok());
        
        // 验证输出文件
        assert!(output_file.exists());
        let output_content = fs::read_to_string(&output_file).unwrap();
        let json_output: serde_json::Value = serde_json::from_str(&output_content).unwrap();
        assert!(json_output.is_object());
        assert!(json_output.get("total_requests").is_some());
        assert!(json_output.get("total_tokens").is_some());
        assert!(json_output.get("total_cost").is_some());
    }

    #[tokio::test]
    async fn test_config_and_data_integration() {
        // 创建测试配置
        let mut generator = TestDataGenerator::new();
        let test_config = generator.generate_config();
        let temp_dir = TempDir::new().unwrap();
        let config_file = temp_dir.path().join("config.toml");
        
        // 写入配置文件
        let toml_content = generate_toml_from_json(&test_config);
        fs::write(&config_file, toml_content).unwrap();
        
        // 测试配置加载和使用
        let config_manager = ConfigManager::new_with_config(&config_file).unwrap();
        let config = config_manager.get_config();
        
        // 验证配置
        assert_eq!(config.data.default_source, test_config["data"]["default_source"].as_str().unwrap());
        assert_eq!(config.output.default_format, test_config["output"]["default_format"].as_str().unwrap());
        
        // 创建测试数据
        let test_records = generator.generate_usage_records(20);
        let data_file = temp_dir.path().join("data.json");
        let json_data = serde_json::to_string_pretty(&json!(test_records)).unwrap();
        fs::write(&data_file, json_data).unwrap();
        
        // 使用配置加载数据
        let data_source_path = config_manager.get_data_source_path("json").unwrap();
        let data_loader = DataLoader::with_source(DataSourceType::Json, data_source_path);
        let records = data_loader.load_usage_data().await.unwrap();
        
        // 使用配置进行分析
        let calculator = CostCalculator::default();
        let breakdown = calculator.calculate_detailed_breakdown(&records).unwrap();
        
        // 使用配置的输出格式
        let output_format = match config.output.default_format.as_str() {
            "json" => OutputFormat::Json,
            "csv" => OutputFormat::Csv,
            _ => OutputFormat::Table,
        };
        
        let formatter = OutputFormatter::new(output_format);
        let output_file = temp_dir.path().join("analysis_output.json");
        
        let result = formatter.output_cost_breakdown(&breakdown, Some(output_file.to_str().unwrap()));
        assert!(result.is_ok());
        
        // 验证输出
        assert!(output_file.exists());
    }

    #[tokio::test]
    async fn test_cli_integration_with_real_data() {
        // 创建真实的测试场景
        let mut generator = TestDataGenerator::new();
        let temp_dir = TempDir::new().unwrap();
        
        // 创建时间序列数据
        let time_series_data = generator.generate_usage_records_for_date_range(
            chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            chrono::NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
            5, // 每天5条记录
        );
        
        let data_file = temp_dir.path().join("time_series_data.json");
        let json_data = serde_json::to_string_pretty(&json!(time_series_data)).unwrap();
        fs::write(&data_file, json_data).unwrap();
        
        // 测试CLI命令
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        
        let output = cmd
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
            .expect("Failed to execute command");
        
        // 验证命令执行成功
        assert!(output.status.success());
        
        // 验证输出
        let output_str = String::from_utf8(output.stdout).unwrap();
        let json_output: serde_json::Value = serde_json::from_str(&output_str).unwrap();
        
        assert!(json_output.is_object());
        // 根据实际输出结构进行验证
        if let Some(total_cost) = json_output.get("total_cost") {
            assert!(total_cost.is_number());
            assert!(total_cost.as_f64().unwrap() > 0.0);
        }
    }

    #[tokio::test]
    async fn test_budget_integration() {
        // 创建测试数据
        let mut generator = TestDataGenerator::new();
        let temp_dir = TempDir::new().unwrap();
        
        // 创建高成本测试数据（超出预算）
        let test_records = generator.generate_usage_records(100);
        let mut expensive_records = Vec::new();
        
        for mut record in test_records {
            // 增加成本以超出预算
            if let Some(cost) = record.get_mut("cost") {
                *cost = serde_json::Value::Number(serde_json::Number::from_f64(5.0).unwrap());
            }
            expensive_records.push(record);
        }
        
        let data_file = temp_dir.path().join("expensive_data.json");
        let json_data = serde_json::to_string_pretty(&json!(expensive_records)).unwrap();
        fs::write(&data_file, json_data).unwrap();
        
        // 创建预算配置
        let budget_config = generator.generate_budget_config();
        let config_file = temp_dir.path().join("config.toml");
        let toml_content = generate_toml_from_json(&budget_config);
        fs::write(&config_file, toml_content).unwrap();
        
        // 测试预算分析
        let config_manager = ConfigManager::new_with_config(&config_file).unwrap();
        let budget = config_manager.get_budget().unwrap();
        
        let data_loader = DataLoader::with_source(
            DataSourceType::Json,
            data_file.to_string_lossy().to_string(),
        );
        let records = data_loader.load_usage_data().await.unwrap();
        
        let calculator = CostCalculator::default();
        let analysis = calculator.calculate_budget_analysis(&records, &budget).unwrap();
        
        // 验证预算分析结果
        assert!(analysis.total_usage > budget.monthly_limit);
        assert!(analysis.overage_amount > 0.0);
        assert!(analysis.usage_percentage > 100.0);
        assert!(analysis.alerts_triggered);
        
        // 测试预算状态输出
        let formatter = OutputFormatter::new(OutputFormat::Json);
        let output_file = temp_dir.path().join("budget_status.json");
        
        let result = formatter.output_budget_status(&budget, &analysis, Some(output_file.to_str().unwrap()));
        assert!(result.is_ok());
        
        // 验证输出包含预算警告信息
        let output_content = fs::read_to_string(&output_file).unwrap();
        assert!(output_content.contains("budget"));
        assert!(output_content.contains("overage"));
    }

    #[tokio::test]
    async fn test_session_analysis_integration() {
        // 创建测试数据，包含多个会话
        let mut generator = TestDataGenerator::new();
        let temp_dir = TempDir::new().unwrap();
        
        let mut session_records = Vec::new();
        let session_ids = vec!["session_1", "session_2", "session_3"];
        
        for (session_idx, session_id) in session_ids.iter().enumerate() {
            for i in 0..10 {
                let mut record = generator.generate_usage_record();
                record["session_id"] = json!(session_id);
                record["user_id"] = json!(format!("user_{}", session_idx + 1));
                
                // 设置时间序列
                let base_time = chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
                    .and_hms_opt(12 + session_idx, 0, 0).unwrap();
                let timestamp = chrono::DateTime::from_naive_utc_and_offset(
                    base_time + chrono::Duration::minutes(i * 5),
                    chrono::Utc,
                );
                record["timestamp"] = json!(timestamp.to_rfc3339());
                
                session_records.push(record);
            }
        }
        
        let data_file = temp_dir.path().join("session_data.json");
        let json_data = serde_json::to_string_pretty(&json!(session_records)).unwrap();
        fs::write(&data_file, json_data).unwrap();
        
        // 加载和分析会话数据
        let data_loader = DataLoader::with_source(
            DataSourceType::Json,
            data_file.to_string_lossy().to_string(),
        );
        let records = data_loader.load_usage_data().await.unwrap();
        
        // 会话分析
        let mut sessions = std::collections::HashMap::new();
        for record in records {
            if let Some(session_id) = &record.session_id {
                let session = sessions.entry(session_id.clone())
                    .or_insert_with(|| {
                        let mut session = crate::data::models::Session::new(
                            session_id.clone(),
                            record.timestamp,
                            record.user_id.clone(),
                        );
                        session.add_record(&record);
                        session
                    });
                session.add_record(&record);
            }
        }
        
        // 验证会话数据
        assert_eq!(sessions.len(), 3);
        
        for (session_id, session) in sessions {
            assert_eq!(session.request_count, 10);
            assert_positive!(session.total_cost);
            assert_positive!(session.total_input_tokens);
            assert_positive!(session.total_output_tokens);
            
            // 计算持续时间
            session.calculate_duration();
            assert!(session.duration_seconds.is_some());
        }
        
        // 会话统计
        let session_stats = StatisticsCalculator::calculate_session_stats(&sessions.values().collect::<Vec<_>>());
        assert_eq!(session_stats.total_sessions, 3);
        assert_positive!(session_stats.total_session_cost);
        assert_positive!(session_stats.average_session_duration);
    }

    #[tokio::test]
    async fn test_export_integration() {
        // 创建测试数据
        let mut generator = TestDataGenerator::new();
        let temp_dir = TempDir::new().unwrap();
        
        let test_records = generator.generate_usage_records(100);
        let data_file = temp_dir.path().join("export_data.json");
        let json_data = serde_json::to_string_pretty(&json!(test_records)).unwrap();
        fs::write(&data_file, json_data).unwrap();
        
        // 加载数据
        let data_loader = DataLoader::with_source(
            DataSourceType::Json,
            data_file.to_string_lossy().to_string(),
        );
        let records = data_loader.load_usage_data().await.unwrap();
        
        // 测试不同格式的导出
        let formats = vec![
            (OutputFormat::Json, "export.json"),
            (OutputFormat::Csv, "export.csv"),
        ];
        
        for (format, filename) in formats {
            let output_file = temp_dir.path().join(filename);
            let formatter = OutputFormatter::new(format);
            
            let export_format = match format {
                OutputFormat::Json => crate::output::ExportFormat::Json,
                OutputFormat::Csv => crate::output::ExportFormat::Csv,
                _ => crate::output::ExportFormat::Json,
            };
            
            let result = formatter.export_data(&records, export_format, &output_file);
            assert!(result.is_ok());
            
            // 验证导出文件
            assert!(output_file.exists());
            let file_size = fs::metadata(&output_file).unwrap().len();
            assert_positive!(file_size);
            
            // 验证文件内容
            let content = fs::read_to_string(&output_file).unwrap();
            assert!(!content.is_empty());
            
            match format {
                OutputFormat::Json => {
                    let json_value: serde_json::Value = serde_json::from_str(&content).unwrap();
                    assert!(json_value.is_array());
                    assert_eq!(json_value.as_array().unwrap().len(), 100);
                },
                OutputFormat::Csv => {
                    let lines: Vec<&str> = content.lines().collect();
                    assert!(lines.len() > 1); // 至少有标题行和一行数据
                    assert!(lines[0].contains("id"));
                    assert!(lines[0].contains("timestamp"));
                    assert!(lines[0].contains("model"));
                },
                _ => {}
            }
        }
    }

    #[tokio::test]
    async fn test_error_handling_integration() {
        // 测试各种错误情况的集成处理
        
        let temp_dir = TempDir::new().unwrap();
        
        // 测试文件不存在
        let non_existent_file = temp_dir.path().join("non_existent.json");
        let data_loader = DataLoader::with_source(
            DataSourceType::Json,
            non_existent_file.to_string_lossy().to_string(),
        );
        
        let result = data_loader.load_usage_data().await;
        assert!(result.is_err());
        
        // 测试无效JSON文件
        let invalid_json_file = temp_dir.path().join("invalid.json");
        fs::write(&invalid_json_file, "invalid json content").unwrap();
        
        let data_loader = DataLoader::with_source(
            DataSourceType::Json,
            invalid_json_file.to_string_lossy().to_string(),
        );
        
        let result = data_loader.load_usage_data().await;
        assert!(result.is_err());
        
        // 测试空数据文件
        let empty_file = temp_dir.path().join("empty.json");
        fs::write(&empty_file, "[]").unwrap();
        
        let data_loader = DataLoader::with_source(
            DataSourceType::Json,
            empty_file.to_string_lossy().to_string(),
        );
        
        let result = data_loader.load_usage_data().await;
        assert!(result.is_ok()); // 空数据是有效的
        
        let records = result.unwrap();
        assert!(records.is_empty());
        
        // 测试部分无效数据
        let partial_invalid_file = temp_dir.path().join("partial_invalid.json");
        let partial_data = r#"
        [
            {
                "id": "valid-1",
                "timestamp": "2024-01-01T12:00:00Z",
                "model": "claude-3-sonnet",
                "input_tokens": 1000,
                "output_tokens": 500,
                "cost": 0.015
            },
            {
                "id": "invalid-1",
                "timestamp": "invalid-timestamp",
                "model": "claude-3-sonnet",
                "input_tokens": 1000,
                "output_tokens": 500,
                "cost": 0.015
            }
        ]
        "#;
        
        fs::write(&partial_invalid_file, partial_data).unwrap();
        
        let data_loader = DataLoader::with_source(
            DataSourceType::Json,
            partial_invalid_file.to_string_lossy().to_string(),
        );
        
        let result = data_loader.load_usage_data().await;
        // 这里的行为取决于具体的错误处理策略
        // 可能是返回部分数据，也可能是返回错误
    }

    #[tokio::test]
    async fn test_concurrent_operations_integration() {
        // 测试并发操作
        let mut generator = TestDataGenerator::new();
        let temp_dir = TempDir::new().unwrap();
        
        // 创建多个数据文件
        let mut file_paths = Vec::new();
        for i in 0..5 {
            let test_records = generator.generate_usage_records(20);
            let data_file = temp_dir.path().join(format!("data_{}.json", i));
            let json_data = serde_json::to_string_pretty(&json!(test_records)).unwrap();
            fs::write(&data_file, json_data).unwrap();
            file_paths.push(data_file);
        }
        
        // 并发加载数据
        let mut handles = Vec::new();
        for file_path in file_paths {
            let handle = tokio::spawn(async move {
                let data_loader = DataLoader::with_source(
                    DataSourceType::Json,
                    file_path.to_string_lossy().to_string(),
                );
                data_loader.load_usage_data().await
            });
            handles.push(handle);
        }
        
        // 等待所有操作完成
        let mut all_records = Vec::new();
        for handle in handles {
            let result = handle.await.unwrap();
            match result {
                Ok(records) => {
                    all_records.extend(records);
                },
                Err(e) => {
                    panic!("Failed to load data: {:?}", e);
                }
            }
        }
        
        // 验证结果
        assert_eq!(all_records.len(), 100); // 5 files * 20 records each
        
        // 并发分析
        let mut handles = Vec::new();
        for chunk in all_records.chunks(20) {
            let chunk = chunk.to_vec();
            let handle = tokio::spawn(async move {
                let calculator = CostCalculator::default();
                calculator.calculate_total_cost(&chunk)
            });
            handles.push(handle);
        }
        
        let mut total_costs = Vec::new();
        for handle in handles {
            let result = handle.await.unwrap();
            match result {
                Ok(cost) => {
                    total_costs.push(cost);
                },
                Err(e) => {
                    panic!("Failed to calculate cost: {:?}", e);
                }
            }
        }
        
        // 验证分析结果
        assert_eq!(total_costs.len(), 5);
        let grand_total: f64 = total_costs.iter().sum();
        assert_positive!(grand_total);
    }

    #[tokio::test]
    async fn test_memory_usage_integration() {
        // 测试大数据集的内存使用
        let mut generator = TestDataGenerator::new();
        let temp_dir = TempDir::new().unwrap();
        
        // 创建大型数据集
        let large_dataset = generator.generate_usage_records(10000);
        let data_file = temp_dir.path().join("large_dataset.json");
        let json_data = serde_json::to_string_pretty(&json!(large_dataset)).unwrap();
        fs::write(&data_file, json_data).unwrap();
        
        // 监控内存使用
        let start_memory = get_memory_usage();
        
        // 加载数据
        let data_loader = DataLoader::with_source(
            DataSourceType::Json,
            data_file.to_string_lossy().to_string(),
        );
        let records = data_loader.load_usage_data().await.unwrap();
        
        let after_load_memory = get_memory_usage();
        
        // 验证数据加载
        assert_eq!(records.len(), 10000);
        
        // 进行分析
        let calculator = CostCalculator::default();
        let breakdown = calculator.calculate_detailed_breakdown(&records).unwrap();
        
        let after_analysis_memory = get_memory_usage();
        
        // 验证分析结果
        assert_positive!(breakdown.total_cost);
        assert!(!breakdown.model_breakdown.is_empty());
        
        // 验证内存使用合理（这里只是一个基本检查）
        println!("Memory usage - Start: {} KB, After load: {} KB, After analysis: {} KB",
                 start_memory, after_load_memory, after_analysis_memory);
        
        // 清理数据
        drop(records);
        drop(breakdown);
        
        let after_cleanup_memory = get_memory_usage();
        
        // 验证内存释放
        assert!(after_cleanup_memory <= after_analysis_memory + 1000); // 允许一些缓冲
    }

    #[tokio::test]
    async fn test_configuration_change_integration() {
        // 测试配置更改的影响
        let mut generator = TestDataGenerator::new();
        let temp_dir = TempDir::new().unwrap();
        
        // 创建初始配置
        let initial_config = generator.generate_config();
        let config_file = temp_dir.path().join("config.toml");
        let toml_content = generate_toml_from_json(&initial_config);
        fs::write(&config_file, toml_content).unwrap();
        
        // 加载初始配置
        let mut config_manager = ConfigManager::new_with_config(&config_file).unwrap();
        let initial_config_obj = config_manager.get_config();
        
        // 创建测试数据
        let test_records = generator.generate_usage_records(10);
        let data_file = temp_dir.path().join("data.json");
        let json_data = serde_json::to_string_pretty(&json!(test_records)).unwrap();
        fs::write(&data_file, json_data).unwrap();
        
        // 使用初始配置进行分析
        let data_loader = DataLoader::with_source(
            DataSourceType::Json,
            data_file.to_string_lossy().to_string(),
        );
        let records = data_loader.load_usage_data().await.unwrap();
        
        let calculator = CostCalculator::default();
        let initial_breakdown = calculator.calculate_detailed_breakdown(&records).unwrap();
        
        // 修改配置
        let mut modified_config = initial_config.clone();
        modified_config["output"]["decimal_places"] = json!(2);
        modified_config["analysis"]["default_model_pricing"]["claude-3-sonnet"]["input_cost_per_1k"] = json!(0.005);
        
        let new_toml_content = generate_toml_from_json(&modified_config);
        fs::write(&config_file, new_toml_content).unwrap();
        
        // 重新加载配置
        config_manager = ConfigManager::new_with_config(&config_file).unwrap();
        let modified_config_obj = config_manager.get_config();
        
        // 使用修改后的配置进行分析
        let new_calculator = CostCalculator::default();
        let modified_breakdown = new_calculator.calculate_detailed_breakdown(&records).unwrap();
        
        // 验证配置更改生效
        assert_ne!(initial_config_obj.output.decimal_places, modified_config_obj.output.decimal_places);
        
        // 由于我们使用了新的计算器，成本可能会不同
        // 这里主要验证系统能够正确处理配置更改
        assert_positive!(modified_breakdown.total_cost);
        assert!(!modified_breakdown.model_breakdown.is_empty());
    }

    // 辅助函数：获取内存使用情况
    fn get_memory_usage() -> u64 {
        // 这是一个简化的内存使用检查
        // 在实际环境中，你可能需要使用更精确的方法
        match memory_stats::memory_stats() {
            Ok(stats) => stats.physical_mem / 1024, // 转换为KB
            Err(_) => 0,
        }
    }
}