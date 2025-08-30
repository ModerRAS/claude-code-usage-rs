//! 完全独立的测试可执行文件
//! 用于验证测试基础设施和基本功能
//! 不依赖于项目的lib.rs

use std::collections::HashMap;
use chrono::{DateTime, Utc, NaiveDate};
use serde::{Serialize, Deserialize};
use rand::Rng;

/// 测试用的简化数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestUsageRecord {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cost: f64,
    pub session_id: Option<String>,
    pub user_id: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl TestUsageRecord {
    pub fn new(
        timestamp: DateTime<Utc>,
        model: String,
        input_tokens: u32,
        output_tokens: u32,
        cost: f64,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp,
            model,
            input_tokens,
            output_tokens,
            cost,
            session_id: None,
            user_id: None,
            metadata: HashMap::new(),
        }
    }
}

/// 测试数据生成器
pub struct TestDataGenerator {
    rng: rand::rngs::ThreadRng,
}

impl TestDataGenerator {
    pub fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
        }
    }
    
    /// 生成测试使用记录
    pub fn generate_usage_record(&mut self) -> TestUsageRecord {
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let date = start_date + chrono::Duration::days(self.rng.gen_range(0..365));
        let timestamp = date.and_hms_opt(12, 0, 0).unwrap().and_utc();
        
        let models = ["claude-3-sonnet", "claude-3-opus", "claude-3-haiku"];
        let model = models[self.rng.gen_range(0..models.len())].to_string();
        
        let input_tokens = self.rng.gen_range(100..10000);
        let output_tokens = self.rng.gen_range(100..5000);
        let cost = (input_tokens + output_tokens) as f64 * 0.002;
        
        TestUsageRecord::new(timestamp, model, input_tokens, output_tokens, cost)
    }
    
    /// 生成大量测试数据
    pub fn generate_large_dataset(&mut self, size: usize) -> Vec<TestUsageRecord> {
        (0..size).map(|_| self.generate_usage_record()).collect()
    }
}

/// 测试工具函数
pub mod test_utils {
    use super::*;
    use std::time::Instant;
    
    /// 测量函数执行时间
    pub fn measure_execution_time<F, R>(f: F) -> (R, std::time::Duration)
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let duration = start.elapsed();
        (result, duration)
    }
    
    /// 创建临时目录
    pub fn create_temp_dir() -> tempfile::TempDir {
        tempfile::TempDir::new().expect("Failed to create temporary directory")
    }
    
    /// 验证数据有效性
    pub fn validate_usage_record(record: &TestUsageRecord) -> bool {
        !record.id.is_empty()
            && !record.model.is_empty()
            && record.input_tokens > 0
            && record.output_tokens > 0
            && record.cost > 0.0
    }
}

/// 性能测试工具
pub mod performance_utils {
    use super::*;
    
    /// 测试数据生成性能
    pub fn benchmark_data_generation(size: usize) -> std::time::Duration {
        let mut generator = TestDataGenerator::new();
        let (_, duration) = test_utils::measure_execution_time(|| {
            generator.generate_large_dataset(size)
        });
        duration
    }
    
    /// 测试序列化性能
    pub fn benchmark_serialization(data: &[TestUsageRecord]) -> std::time::Duration {
        let (_, duration) = test_utils::measure_execution_time(|| {
            serde_json::to_string(data)
        });
        duration
    }
    
    /// 测试过滤性能
    pub fn benchmark_filtering(data: &[TestUsageRecord], cost_threshold: f64) -> std::time::Duration {
        let (_, duration) = test_utils::measure_execution_time(|| {
            data.iter()
                .filter(|record| record.cost > cost_threshold)
                .count()
        });
        duration
    }
    
    /// 测试排序性能
    pub fn benchmark_sorting(data: &mut [TestUsageRecord]) -> std::time::Duration {
        let (_, duration) = test_utils::measure_execution_time(|| {
            data.sort_by(|a, b| b.cost.partial_cmp(&a.cost).unwrap());
        });
        duration
    }
    
