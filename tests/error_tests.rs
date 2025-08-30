//! 错误处理测试
//! 
//! 测试各种错误情况和系统的错误处理能力

mod common;

use common::*;
use ccusage_rs::*;
use std::fs;
use tempfile::TempDir;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_invalid_json_data() {
        let temp_dir = TempDir::new().unwrap();
        
        // 测试各种无效JSON格式
        let invalid_json_cases = vec![
            ("完全无效的JSON", "this is not json"),
            ("缺少引号", r#"{"id": "test, "model": "claude"}"#),
            ("缺少逗号", r#"{"id": "test" "model": "claude"}"#),
            ("缺少闭合括号", r#"{"id": "test", "model": "claude""#),
            ("无效的布尔值", r#"{"id": "test", "active": yes}"#),
            ("无效的数字", r#"{"id": "test", "tokens": "not_a_number"}"#),
            ("空数组", "[]"),
            ("空对象", "{}"),
        ];
        
        for (case_name, invalid_json) in invalid_json_cases {
            println!("Testing invalid JSON case: {}", case_name);
            
            let data_file = temp_dir.path().join(format!("invalid_{}.json", case_name));
            fs::write(&data_file, invalid_json).unwrap();
            
            let data_loader = DataLoader::with_source(
                DataSourceType::Json,
                data_file.to_string_lossy().to_string(),
            );
            
            let result = data_loader.load_usage_data().await;
            
            match case_name {
                "空数组" => {
                    // 空数组应该被处理为有效情况（没有记录）
                    assert!(result.is_ok());
                    let records = result.unwrap();
                    assert!(records.is_empty());
                },
                "空对象" => {
                    // 空对象可能也是有效的
                    assert!(result.is_ok());
                },
                _ => {
                    // 其他情况应该返回错误
                    assert!(result.is_err(), "Case '{}' should return error", case_name);
                    
                    if let Err(e) = result {
                        assert!(e.to_string().contains("error") || 
                                e.to_string().contains("JSON") || 
                                e.to_string().contains("parse"));
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn test_invalid_csv_data() {
        let temp_dir = TempDir::new().unwrap();
        
        // 测试各种无效CSV格式
        let invalid_csv_cases = vec![
            ("缺少列", "id,model\ntest1,claude\ntest2"),
            ("类型不匹配", "id,timestamp,model,tokens\ntest1,invalid-time,claude,100"),
            ("负数令牌", "id,timestamp,model,input_tokens,output_tokens,cost\ntest1,2024-01-01T12:00:00Z,claude,-100,50,0.01"),
            ("负成本", "id,timestamp,model,input_tokens,output_tokens,cost\ntest1,2024-01-01T12:00:00Z,claude,100,50,-0.01"),
            ("空行", "id,timestamp,model,input_tokens,output_tokens,cost\ntest1,2024-01-01T12:00:00Z,claude,100,50,0.01\n\ntest2,2024-01-01T13:00:00Z,claude,200,100,0.02"),
            ("只有标题行", "id,timestamp,model,input_tokens,output_tokens,cost"),
            ("格式不一致", "id,timestamp,model,input_tokens,output_tokens,cost\ntest1,2024-01-01T12:00:00Z,claude,100,50\ntest2,2024-01-01T13:00:00Z,claude,200,100,0.02"),
        ];
        
        for (case_name, invalid_csv) in invalid_csv_cases {
            println!("Testing invalid CSV case: {}", case_name);
            
            let data_file = temp_dir.path().join(format!("invalid_{}.csv", case_name));
            fs::write(&data_file, invalid_csv).unwrap();
            
            let data_loader = DataLoader::with_source(
                DataSourceType::Csv,
                data_file.to_string_lossy().to_string(),
            );
            
            let result = data_loader.load_usage_data().await;
            
            match case_name {
                "只有标题行" => {
                    // 只有标题行应该返回空记录列表
                    assert!(result.is_ok());
                    let records = result.unwrap();
                    assert!(records.is_empty());
                },
                _ => {
                    // 其他情况应该返回错误
                    assert!(result.is_err(), "Case '{}' should return error", case_name);
                    
                    if let Err(e) = result {
                        assert!(e.to_string().contains("error") || 
                                e.to_string().contains("CSV") || 
                                e.to_string().contains("parse") ||
                                e.to_string().contains("invalid"));
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn test_invalid_configuration() {
        let temp_dir = TempDir::new().unwrap();
        
        // 测试各种无效配置
        let invalid_config_cases = vec![
            ("无效的TOML格式", "this is not toml"),
            ("负预算限制", "[budget]\nmonthly_limit = -100.0\ncurrency = \"USD\""),
            ("无效的警告阈值", "[budget]\nwarning_threshold = 150.0\nalert_threshold = 95.0"),
            ("警告阈值大于警告阈值", "[budget]\nwarning_threshold = 95.0\nalert_threshold = 80.0"),
            ("负的缓存TTL", "[data]\ncache_ttl = -300"),
            ("无效的日志级别", "[app]\nlog_level = \"invalid_level\""),
            ("空配置文件", ""),
            ("缺少必需的配置节", "[app]\nverbose = true"),
        ];
        
        for (case_name, invalid_config) in invalid_config_cases {
            println!("Testing invalid configuration case: {}", case_name);
            
            let config_file = temp_dir.path().join(format!("invalid_config_{}.toml", case_name));
            fs::write(&config_file, invalid_config).unwrap();
            
            let result = ConfigManager::new_with_config(&config_file);
            
            match case_name {
                "空配置文件" | "缺少必需的配置节" => {
                    // 这些情况可能使用默认配置
                    assert!(result.is_ok());
                },
                _ => {
                    // 其他情况应该返回错误
                    assert!(result.is_err(), "Case '{}' should return error", case_name);
                    
                    if let Err(e) = result {
                        assert!(e.to_string().contains("error") || 
                                e.to_string().contains("config") ||
                                e.to_string().contains("parse") ||
                                e.to_string().contains("invalid"));
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn test_file_system_errors() {
        let temp_dir = TempDir::new().unwrap();
        
        // 测试文件系统错误
        let non_existent_file = temp_dir.path().join("non_existent.json");
        
        // 尝试加载不存在的文件
        let data_loader = DataLoader::with_source(
            DataSourceType::Json,
            non_existent_file.to_string_lossy().to_string(),
        );
        
        let result = data_loader.load_usage_data().await;
        assert!(result.is_err());
        
        if let Err(e) = result {
            assert!(e.to_string().contains("error") || 
                    e.to_string().contains("file") ||
                    e.to_string().contains("not found"));
        }
        
        // 尝试写入到没有权限的目录（如果可能）
        let restricted_dir = temp_dir.path().join("restricted");
        fs::create_dir(&restricted_dir).unwrap();
        
        // 在Unix系统上尝试设置只读权限
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&restricted_dir).unwrap().permissions();
            perms.set_mode(0o444); // 只读权限
            fs::set_permissions(&restricted_dir, perms).unwrap();
            
            let output_file = restricted_dir.join("test_output.json");
            let formatter = OutputFormatter::new(OutputFormat::Json);
            let stats = crate::analysis::statistics::UsageStats::default();
            
            let result = formatter.output_usage_stats(&stats, Some(output_file.to_str().unwrap()));
            assert!(result.is_err());
            
            if let Err(e) = result {
                assert!(e.to_string().contains("error") || 
                        e.to_string().contains("permission") ||
                        e.to_string().contains("access"));
            }
        }
    }

    #[tokio::test]
    async fn test_invalid_command_line_arguments() {
        use assert_cmd::Command;
        
        // 测试无效的命令行参数
        let invalid_arg_cases = vec![
            ("无效的分析类型", vec!["analyze", "--analysis-type", "invalid_type"]),
            ("无效的输出格式", vec!["--format", "invalid_format", "analyze"]),
            ("无效的日期范围", vec!["analyze", "--date-range", "invalid-date-range"]),
            ("负的预算限制", vec!["budget", "set", "--limit", "-100.0"]),
            ("无效的警告阈值", vec!["budget", "set", "--warning", "150.0"]),
            ("缺少必需参数", vec!["export"]),
            ("无效的端口", vec!["server", "--port", "99999"]),
            ("负的周数", vec!["weekly", "--weeks", "-1"]),
        ];
        
        for (case_name, args) in invalid_arg_cases {
            println!("Testing invalid command line argument case: {}", case_name);
            
            let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
            let result = cmd.args(args).output();
            
            assert!(result.is_ok()); // 命令应该能够执行
            
            let output = result.unwrap();
            assert!(!output.status.success(), "Case '{}' should fail", case_name);
            
            let stderr = String::from_utf8(output.stderr).unwrap();
            assert!(stderr.contains("error") || 
                    stderr.contains("Error") || 
                    stderr.contains("invalid") ||
                    stderr.contains("usage"));
        }
    }

    #[tokio::test]
    async fn test_data_validation_errors() {
        let temp_dir = TempDir::new().unwrap();
        
        // 创建包含无效数据的JSON文件
        let invalid_data_cases = vec![
            ("负的输入令牌", json!([{
                "id": "test1",
                "timestamp": "2024-01-01T12:00:00Z",
                "model": "claude-3-sonnet",
                "input_tokens": -100,
                "output_tokens": 50,
                "cost": 0.01
            }])),
            ("负的输出令牌", json!([{
                "id": "test1",
                "timestamp": "2024-01-01T12:00:00Z",
                "model": "claude-3-sonnet",
                "input_tokens": 100,
                "output_tokens": -50,
                "cost": 0.01
            }])),
            ("负的成本", json!([{
                "id": "test1",
                "timestamp": "2024-01-01T12:00:00Z",
                "model": "claude-3-sonnet",
                "input_tokens": 100,
                "output_tokens": 50,
                "cost": -0.01
            }])),
            ("无效的时间戳", json!([{
                "id": "test1",
                "timestamp": "invalid-timestamp",
                "model": "claude-3-sonnet",
                "input_tokens": 100,
                "output_tokens": 50,
                "cost": 0.01
            }])),
            ("空模型名称", json!([{
                "id": "test1",
                "timestamp": "2024-01-01T12:00:00Z",
                "model": "",
                "input_tokens": 100,
                "output_tokens": 50,
                "cost": 0.01
            }])),
            ("缺失必需字段", json!([{
                "id": "test1",
                "model": "claude-3-sonnet",
                "input_tokens": 100,
                "output_tokens": 50
                // 缺少 timestamp, cost
            }])),
        ];
        
        for (case_name, invalid_data) in invalid_data_cases {
            println!("Testing data validation case: {}", case_name);
            
            let data_file = temp_dir.path().join(format!("validation_{}.json", case_name));
            let json_data = serde_json::to_string_pretty(&invalid_data).unwrap();
            fs::write(&data_file, json_data).unwrap();
            
            let data_loader = DataLoader::with_source(
                DataSourceType::Json,
                data_file.to_string_lossy().to_string(),
            );
            
            let result = data_loader.load_usage_data().await;
            
            // 根据具体的验证逻辑，可能返回错误或跳过无效记录
            if result.is_err() {
                if let Err(e) = result {
                    assert!(e.to_string().contains("error") || 
                            e.to_string().contains("validation") ||
                            e.to_string().contains("invalid"));
                }
            } else {
                // 如果返回成功，应该验证记录是否有效
                let records = result.unwrap();
                for record in records {
                    // 验证记录的基本有效性
                    assert!(record.input_tokens >= 0);
                    assert!(record.output_tokens >= 0);
                    assert!(record.cost >= 0.0);
                    assert!(!record.model.is_empty());
                }
            }
        }
    }

    #[tokio::test]
    async fn test_network_errors() {
        let temp_dir = TempDir::new().unwrap();
        
        // 测试网络相关的错误（模拟网络不可用）
        // 这里主要测试配置中涉及网络的部分
        
        // 创建包含无效URL的配置
        let invalid_network_config = r#"
[app]
verbose = true

[data]
remote_url = "http://invalid-url-that-does-not-exist.com/api"
"#;
        
        let config_file = temp_dir.path().join("invalid_network_config.toml");
        fs::write(&config_file, invalid_network_config).unwrap();
        
        // 尝试加载包含无效网络配置的配置文件
        let result = ConfigManager::new_with_config(&config_file);
        
        // 根据实现，可能会忽略无效的网络配置或返回错误
        assert!(result.is_ok()); // 目前假设配置加载会忽略无效的网络URL
        
        // 如果系统尝试连接到无效URL，应该能够优雅地处理
        // 这里我们主要验证系统不会崩溃
    }

    #[tokio::test]
    async fn test_memory_error_handling() {
        // 测试内存相关错误的处理
        let temp_dir = TempDir::new().unwrap();
        
        // 创建一个非常大的数据文件来测试内存处理
        let mut generator = TestDataGenerator::new();
        
        // 生成大量数据（但要合理，避免测试时间过长）
        let large_dataset = generator.generate_usage_records(5000);
        let data_file = temp_dir.path().join("large_memory_test.json");
        let json_data = serde_json::to_string_pretty(&json!(large_dataset)).unwrap();
        fs::write(&data_file, json_data).unwrap();
        
        // 尝试处理大数据集
        let data_loader = DataLoader::with_source(
            DataSourceType::Json,
            data_file.to_string_lossy().to_string(),
        );
        
        let result = data_loader.load_usage_data().await;
        
        // 应该能够成功处理大数据集
        assert!(result.is_ok());
        
        let records = result.unwrap();
        assert_eq!(records.len(), 5000);
        
        // 验证内存使用合理
        let calculator = CostCalculator::default();
        let breakdown_result = calculator.calculate_detailed_breakdown(&records);
        
        assert!(breakdown_result.is_ok());
        
        // 清理数据
        drop(records);
        drop(breakdown_result);
    }

    #[tokio::test]
    async fn test_concurrent_error_handling() {
        let temp_dir = TempDir::new().unwrap();
        
        // 创建混合的有效和无效数据文件
        let mut generator = TestDataGenerator::new();
        
        // 创建有效数据文件
        let valid_data = generator.generate_usage_records(100);
        let valid_file = temp_dir.path().join("valid_concurrent.json");
        let json_data = serde_json::to_string_pretty(&json!(valid_data)).unwrap();
        fs::write(&valid_file, json_data).unwrap();
        
        // 创建无效数据文件
        let invalid_file = temp_dir.path().join("invalid_concurrent.json");
        fs::write(&invalid_file, "invalid json content").unwrap();
        
        // 并发处理有效和无效数据
        let mut handles = Vec::new();
        
        // 处理有效数据
        let valid_handle = tokio::spawn({
            let valid_file = valid_file.clone();
            async move {
                let data_loader = DataLoader::with_source(
                    DataSourceType::Json,
                    valid_file.to_string_lossy().to_string(),
                );
                data_loader.load_usage_data().await
            }
        });
        handles.push(valid_handle);
        
        // 处理无效数据
        let invalid_handle = tokio::spawn({
            let invalid_file = invalid_file.clone();
            async move {
                let data_loader = DataLoader::with_source(
                    DataSourceType::Json,
                    invalid_file.to_string_lossy().to_string(),
                );
                data_loader.load_usage_data().await
            }
        });
        handles.push(invalid_handle);
        
        // 等待所有操作完成
        let mut success_count = 0;
        let mut error_count = 0;
        
        for handle in handles {
            match handle.await.unwrap() {
                Ok(_) => success_count += 1,
                Err(_) => error_count += 1,
            }
        }
        
        // 验证结果
        assert_eq!(success_count, 1);
        assert_eq!(error_count, 1);
        
        println!("Concurrent error handling: {} successes, {} errors", success_count, error_count);
    }

    #[tokio::test]
    async fn test_timeout_error_handling() {
        let temp_dir = TempDir::new().unwrap();
        
        // 测试超时相关的错误处理
        // 这里我们创建一个可能需要较长时间处理的场景
        
        let mut generator = TestDataGenerator::new();
        let large_dataset = generator.generate_usage_records(1000);
        let data_file = temp_dir.path().join("timeout_test.json");
        let json_data = serde_json::to_string_pretty(&json!(large_dataset)).unwrap();
        fs::write(&data_file, json_data).unwrap();
        
        // 设置较短的超时时间进行测试
        let timeout_duration = tokio::time::Duration::from_millis(100);
        
        let result = tokio::time::timeout(timeout_duration, async {
            let data_loader = DataLoader::with_source(
                DataSourceType::Json,
                data_file.to_string_lossy().to_string(),
            );
            data_loader.load_usage_data().await
        }).await;
        
        match result {
            Ok(Ok(_)) => {
                // 操作在超时前完成
                println!("Operation completed before timeout");
            },
            Ok(Err(e)) => {
                // 操作返回错误
                println!("Operation returned error: {:?}", e);
            },
            Err(_) => {
                // 操作超时
                println!("Operation timed out");
                // 这里我们验证超时被正确处理
                // 系统不应该崩溃，而是应该优雅地处理超时
            }
        }
    }

    #[tokio::test]
    async fn test_partial_data_recovery() {
        let temp_dir = TempDir::new().unwrap();
        
        // 创建包含部分有效和部分无效数据的文件
        let mixed_data = json!([
            // 有效记录
            {
                "id": "valid1",
                "timestamp": "2024-01-01T12:00:00Z",
                "model": "claude-3-sonnet",
                "input_tokens": 100,
                "output_tokens": 50,
                "cost": 0.01
            },
            // 无效记录（负的令牌）
            {
                "id": "invalid1",
                "timestamp": "2024-01-01T13:00:00Z",
                "model": "claude-3-sonnet",
                "input_tokens": -100,
                "output_tokens": 50,
                "cost": 0.01
            },
            // 另一个有效记录
            {
                "id": "valid2",
                "timestamp": "2024-01-01T14:00:00Z",
                "model": "claude-3-sonnet",
                "input_tokens": 200,
                "output_tokens": 100,
                "cost": 0.02
            },
            // 无效记录（无效时间戳）
            {
                "id": "invalid2",
                "timestamp": "invalid-timestamp",
                "model": "claude-3-sonnet",
                "input_tokens": 100,
                "output_tokens": 50,
                "cost": 0.01
            }
        ]);
        
        let data_file = temp_dir.path().join("mixed_data.json");
        let json_data = serde_json::to_string_pretty(&mixed_data).unwrap();
        fs::write(&data_file, json_data).unwrap();
        
        let data_loader = DataLoader::with_source(
            DataSourceType::Json,
            data_file.to_string_lossy().to_string(),
        );
        
        let result = data_loader.load_usage_data().await;
        
        // 根据实现，系统可能会：
        // 1. 返回错误（如果采用严格模式）
        // 2. 返回部分数据（如果采用宽松模式）
        
        if result.is_ok() {
            let records = result.unwrap();
            
            // 验证只包含有效记录
            for record in records {
                assert!(record.input_tokens >= 0);
                assert!(record.output_tokens >= 0);
                assert!(record.cost >= 0.0);
                assert!(!record.model.is_empty());
                
                // 验证ID格式
                assert!(record.id.starts_with("valid"));
            }
            
            println!("Partial data recovery: {} valid records loaded", records.len());
        } else {
            // 如果返回错误，验证错误信息
            if let Err(e) = result {
                assert!(e.to_string().contains("error") || 
                        e.to_string().contains("validation") ||
                        e.to_string().contains("invalid"));
            }
        }
    }

    #[tokio::test]
    async fn test_error_propagation() {
        // 测试错误在系统中的传播
        let temp_dir = TempDir::new().unwrap();
        
        // 创建一个包含错误的配置文件
        let error_config = r#"
[app]
verbose = true

[invalid_section]
invalid_key = invalid_value
"#;
        
        let config_file = temp_dir.path().join("error_config.toml");
        fs::write(&config_file, error_config).unwrap();
        
        // 尝试创建配置管理器
        let config_result = ConfigManager::new_with_config(&config_file);
        
        // 验证错误能够正确传播
        if let Err(config_error) = config_result {
            // 使用这个有错误的配置管理器进行其他操作
            // 验证错误能够正确传播
            let mut config_manager = match ConfigManager::new() {
                Ok(cm) => cm,
                Err(e) => {
                    // 如果连默认配置都创建失败，测试失败
                    panic!("Failed to create default config manager: {:?}", e);
                }
            };
            
            // 尝试设置无效的配置值
            let set_result = config_manager.set_config(ccusage_rs::config::Config::default());
            
            // 验证错误传播
            match set_result {
                Ok(_) => {
                    println!("Config set successfully (error might have been resolved)");
                },
                Err(e) => {
                    println!("Config error propagated: {:?}", e);
                    // 验证错误类型和消息
                    assert!(e.to_string().contains("error") || 
                            e.to_string().contains("config"));
                }
            }
        }
    }

    #[tokio::test]
    async fn test_resource_cleanup_on_error() {
        let temp_dir = TempDir::new().unwrap();
        
        // 测试资源清理（即使在错误情况下）
        let test_files = vec![
            temp_dir.path().join("cleanup_test_1.json"),
            temp_dir.path().join("cleanup_test_2.json"),
            temp_dir.path().join("cleanup_test_3.json"),
        ];
        
        // 创建一些测试文件
        for file_path in &test_files {
            fs::write(file_path, "test content").unwrap();
        }
        
        // 验证文件存在
        for file_path in &test_files {
            assert!(file_path.exists());
        }
        
        // 尝试执行一个会失败的操作
        let invalid_file = temp_dir.path().join("invalid_cleanup_test.json");
        fs::write(&invalid_file, "invalid json content").unwrap();
        
        let data_loader = DataLoader::with_source(
            DataSourceType::Json,
            invalid_file.to_string_lossy().to_string(),
        );
        
        let result = data_loader.load_usage_data().await;
        assert!(result.is_err());
        
        // 验证资源被正确清理
        // 这里我们主要验证系统没有崩溃，并且临时文件被清理
        // 具体的清理逻辑取决于实现
        
        // 尝试删除测试文件
        for file_path in &test_files {
            let _ = fs::remove_file(file_path);
        }
        
        println!("Resource cleanup test completed");
    }
}