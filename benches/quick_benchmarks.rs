use criterion::{criterion_group, criterion_main, Criterion};

// 导入主要基准测试函数
mod performance_benchmarks;

/// 快速基准测试 - 用于CI/CD和开发过程中的快速验证
fn quick_benchmarks(c: &mut Criterion) {
    // 只运行关键的基准测试，用于快速验证性能回归
    let mut group = c.benchmark_group("quick_checks");
    
    // 数据加载基准测试（小数据集）
    group.bench_function("small_data_load", |b| {
        use performance_benchmarks::bench_utils;
        let data = bench_utils::generate_large_dataset(100);
        
        b.iter(|| {
            use claude_code_usage_rs::data::DataLoader;
            let loader = DataLoader::new();
            criterion::black_box(loader.process_records(&data))
        })
    });
    
    // 数据处理基准测试（小数据集）
    group.bench_function("small_data_process", |b| {
        use performance_benchmarks::bench_utils;
        let data = bench_utils::generate_large_dataset(100);
        
        b.iter(|| {
            criterion::black_box(
                data.iter()
                    .filter(|record| record.cost_usd > 1.0)
                    .count()
            )
        })
    });
    
    // CLI解析基准测试
    group.bench_function("cli_parsing", |b| {
        use claude_code_usage_rs::cli::Cli;
        
        b.iter(|| {
            criterion::black_box(
                Cli::try_parse_from(&[
                    "ccusage-rs",
                    "--format", "json",
                    "analyze",
                    "--start-date", "2024-01-01"
                ])
            )
        })
    });
    
    group.finish();
}

/// 内存使用基准测试 - 专门用于检测内存泄漏
fn memory_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_checks");
    
    // 测试大量数据创建和销毁
    group.bench_function("large_data_creation_cleanup", |b| {
        use performance_benchmarks::bench_utils;
        
        b.iter(|| {
            let data = bench_utils::generate_large_dataset(1000);
            let _processed = data.into_iter()
                .filter(|record| record.tokens_used > 1000)
                .collect::<Vec<_>>();
            // 数据应该在这里被自动清理
        })
    });
    
    // 测试文件操作内存使用
    group.bench_function("file_operations_memory", |b| {
        use tempfile::TempDir;
        use std::fs;
        
        b.iter(|| {
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("test.json");
            
            // 写入数据
            let data = bench_utils::generate_large_dataset(500);
            let json = serde_json::to_string(&data).unwrap();
            fs::write(&file_path, json).unwrap();
            
            // 读取数据
            let content = fs::read_to_string(&file_path).unwrap();
            let _parsed: Vec<claude_code_usage_rs::data::UsageRecord> = 
                serde_json::from_str(&content).unwrap();
            
            // 临时目录和文件应该在这里被自动清理
        })
    });
    
    group.finish();
}

criterion_group!(
    quick_benches,
    quick_benchmarks,
    memory_benchmarks
);

criterion_main!(quick_benches);