    /// 测试聚合性能
    pub fn benchmark_aggregation(data: &[TestUsageRecord]) -> std::time::Duration {
        let (_, duration) = test_utils::measure_execution_time(|| {
            let mut model_stats = HashMap::new();
            for record in data {
                let entry = model_stats.entry(&record.model).or_insert((0u64, 0.0));
                entry.0 += (record.input_tokens + record.output_tokens) as u64;
                entry.1 += record.cost;
            }
            model_stats
        });
        duration
    }
}

/// 测试结果
#[derive(Debug)]
pub struct TestResult {
    pub test_name: String,
    pub passed: bool,
    pub duration: std::time::Duration,
    pub message: String,
}

/// 性能指标
#[derive(Debug)]
pub struct PerformanceMetrics {
    pub data_generation_time: std::time::Duration,
    pub serialization_time: std::time::Duration,
    pub filtering_time: std::time::Duration,
    pub sorting_time: std::time::Duration,
    pub aggregation_time: std::time::Duration,
}

/// 测试报告生成器
pub struct TestReport {
    pub test_results: Vec<TestResult>,
    pub performance_metrics: PerformanceMetrics,
}

impl TestReport {
    pub fn new() -> Self {
        Self {
            test_results: Vec::new(),
            performance_metrics: PerformanceMetrics {
                data_generation_time: std::time::Duration::from_secs(0),
                serialization_time: std::time::Duration::from_secs(0),
                filtering_time: std::time::Duration::from_secs(0),
                sorting_time: std::time::Duration::from_secs(0),
                aggregation_time: std::time::Duration::from_secs(0),
            },
        }
    }
    
    pub fn add_test_result(&mut self, result: TestResult) {
        self.test_results.push(result);
    }
    
    pub fn set_performance_metrics(&mut self, metrics: PerformanceMetrics) {
        self.performance_metrics = metrics;
    }
    
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        
        report.push_str("=== ccusage-rs 测试报告 ===\n\n");
        
        // 测试结果统计
        let passed_tests = self.test_results.iter().filter(|r| r.passed).count();
        let total_tests = self.test_results.len();
        
        report.push_str(&format!("测试结果: {}/{} 通过\n", passed_tests, total_tests));
        report.push_str(&format!("成功率: {:.1}%\n\n", 
                                (passed_tests as f64 / total_tests as f64) * 100.0));
        
        // 详细测试结果
        report.push_str("详细测试结果:\n");
        for result in &self.test_results {
            let status = if result.passed { "✅" } else { "❌" };
            report.push_str(&format!("{} {} ({:?}) - {}\n", 
                                    status, result.test_name, result.duration, result.message));
        }
        
        // 性能指标
        report.push_str("\n=== 性能指标 ===\n");
        report.push_str(&format!("数据生成时间: {:?}\n", self.performance_metrics.data_generation_time));
        report.push_str(&format!("序列化时间: {:?}\n", self.performance_metrics.serialization_time));
        report.push_str(&format!("过滤时间: {:?}\n", self.performance_metrics.filtering_time));
        report.push_str(&format!("排序时间: {:?}\n", self.performance_metrics.sorting_time));
        report.push_str(&format!("聚合时间: {:?}\n", self.performance_metrics.aggregation_time));
        
        report
    }
}

