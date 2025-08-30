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
}

/// 数据生成性能基准测试
fn bench_data_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_generation");
    
    // 测试不同数据集大小的生成性能
    for size in [100, 1000, 5000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("generate_dataset", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let data = bench_utils::generate_large_dataset(size);
                    black_box(data)
                })
            },
        );
    }
    
    group.finish();
}

/// 序列化性能基准测试
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

/// 数据处理性能基准测试
fn bench_data_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_processing");
    
    // 测试数据过滤性能
    for size in [1000, 5000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::new("filter_by_cost", size),
            size,
            |b, &size| {
                let data = bench_utils::generate_large_dataset(size);
                
                b.iter(|| {
                    black_box(
                        data.iter()
                            .filter(|record| record.cost > 5.0)
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
                .filter(|record| record.cost > 5.0)
                .cloned()
                .collect();
            let aggregated: std::collections::HashMap<_, _> = filtered
                .into_iter()
                .map(|record| (record.model, record.cost))
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
                    use std::thread;
                    
                    let data_arc = Arc::new(data.clone());
                    let mut handles = vec![];
                    
                    for i in 0..thread_count {
                        let data_clone = data_arc.clone();
                        let start = i * chunk_size;
                        let end = std::cmp::min(start + chunk_size, data_clone.len());
                        
                        let handle = thread::spawn(move || {
                            data_clone[start..end]
                                .iter()
                                .filter(|record| record.input_tokens + record.output_tokens > 5000)
                                .count()
                        });
                        handles.push(handle);
                    }
                    
                    let result: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();
                    black_box(result)
                })
            },
        );
    }
    
    group.finish();
}

/// 基准测试组
criterion_group!(
    benches,
    bench_data_generation,
    bench_serialization,
    bench_data_processing,
    bench_memory_usage,
    bench_concurrent_processing
);

criterion_main!(benches);