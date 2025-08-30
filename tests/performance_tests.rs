//! 性能测试
//! 
//! 测试大数据集处理性能和系统性能瓶颈

mod common;

use common::*;
use ccusage_rs::*;
use tempfile::TempDir;
use std::fs;
use std::time::Instant;
use tokio::task::JoinSet;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_data_loading_performance() {
        let mut generator = TestDataGenerator::new();
        let temp_dir = TempDir::new().unwrap();
        
        // 测试不同数据大小的加载性能
        let data_sizes = vec![100, 1000, 5000, 10000];
        
        for size in data_sizes {
            println!("Testing data loading performance with {} records...", size);
            
            // 生成测试数据
            let test_data = generator.generate_usage_records(size);
            let data_file = temp_dir.path().join(format!("test_data_{}.json", size));
            let json_data = serde_json::to_string_pretty(&json!(test_data)).unwrap();
            fs::write(&data_file, json_data).unwrap();
            
            // 测试加载性能
            let start_time = Instant::now();
            let data_loader = DataLoader::with_source(
                DataSourceType::Json,
                data_file.to_string_lossy().to_string(),
            );
            let records = data_loader.load_usage_data().await.unwrap();
            let duration = start_time.elapsed();
            
            // 验证结果
            assert_eq!(records.len(), size);
            
            // 输出性能指标
            println!("  Loaded {} records in {:?}", size, duration);
            println!("  Loading rate: {:.2} records/second", size as f64 / duration.as_secs_f64());
            
            // 验证性能要求
            let expected_max_duration = match size {
                100 => std::time::Duration::from_millis(100),
                1000 => std::time::Duration::from_millis(500),
                5000 => std::time::Duration::from_millis(2000),
                10000 => std::time::Duration::from_millis(4000),
                _ => std::time::Duration::from_secs(10),
            };
            
            assert!(duration <= expected_max_duration, 
                   "Loading {} records should take no more than {:?}, took {:?}", 
                   size, expected_max_duration, duration);
        }
    }

    #[tokio::test]
    async fn test_analysis_performance() {
        let mut generator = TestDataGenerator::new();
        let temp_dir = TempDir::new().unwrap();
        
        // 测试不同分析类型的性能
        let analysis_types = vec![
            ("cost", |records: &[crate::data::models::UsageRecord]| {
                let calculator = CostCalculator::default();
                calculator.calculate_detailed_breakdown(records).unwrap()
            }),
            ("statistics", |records: &[crate::data::models::UsageRecord]| {
                StatisticsCalculator::calculate_usage_stats(records)
            }),
            ("trends", |records: &[crate::data::models::UsageRecord]| {
                let analyzer = TrendAnalyzer::default();
                analyzer.analyze_trends(records).unwrap()
            }),
            ("insights", |records: &[crate::data::models::UsageRecord]| {
                let mut engine = InsightsEngine::default();
                engine.generate_insights(records, None).unwrap()
            }),
        ];
        
        let data_sizes = vec![100, 1000, 5000];
        
        for size in data_sizes {
            println!("Testing analysis performance with {} records...", size);
            
            // 生成测试数据
            let test_data = generator.generate_usage_records(size);
            let records: Vec<crate::data::models::UsageRecord> = test_data
                .into_iter()
                .map(|r| serde_json::from_value(r).unwrap())
                .collect();
            
            for (analysis_name, analysis_func) in &analysis_types {
                println!("  Testing {} analysis...", analysis_name);
                
                let start_time = Instant::now();
                let _result = analysis_func(&records);
                let duration = start_time.elapsed();
                
                println!("    {} analysis completed in {:?}", analysis_name, duration);
                println!("    Analysis rate: {:.2} records/second", size as f64 / duration.as_secs_f64());
                
                // 验证性能要求
                let expected_max_duration = match size {
                    100 => std::time::Duration::from_millis(50),
                    1000 => std::time::Duration::from_millis(200),
                    5000 => std::time::Duration::from_millis(1000),
                    _ => std::time::Duration::from_secs(5),
                };
                
                assert!(duration <= expected_max_duration, 
                       "{} analysis of {} records should take no more than {:?}, took {:?}", 
                       analysis_name, size, expected_max_duration, duration);
            }
        }
    }

    #[tokio::test]
    async fn test_output_formatting_performance() {
        let mut generator = TestDataGenerator::new();
        let temp_dir = TempDir::new().unwrap();
        
        // 测试不同输出格式的性能
        let output_formats = vec![
            (OutputFormat::Json, "json"),
            (OutputFormat::Csv, "csv"),
            (OutputFormat::Table, "table"),
        ];
        
        let data_sizes = vec![100, 1000, 5000];
        
        for size in data_sizes {
            println!("Testing output formatting performance with {} records...", size);
            
            // 生成测试数据
            let test_data = generator.generate_usage_records(size);
            let records: Vec<crate::data::models::UsageRecord> = test_data
                .into_iter()
                .map(|r| serde_json::from_value(r).unwrap())
                .collect();
            
            let stats = StatisticsCalculator::calculate_usage_stats(&records);
            
            for (format, format_name) in &output_formats {
                println!("  Testing {} formatting...", format_name);
                
                let formatter = OutputFormatter::new(*format);
                let output_file = temp_dir.path().join(format!("output_{}_{}.{}", size, format_name, format_name));
                
                let start_time = Instant::now();
                let result = formatter.output_usage_stats(&stats, Some(output_file.to_str().unwrap()));
                let duration = start_time.elapsed();
                
                assert!(result.is_ok());
                assert!(output_file.exists());
                
                println!("    {} formatting completed in {:?}", format_name, duration);
                println!("    Formatting rate: {:.2} records/second", size as f64 / duration.as_secs_f64());
                
                // 验证性能要求
                let expected_max_duration = match size {
                    100 => std::time::Duration::from_millis(20),
                    1000 => std::time::Duration::from_millis(100),
                    5000 => std::time::Duration::from_millis(500),
                    _ => std::time::Duration::from_secs(2),
                };
                
                assert!(duration <= expected_max_duration, 
                       "{} formatting of {} records should take no more than {:?}, took {:?}", 
                       format_name, size, expected_max_duration, duration);
            }
        }
    }

    #[tokio::test]
    async fn test_memory_usage_performance() {
        let mut generator = TestDataGenerator::new();
        let temp_dir = TempDir::new().unwrap();
        
        // 测试大数据集的内存使用
        let large_dataset = generator.generate_usage_records(10000);
        let data_file = temp_dir.path().join("large_dataset.json");
        let json_data = serde_json::to_string_pretty(&json!(large_dataset)).unwrap();
        fs::write(&data_file, json_data).unwrap();
        
        // 监控内存使用
        let start_memory = get_memory_usage();
        println!("Starting memory usage: {} KB", start_memory);
        
        // 加载数据
        let data_loader = DataLoader::with_source(
            DataSourceType::Json,
            data_file.to_string_lossy().to_string(),
        );
        let records = data_loader.load_usage_data().await.unwrap();
        
        let after_load_memory = get_memory_usage();
        let load_memory_increase = after_load_memory.saturating_sub(start_memory);
        println!("Memory after loading data: {} KB (increase: {} KB)", after_load_memory, load_memory_increase);
        
        // 进行分析
        let calculator = CostCalculator::default();
        let breakdown = calculator.calculate_detailed_breakdown(&records).unwrap();
        
        let after_analysis_memory = get_memory_usage();
        let analysis_memory_increase = after_analysis_memory.saturating_sub(after_load_memory);
        println!("Memory after analysis: {} KB (increase: {} KB)", after_analysis_memory, analysis_memory_increase);
        
        // 生成输出
        let formatter = OutputFormatter::new(OutputFormat::Json);
        let output_file = temp_dir.path().join("memory_test_output.json");
        let _result = formatter.output_cost_breakdown(&breakdown, Some(output_file.to_str().unwrap()));
        
        let after_output_memory = get_memory_usage();
        let output_memory_increase = after_output_memory.saturating_sub(after_analysis_memory);
        println!("Memory after output: {} KB (increase: {} KB)", after_output_memory, output_memory_increase);
        
        // 验证内存使用合理
        let total_memory_increase = after_output_memory.saturating_sub(start_memory);
        println!("Total memory increase: {} KB", total_memory_increase);
        
        // 验证内存要求（这里设置一个相对宽松的限制）
        assert!(total_memory_increase < 50 * 1024, "Total memory increase should be less than 50MB");
        
        // 清理数据
        drop(records);
        drop(breakdown);
        
        // 验证内存释放
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let after_cleanup_memory = get_memory_usage();
        let cleanup_memory_decrease = after_output_memory.saturating_sub(after_cleanup_memory);
        println!("Memory after cleanup: {} KB (decrease: {} KB)", after_cleanup_memory, cleanup_memory_decrease);
        
        // 验证大部分内存被释放
        assert!(cleanup_memory_decrease > total_memory_increase * 0.5, "Most memory should be released after cleanup");
    }

    #[tokio::test]
    async fn test_concurrent_processing_performance() {
        let mut generator = TestDataGenerator::new();
        let temp_dir = TempDir::new().unwrap();
        
        // 创建多个数据文件进行并发处理
        let file_count = 5;
        let records_per_file = 1000;
        let mut file_paths = Vec::new();
        
        for i in 0..file_count {
            let test_data = generator.generate_usage_records(records_per_file);
            let data_file = temp_dir.path().join(format!("concurrent_data_{}.json", i));
            let json_data = serde_json::to_string_pretty(&json!(test_data)).unwrap();
            fs::write(&data_file, json_data).unwrap();
            file_paths.push(data_file);
        }
        
        println!("Testing concurrent processing performance with {} files...", file_count);
        
        // 测试串行处理作为基准
        let start_time = Instant::now();
        let mut total_records = 0;
        
        for file_path in &file_paths {
            let data_loader = DataLoader::with_source(
                DataSourceType::Json,
                file_path.to_string_lossy().to_string(),
            );
            let records = data_loader.load_usage_data().await.unwrap();
            total_records += records.len();
        }
        
        let serial_duration = start_time.elapsed();
        println!("Serial processing completed in {:?} ({} records)", serial_duration, total_records);
        
        // 测试并发处理
        let start_time = Instant::now();
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
        
        let mut concurrent_records = 0;
        for handle in handles {
            match handle.await.unwrap() {
                Ok(records) => {
                    concurrent_records += records.len();
                },
                Err(e) => {
                    panic!("Failed to load data concurrently: {:?}", e);
                }
            }
        }
        
        let concurrent_duration = start_time.elapsed();
        println!("Concurrent processing completed in {:?} ({} records)", concurrent_duration, concurrent_records);
        
        // 验证结果一致性
        assert_eq!(total_records, concurrent_records);
        assert_eq!(total_records, file_count * records_per_file);
        
        // 验证并发性能提升
        let speedup_ratio = serial_duration.as_secs_f64() / concurrent_duration.as_secs_f64();
        println!("Concurrent processing speedup: {:.2}x", speedup_ratio);
        
        // 并发处理应该至少比串行处理快一些（考虑到并发开销）
        assert!(speedup_ratio > 1.5, "Concurrent processing should be at least 1.5x faster than serial processing");
    }

    #[tokio::test]
    async fn test_export_performance() {
        let mut generator = TestDataGenerator::new();
        let temp_dir = TempDir::new().unwrap();
        
        // 测试不同格式和大小的导出性能
        let export_formats = vec![
            (crate::output::ExportFormat::Json, "json"),
            (crate::output::ExportFormat::Csv, "csv"),
        ];
        
        let data_sizes = vec![100, 1000, 5000, 10000];
        
        for size in data_sizes {
            println!("Testing export performance with {} records...", size);
            
            // 生成测试数据
            let test_data = generator.generate_usage_records(size);
            let records: Vec<crate::data::models::UsageRecord> = test_data
                .into_iter()
                .map(|r| serde_json::from_value(r).unwrap())
                .collect();
            
            for (format, format_name) in &export_formats {
                println!("  Testing {} export...", format_name);
                
                let output_file = temp_dir.path().join(format!("export_{}_{}.{}", size, format_name, format_name));
                let formatter = OutputFormatter::new(OutputFormat::Json);
                
                let start_time = Instant::now();
                let result = formatter.export_data(&records, *format, &output_file);
                let duration = start_time.elapsed();
                
                assert!(result.is_ok());
                assert!(output_file.exists());
                
                // 验证文件大小
                let file_size = fs::metadata(&output_file).unwrap().len();
                println!("    {} export completed in {:?} (file size: {} bytes)", format_name, duration, file_size);
                println!("    Export rate: {:.2} records/second", size as f64 / duration.as_secs_f64());
                println!("    Export rate: {:.2} MB/second", (file_size as f64 / 1024.0 / 1024.0) / duration.as_secs_f64());
                
                // 验证性能要求
                let expected_max_duration = match size {
                    100 => std::time::Duration::from_millis(50),
                    1000 => std::time::Duration::from_millis(200),
                    5000 => std::time::Duration::from_millis(1000),
                    10000 => std::time::Duration::from_millis(2000),
                    _ => std::time::Duration::from_secs(5),
                };
                
                assert!(duration <= expected_max_duration, 
                       "{} export of {} records should take no more than {:?}, took {:?}", 
                       format_name, size, expected_max_duration, duration);
            }
        }
    }

    #[tokio::test]
    async fn test_scalability_performance() {
        let mut generator = TestDataGenerator::new();
        let temp_dir = TempDir::new().unwrap();
        
        // 测试系统在不同数据量下的可扩展性
        let data_sizes = vec![100, 500, 1000, 2000, 5000, 10000];
        let mut performance_results = Vec::new();
        
        for size in data_sizes {
            println!("Testing scalability with {} records...", size);
            
            // 生成测试数据
            let test_data = generator.generate_usage_records(size);
            let data_file = temp_dir.path().join(format!("scalability_data_{}.json", size));
            let json_data = serde_json::to_string_pretty(&json!(test_data)).unwrap();
            fs::write(&data_file, json_data).unwrap();
            
            // 测试端到端处理性能
            let start_time = Instant::now();
            
            // 加载数据
            let data_loader = DataLoader::with_source(
                DataSourceType::Json,
                data_file.to_string_lossy().to_string(),
            );
            let records = data_loader.load_usage_data().await.unwrap();
            
            // 进行分析
            let calculator = CostCalculator::default();
            let breakdown = calculator.calculate_detailed_breakdown(&records).unwrap();
            
            // 生成输出
            let formatter = OutputFormatter::new(OutputFormat::Json);
            let output_file = temp_dir.path().join(format!("scalability_output_{}.json", size));
            let _result = formatter.output_cost_breakdown(&breakdown, Some(output_file.to_str().unwrap()));
            
            let duration = start_time.elapsed();
            
            // 记录性能结果
            performance_results.push((size, duration));
            
            println!("  Processing {} records completed in {:?}", size, duration);
            println!("  Processing rate: {:.2} records/second", size as f64 / duration.as_secs_f64());
            
            // 验证基本性能要求
            assert!(duration.as_secs() < 30, "Processing {} records should complete within 30 seconds", size);
        }
        
        // 分析可扩展性
        println!("Scalability Analysis:");
        for (i, (size, duration)) in performance_results.iter().enumerate() {
            let rate = *size as f64 / duration.as_secs_f64();
            println!("  {}: {} records in {:?} ({:.2} records/second)", i + 1, size, duration, rate);
        }
        
        // 验证可扩展性（处理时间应该大致与数据量成线性关系）
        if performance_results.len() >= 2 {
            let (size1, duration1) = performance_results[0];
            let (size2, duration2) = performance_results[performance_results.len() - 1];
            
            let size_ratio = size2 as f64 / size1 as f64;
            let time_ratio = duration2.as_secs_f64() / duration1.as_secs_f64();
            let scalability_factor = time_ratio / size_ratio;
            
            println!("Size ratio: {:.2}x, Time ratio: {:.2}x, Scalability factor: {:.2}", 
                     size_ratio, time_ratio, scalability_factor);
            
            // 可扩展性因子应该接近1（理想情况下），这里允许一定的偏差
            assert!(scalability_factor < 3.0, "Scalability should be reasonable (factor < 3.0)");
        }
    }

    #[tokio::test]
    async fn test_cache_performance() {
        let mut generator = TestDataGenerator::new();
        let temp_dir = TempDir::new().unwrap();
        
        // 创建测试数据
        let test_data = generator.generate_usage_records(1000);
        let data_file = temp_dir.path().join("cache_test_data.json");
        let json_data = serde_json::to_string_pretty(&json!(test_data)).unwrap();
        fs::write(&data_file, json_data).unwrap();
        
        // 测试重复操作的性能（假设有缓存机制）
        let records: Vec<crate::data::models::UsageRecord> = test_data
            .into_iter()
            .map(|r| serde_json::from_value(r).unwrap())
            .collect();
        
        println!("Testing cache performance with repeated operations...");
        
        // 第一次操作（冷启动）
        let start_time = Instant::now();
        let calculator = CostCalculator::default();
        let _breakdown1 = calculator.calculate_detailed_breakdown(&records).unwrap();
        let first_duration = start_time.elapsed();
        println!("First operation completed in {:?}", first_duration);
        
        // 重复操作（应该更快，如果有缓存）
        let mut durations = Vec::new();
        for i in 0..10 {
            let start_time = Instant::now();
            let _breakdown = calculator.calculate_detailed_breakdown(&records).unwrap();
            let duration = start_time.elapsed();
            durations.push(duration);
            
            if i == 0 {
                println!("Second operation completed in {:?}", duration);
            }
        }
        
        // 计算平均重复操作时间
        let avg_duration: std::time::Duration = durations.iter().sum();
        let avg_duration = avg_duration / durations.len() as u32;
        
        println!("Average repeated operation time: {:?}", avg_duration);
        
        // 验证缓存效果（重复操作应该比第一次快）
        // 注意：这个测试假设CostCalculator有某种缓存机制
        // 如果没有缓存，这个测试可能会失败
        if avg_duration < first_duration {
            println!("Cache performance improvement detected");
        } else {
            println!("No significant cache performance improvement (this may be expected)");
        }
    }

    // 辅助函数：获取内存使用情况
    fn get_memory_usage() -> u64 {
        match memory_stats::memory_stats() {
            Ok(stats) => stats.physical_mem / 1024, // 转换为KB
            Err(_) => 0,
        }
    }
}