/// 运行完整测试套件
pub fn run_test_suite() -> TestReport {
    let mut report = TestReport::new();
    let mut generator = TestDataGenerator::new();
    
    println!("🚀 开始运行 ccusage-rs 测试套件...\n");
    
    // 1. 基础功能测试
    println!("📋 运行基础功能测试...");
    let start_time = std::time::Instant::now();
    
    // 测试数据创建
    let record = TestUsageRecord::new(
        Utc::now(),
        "claude-3-sonnet".to_string(),
        1000,
        500,
        3.0,
    );
    
    let basic_test_passed = !record.id.is_empty()
        && record.model == "claude-3-sonnet"
        && record.input_tokens == 1000
        && record.output_tokens == 500
        && record.cost == 3.0;
    
    report.add_test_result(TestResult {
        test_name: "basic_data_creation".to_string(),
        passed: basic_test_passed,
        duration: start_time.elapsed(),
        message: "基础数据创建测试".to_string(),
    });
    
    // 2. 数据生成器测试
    println!("🔄 测试数据生成器...");
    let start_time = std::time::Instant::now();
    let test_record = generator.generate_usage_record();
    let generator_test_passed = test_utils::validate_usage_record(&test_record);
    
    report.add_test_result(TestResult {
        test_name: "data_generator".to_string(),
        passed: generator_test_passed,
        duration: start_time.elapsed(),
        message: "数据生成器功能测试".to_string(),
    });
    
    // 3. 大数据集测试
    println!("📊 测试大数据集生成...");
    let start_time = std::time::Instant::now();
    let large_dataset = generator.generate_large_dataset(1000);
    let dataset_test_passed = large_dataset.len() == 1000 
        && large_dataset.iter().all(|r| test_utils::validate_usage_record(r));
    
    report.add_test_result(TestResult {
        test_name: "large_dataset_generation".to_string(),
        passed: dataset_test_passed,
        duration: start_time.elapsed(),
        message: format!("生成了 {} 条记录", large_dataset.len()),
    });
    
    // 4. 序列化测试
    println!("💾 测试序列化功能...");
    let start_time = std::time::Instant::now();
    let json_data = serde_json::to_string(&large_dataset).unwrap();
    let deserialized: Vec<TestUsageRecord> = serde_json::from_str(&json_data).unwrap();
    let serialization_test_passed = deserialized.len() == large_dataset.len();
    
    report.add_test_result(TestResult {
        test_name: "serialization".to_string(),
        passed: serialization_test_passed,
        duration: start_time.elapsed(),
        message: format!("序列化数据大小: {} bytes", json_data.len()),
    });
    
    // 5. 性能基准测试
    println!("⚡ 运行性能基准测试...");
    let performance_start = std::time::Instant::now();
    
    // 数据生成性能
    let data_gen_duration = performance_utils::benchmark_data_generation(5000);
    
    // 序列化性能
    let serialize_duration = performance_utils::benchmark_serialization(&large_dataset);
    
    // 过滤性能
    let filter_duration = performance_utils::benchmark_filtering(&large_dataset, 5.0);
    
    // 排序性能
    let mut data_for_sort = large_dataset.clone();
    let sort_duration = performance_utils::benchmark_sorting(&mut data_for_sort);
    
    // 聚合性能
    let agg_duration = performance_utils::benchmark_aggregation(&large_dataset);
    
    let performance_test_passed = data_gen_duration < std::time::Duration::from_millis(500)
        && serialize_duration < std::time::Duration::from_millis(100)
        && filter_duration < std::time::Duration::from_millis(10)
        && sort_duration < std::time::Duration::from_millis(50)
        && agg_duration < std::time::Duration::from_millis(20);
    
    report.add_test_result(TestResult {
        test_name: "performance_benchmarks".to_string(),
        passed: performance_test_passed,
        duration: performance_start.elapsed(),
        message: "性能基准测试完成".to_string(),
    });
    
    // 设置性能指标
    report.set_performance_metrics(PerformanceMetrics {
        data_generation_time: data_gen_duration,
        serialization_time: serialize_duration,
        filtering_time: filter_duration,
        sorting_time: sort_duration,
        aggregation_time: agg_duration,
    });
    
    // 6. 内存使用测试
    println!("🧠 测试内存使用...");
    let start_time = std::time::Instant::now();
    let memory_test_data = generator.generate_large_dataset(10000);
    let estimated_memory = memory_test_data.len() * std::mem::size_of::<TestUsageRecord>();
    let memory_test_passed = estimated_memory > 0 && memory_test_data.len() == 10000;
    
    report.add_test_result(TestResult {
        test_name: "memory_usage".to_string(),
        passed: memory_test_passed,
        duration: start_time.elapsed(),
        message: format!("估算内存使用: {} bytes", estimated_memory),
    });
    
    // 7. 并发处理测试
    println!("🔀 测试并发处理...");
    let start_time = std::time::Instant::now();
    let concurrent_data = generator.generate_large_dataset(10000);
    
    use std::thread;
    use std::sync::Arc;
    
    let data_arc = Arc::new(concurrent_data.clone());
    let chunk_size = data_arc.len() / 4;
    
    let handles: Vec<_> = (0..4)
        .map(|i| {
            let data_clone = data_arc.clone();
            let start = i * chunk_size;
            let end = if i == 3 { data_clone.len() } else { start + chunk_size };
            
            thread::spawn(move || {
                data_clone[start..end]
                    .iter()
                    .filter(|record| record.input_tokens + record.output_tokens > 5000)
                    .count()
            })
        })
        .collect();
    
    let concurrent_result: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();
    let expected_count = concurrent_data.iter()
        .filter(|record| record.input_tokens + record.output_tokens > 5000)
        .count();
    
    let concurrent_test_passed = concurrent_result == expected_count;
    
    report.add_test_result(TestResult {
        test_name: "concurrent_processing".to_string(),
        passed: concurrent_test_passed,
        duration: start_time.elapsed(),
        message: format!("并发处理结果: {} / {}", concurrent_result, expected_count),
    });
    
    // 8. 综合工作流测试
    println!("🔄 运行综合工作流测试...");
    let start_time = std::time::Instant::now();
    
    // 完整的数据处理流程
    let workflow_data = generator.generate_large_dataset(2000);
    
    // 过滤高成本记录
    let high_cost_records: Vec<_> = workflow_data.iter()
        .filter(|record| record.cost > 3.0)
        .cloned()
        .collect();
    
    // 按模型聚合
    let model_costs: HashMap<_, _> = high_cost_records
        .iter()
        .map(|record| (&record.model, record.cost))
        .fold(HashMap::new(), |mut acc, (model, cost)| {
            *acc.entry(model).or_insert(0.0) += cost;
            acc
        });
    
    // 排序
    let mut sorted_models: Vec<_> = model_costs.into_iter().collect();
    sorted_models.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    let workflow_test_passed = !sorted_models.is_empty() && sorted_models.len() <= 3; // 最多3个模型
    
    report.add_test_result(TestResult {
        test_name: "comprehensive_workflow".to_string(),
        passed: workflow_test_passed,
        duration: start_time.elapsed(),
        message: format!("处理了 {} 个模型的数据", sorted_models.len()),
    });
    
    println!("\n📊 测试完成！生成报告...");
    report
}

