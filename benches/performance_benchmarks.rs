use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;
use tempfile::TempDir;
use std::fs;
use std::path::Path;
use chrono::{NaiveDate, Utc};
use serde_json::json;
use claude_code_usage_rs::data::UsageRecord;

/// 基准测试工具模块
mod bench_utils {
    use super::*;
    
    /// 生成大量测试数据
    pub fn generate_large_dataset(size: usize) -> Vec<UsageRecord> {
        let mut records = Vec::with_capacity(size);
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        
        for i in 0..size {
            let date = start_date + chrono::Duration::days((i % 365) as i64);
            let tokens = 1000 + (i % 10000) as u64;
            let cost = tokens as f64 * 0.002;
            
            records.push(UsageRecord {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: date.and_hms_opt(12, 0, 0).unwrap().and_utc(),
                model: format!("claude-3-{}", ["sonnet", "opus", "haiku"][i % 3]),
                input_tokens: (tokens / 2) as u32,
                output_tokens: (tokens / 2) as u32,
                cost,
                session_id: Some(format!("session_{}", i % 50)),
                user_id: Some(format!("user_{}", i % 100)),
                metadata: json!({
                    "language": "rust",
                    "file_count": i % 10 + 1,
                    "complexity": ["low", "medium", "high"][i % 3]
                }),
            });
        }
        
        records
    }
    
    /// 创建测试配置（简化版本）
    pub fn create_test_config() -> () {
        // 简化版本，实际项目中应该使用真实配置
    }
}

/// 数据加载性能基准测试（简化版本）
fn bench_data_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_loading");
    
    // 测试不同数据集大小的生成性能
    for size in [100, 1000, 5000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("generate_data", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let data = bench_utils::generate_large_dataset(size);
                    black_box(data)
                })
            },
        );
    }
    
    // 测试序列化性能
    for size in [100, 1000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::new("serialize_data", size),
            size,
            |b, &size| {
                let data = bench_utils::generate_large_dataset(size);
                
                b.iter(|| {
                    black_box(serde_json::to_string(&data))
                })
            },
        );
    }
    
    group.finish();
}

/// 数据处理性能基准测试
fn bench_data_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_processing");
    
    // 测试数据过滤性能
    for size in [1000, 5000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("filter_by_date", size),
            size,
            |b, &size| {
                let data = bench_utils::generate_large_dataset(size);
                let start_date = NaiveDate::from_ymd_opt(2024, 6, 1).unwrap();
                let end_date = NaiveDate::from_ymd_opt(2024, 6, 30).unwrap();
                
                b.iter(|| {
                    black_box(
                        data.iter()
                            .filter(|record| {
                                let record_date = record.timestamp.date_naive();
                                record_date >= start_date && record_date <= end_date
                            })
                            .count()
                    )
                })
            },
        );
    }
    
    // 测试数据聚合性能
    for size in [1000, 5000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("aggregate_by_model", size),
            size,
            |b, &size| {
                let data = bench_utils::generate_large_dataset(size);
                
                b.iter(|| {
                    let mut model_stats = std::collections::HashMap::new();
                    for record in black_box(&data) {
                        let entry = model_stats.entry(&record.model).or_insert((0u64, 0.0));
                        entry.0 += (record.input_tokens + record.output_tokens) as u64;
                        entry.1 += record.cost;
                    }
                    black_box(model_stats)
                })
            },
        );
    }
    
    // 测试数据排序性能
    for size in [1000, 5000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("sort_by_cost", size),
            size,
            |b, &size| {
                let mut data = bench_utils::generate_large_dataset(size);
                
                b.iter(|| {
                    data.sort_by(|a, b| b.cost.partial_cmp(&a.cost).unwrap());
                    black_box(&data)
                })
            },
        );
    }
    
    group.finish();
}

/// 内存使用基准测试
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    // 测试大数据集的内存占用
    group.bench_function("large_dataset_memory", |b| {
        b.iter(|| {
            let data = bench_utils::generate_large_dataset(50000);
            black_box(data);
        })
    });
    
    // 测试数据处理过程中的内存使用
    group.bench_function("processing_memory", |b| {
        let data = bench_utils::generate_large_dataset(10000);
        
        b.iter(|| {
            let filtered: Vec<_> = black_box(&data)
                .iter()
                .filter(|record| record.cost_usd > 5.0)
                .cloned()
                .collect();
            let aggregated: std::collections::HashMap<_, _> = filtered
                .into_iter()
                .map(|record| (record.model_name, record.cost_usd))
                .fold(std::collections::HashMap::new(), |mut acc, (model, cost)| {
                    *acc.entry(model).or_insert(0.0) += cost;
                    acc
                });
            black_box(aggregated);
        })
    });
    
    group.finish();
}