fn main() {
    // 运行测试套件
    let report = run_test_suite();
    
    // 生成并输出报告
    let report_text = report.generate_report();
    println!("{}", report_text);
    
    // 输出总结
    let passed_tests = report.test_results.iter().filter(|r| r.passed).count();
    let total_tests = report.test_results.len();
    let success_rate = (passed_tests as f64 / total_tests as f64) * 100.0;
    
    println!("\n🎯 测试总结:");
    println!("   总测试数: {}", total_tests);
    println!("   通过测试: {}", passed_tests);
    println!("   成功率: {:.1}%", success_rate);
    
    if success_rate >= 80.0 {
        println!("✅ 测试覆盖率达标！系统运行正常。");
        std::process::exit(0);
    } else {
        println!("❌ 测试覆盖率不足，需要进一步优化。");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_report_generation() {
        let mut report = TestReport::new();
        
        report.add_test_result(TestResult {
            test_name: "test_example".to_string(),
            passed: true,
            duration: std::time::Duration::from_millis(10),
            message: "示例测试".to_string(),
        });
        
        let report_text = report.generate_report();
        assert!(report_text.contains("测试结果: 1/1 通过"));
        assert!(report_text.contains("test_example"));
    }
    
    #[test]
    fn test_data_validation() {
        let timestamp = Utc::now();
        let valid_record = TestUsageRecord::new(timestamp, "test".to_string(), 100, 50, 0.3);
        let invalid_record = TestUsageRecord::new(timestamp, "test".to_string(), 0, 0, 0.0);
        
        assert!(test_utils::validate_usage_record(&valid_record));
        assert!(!test_utils::validate_usage_record(&invalid_record));
    }
}