/// 并发处理性能基准测试
fn bench_concurrent_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_processing");
    
    // 测试并发数据处理
    for thread_count in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_filter", thread_count),
            thread_count,
            |b, &thread_count| {
                let data = bench_utils::generate_large_dataset(10000);
                let chunk_size = (data.len() + thread_count - 1) / thread_count;
                
                b.iter(|| {
                    use std::sync::Arc;
                    use tokio::runtime::Runtime;
                    
                    let rt = Runtime::new().unwrap();
                    let data_arc = Arc::new(data.clone());
                    
                    let handles: Vec<_> = (0..thread_count)
                        .map(|i| {
                            let data_clone = data_arc.clone();
                            let start = i * chunk_size;
                            let end = std::cmp::min(start + chunk_size, data_clone.len());
                            
                            rt.spawn(async move {
                                data_clone[start..end]
                                    .iter()
                                    .filter(|record| record.tokens_used > 5000)
                                    .count()
                            })
                        })
                        .collect();
                    
                    let result: usize = rt.block_on(async {
                        futures::future::try_join_all(handles)
                            .await
                            .unwrap()
                            .into_iter()
                            .sum()
                    });
                    
                    black_box(result)
                })
            },
        );
    }
    
    group.finish();
}

/// CLI命令处理性能基准测试
fn bench_cli_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("cli_processing");
    
    // 测试CLI参数解析性能
    group.bench_function("argument_parsing", |b| {
        let args = vec![
            "ccusage-rs",
            "--data-dir", "/tmp/test",
            "--format", "json",
            "--verbose",
            "analyze",
            "--start-date", "2024-01-01",
            "--end-date", "2024-12-31"
        ];
        
        b.iter(|| {
            black_box(Cli::try_parse_from(&args))
        })
    });
    
    // 测试配置加载性能
    group.bench_function("config_loading", |b| {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        
        let config_content = r#"
            data_dir = "/tmp/test_data"
            cache_dir = "/tmp/test_cache"
            max_records = 10000
            default_output_format = "json"
            date_format = "%Y-%m-%d"
            currency = "USD"
            timezone = "UTC"
            enable_monitoring = true
            log_level = "info"
        "#;
        
        fs::write(&config_path, config_content).unwrap();
        
        b.iter(|| {
            black_box(Config::load_from_file(&config_path))
        })
    });
    
    group.finish();
}

/// 序列化/反序列化性能基准测试
fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");
    
    // 测试JSON序列化性能
    for size in [100, 1000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::new("json_serialize", size),
            size,
            |b, &size| {
                let data = bench_utils::generate_large_dataset(size);
                
                b.iter(|| {
                    black_box(serde_json::to_string(&data))
                })
            },
        );
    }
    
    // 测试JSON反序列化性能
    for size in [100, 1000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::new("json_deserialize", size),
            size,
            |b, &size| {
                let data = bench_utils::generate_large_dataset(size);
                let json_data = serde_json::to_string(&data).unwrap();
                
                b.iter(|| {
                    black_box(serde_json::from_str::<Vec<UsageRecord>>(&json_data))
                })
            },
        );
    }
    
    group.finish();
}

/// 错误处理性能基准测试
fn bench_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");
    
    // 测试错误创建和传播性能
    group.bench_function("error_creation", |b| {
        b.iter(|| {
            let error = CcusageError::DataFormat("Invalid JSON format".to_string());
            let result: Result<(), CcusageError> = Err(error);
            black_box(result)
        })
    });
    
    // 测试错误转换性能
    group.bench_function("error_conversion", |b| {
        b.iter(|| {
            let io_error = std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found"
            );
            let converted: Result<(), CcusageError> = Err(io_error.into());
            black_box(converted)
        })
    });
    
    group.finish();
}

/// 综合基准测试组
criterion_group!(
    benches,
    bench_data_loading,
    bench_data_processing,
    bench_memory_usage,
    bench_concurrent_processing,
    bench_cli_processing,
    bench_serialization,
    bench_error_handling
);

criterion_main!(benches);