//! 完全独立的测试套件
//! 测试基础设施验证和基本功能测试

use std::collections::HashMap;
use chrono::{DateTime, Utc, NaiveDate};
use serde::{Serialize, Deserialize};

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

/// 内存使用测试
pub mod memory_utils {
    use super::*;
    
    /// 测试大数据集内存使用
    pub fn test_large_dataset_memory(size: usize) -> Result<(), String> {
        let mut generator = TestDataGenerator::new();
        let data = generator.generate_large_dataset(size);
        
        // 估算内存使用
        let estimated_size = data.len() * std::mem::size_of::<TestUsageRecord>();
        println!("估算内存使用: {} bytes", estimated_size);
        
        // 验证数据完整性
        for record in &data {
            if !test_utils::validate_usage_record(record) {
                return Err("数据验证失败".to_string());
            }
        }
        
        Ok(())
    }
}

/// 错误处理测试
pub mod error_utils {
    use super::*;
    
    /// 测试无效数据处理
    pub fn test_invalid_data_handling() -> Result<(), String> {
        // 测试无效数据创建
        let timestamp = Utc::now();
        let record = TestUsageRecord::new(timestamp, "test".to_string(), 0, 0, 0.0);
        
        if test_utils::validate_usage_record(&record) {
            return Err("无效数据验证失败".to_string());
        }
        
        // 测试JSON序列化错误处理
        let invalid_json = "{ invalid json }";
        let result: Result<Vec<TestUsageRecord>, _> = serde_json::from_str(invalid_json);
        
        if result.is_ok() {
            return Err("JSON解析应该失败".to_string());
        }
        
        Ok(())
    }
}

/// 并发处理测试
pub mod concurrency_utils {
    use super::*;
    use std::thread;
    use std::sync::Arc;
    
    /// 测试并发数据处理
    pub fn test_concurrent_processing(data: &[TestUsageRecord], thread_count: usize) -> usize {
        let data_arc = Arc::new(data.to_vec());
        let chunk_size = (data.len() + thread_count - 1) / thread_count;
        
        let handles: Vec<_> = (0..thread_count)
            .map(|i| {
                let data_clone = data_arc.clone();
                let start = i * chunk_size;
                let end = std::cmp::min(start + chunk_size, data_clone.len());
                
                thread::spawn(move || {
                    data_clone[start..end]
                        .iter()
                        .filter(|record| record.input_tokens + record.output_tokens > 5000)
                        .count()
                })
            })
            .collect();
        
        handles.into_iter().map(|h| h.join().unwrap()).sum()
    }
}

/// 测试报告生成器
pub struct TestReport {
    pub test_results: Vec<TestResult>,
    pub performance_metrics: PerformanceMetrics,
}

#[derive(Debug)]
pub struct TestResult {
    pub test_name: String,
    pub passed: bool,
    pub duration: std::time::Duration,
    pub message: String,
}

#[derive(Debug)]
pub struct PerformanceMetrics {
    pub data_generation_time: std::time::Duration,
    pub serialization_time: std::time::Duration,
    pub filtering_time: std::time::Duration,
    pub sorting_time: std::time::Duration,
    pub aggregation_time: std::time::Duration,
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
        
        report.push_str("=== 测试报告 ===\n\n");
        
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_usage_record() {
        let timestamp = Utc::now();
        let record = TestUsageRecord::new(
            timestamp,
            "claude-3-sonnet".to_string(),
            1000,
            500,
            3.0,
        );
        
        assert!(!record.id.is_empty());
        assert_eq!(record.model, "claude-3-sonnet");
        assert_eq!(record.input_tokens, 1000);
        assert_eq!(record.output_tokens, 500);
        assert_eq!(record.cost, 3.0);
    }
    
    #[test]
    fn test_data_generator() {
        let mut generator = TestDataGenerator::new();
        let record = generator.generate_usage_record();
        
        assert!(test_utils::validate_usage_record(&record));
    }
    
    #[test]
    fn test_generate_large_dataset() {
        let mut generator = TestDataGenerator::new();
        let dataset = generator.generate_large_dataset(100);
        
        assert_eq!(dataset.len(), 100);
        for record in dataset {
            assert!(test_utils::validate_usage_record(&record));
        }
    }
    
    #[test]
    fn test_serialization() {
        let mut generator = TestDataGenerator::new();
        let data = generator.generate_large_dataset(10);
        
        let json = serde_json::to_string(&data).unwrap();
        let deserialized: Vec<TestUsageRecord> = serde_json::from_str(&json).unwrap();
        
        assert_eq!(data.len(), deserialized.len());
    }
    
    #[test]
    fn test_measure_execution_time() {
        let (result, duration) = test_utils::measure_execution_time(|| {
            std::thread::sleep(std::time::Duration::from_millis(10));
            42
        });
        
        assert_eq!(result, 42);
        assert!(duration >= std::time::Duration::from_millis(10));
    }
    
    #[test]
    fn test_performance_benchmarks() {
        let mut generator = TestDataGenerator::new();
        let data = generator.generate_large_dataset(1000);
        
        // 测试序列化性能
        let serialize_duration = performance_utils::benchmark_serialization(&data);
        println!("序列化 1000 条记录耗时: {:?}", serialize_duration);
        
        // 测试过滤性能
        let filter_duration = performance_utils::benchmark_filtering(&data, 5.0);
        println!("过滤 1000 条记录耗时: {:?}", filter_duration);
        
        // 测试排序性能
        let mut data_for_sort = data.clone();
        let sort_duration = performance_utils::benchmark_sorting(&mut data_for_sort);
        println!("排序 1000 条记录耗时: {:?}", sort_duration);
        
        // 测试聚合性能
        let agg_duration = performance_utils::benchmark_aggregation(&data);
        println!("聚合 1000 条记录耗时: {:?}", agg_duration);
        
        // 验证性能在合理范围内
        assert!(serialize_duration < std::time::Duration::from_millis(100));
        assert!(filter_duration < std::time::Duration::from_millis(10));
        assert!(sort_duration < std::time::Duration::from_millis(50));
        assert!(agg_duration < std::time::Duration::from_millis(20));
    }
    
    #[test]
    fn test_memory_usage() {
        // 测试大数据集内存使用
        assert!(memory_utils::test_large_dataset_memory(10000).is_ok());
        
        // 测试处理过程中的内存使用
        let mut generator = TestDataGenerator::new();
        let data = generator.generate_large_dataset(5000);
        
        let filtered: Vec<_> = data.iter()
            .filter(|record| record.cost > 5.0)
            .cloned()
            .collect();
        
        let aggregated: HashMap<_, _> = filtered
            .into_iter()
            .map(|record| (record.model, record.cost))
            .fold(HashMap::new(), |mut acc, (model, cost)| {
                *acc.entry(model).or_insert(0.0) += cost;
                acc
            });
        
        assert!(!aggregated.is_empty());
    }
    
    #[test]
    fn test_concurrent_processing() {
        let mut generator = TestDataGenerator::new();
        let data = generator.generate_large_dataset(10000);
        
        // 测试并发处理
        let concurrent_result = concurrency_utils::test_concurrent_processing(&data, 4);
        
        // 验证结果
        let expected_count = data.iter()
            .filter(|record| record.input_tokens + record.output_tokens > 5000)
            .count();
        
        assert_eq!(concurrent_result, expected_count);
    }
    
    #[test]
    fn test_error_handling() {
        assert!(error_utils::test_invalid_data_handling().is_ok());
    }
    
    #[test]
    fn test_scalability() {
        let mut generator = TestDataGenerator::new();
        
        // 测试不同数据规模的性能
        let sizes = [100, 1000, 5000, 10000];
        
        for &size in &sizes {
            let data = generator.generate_large_dataset(size);
            
            let serialize_duration = performance_utils::benchmark_serialization(&data);
            let filter_duration = performance_utils::benchmark_filtering(&data, 5.0);
            let sort_duration = performance_utils::benchmark_sorting(&mut data.clone());
            
            println!("数据大小: {}, 序列化: {:?}, 过滤: {:?}, 排序: {:?}", 
                     size, serialize_duration, filter_duration, sort_duration);
            
            // 验证性能随着数据增长的合理性
            assert!(serialize_duration < std::time::Duration::from_millis(size as u64 / 10));
            assert!(filter_duration < std::time::Duration::from_millis(size as u64 / 100));
        }
    }
    
    #[test]
    fn test_report_generation() {
        let mut report = TestReport::new();
        
        // 添加测试结果
        report.add_test_result(TestResult {
            test_name: "test_basic_functionality".to_string(),
            passed: true,
            duration: std::time::Duration::from_millis(10),
            message: "基础功能测试通过".to_string(),
        });
        
        report.add_test_result(TestResult {
            test_name: "test_performance".to_string(),
            passed: true,
            duration: std::time::Duration::from_millis(100),
            message: "性能测试通过".to_string(),
        });
        
        // 设置性能指标
        report.set_performance_metrics(PerformanceMetrics {
            data_generation_time: std::time::Duration::from_millis(50),
            serialization_time: std::time::Duration::from_millis(20),
            filtering_time: std::time::Duration::from_millis(5),
            sorting_time: std::time::Duration::from_millis(30),
            aggregation_time: std::time::Duration::from_millis(15),
        });
        
        let report_text = report.generate_report();
        assert!(report_text.contains("测试结果: 2/2 通过"));
        assert!(report_text.contains("性能指标"));
    }
    
    #[test]
    fn test_comprehensive_workflow() {
        let mut generator = TestDataGenerator::new();
        let mut report = TestReport::new();
        
        // 1. 数据生成测试
        let start_time = std::time::Instant::now();
        let data = generator.generate_large_dataset(5000);
        let data_gen_duration = start_time.elapsed();
        
        report.add_test_result(TestResult {
            test_name: "data_generation".to_string(),
            passed: data.len() == 5000,
            duration: data_gen_duration,
            message: format!("生成了 {} 条记录", data.len()),
        });
        
        // 2. 数据验证测试
        let validation_start = std::time::Instant::now();
        let valid_records = data.iter()
            .filter(|record| test_utils::validate_usage_record(record))
            .count();
        let validation_duration = validation_start.elapsed();
        
        report.add_test_result(TestResult {
            test_name: "data_validation".to_string(),
            passed: valid_records == 5000,
            duration: validation_duration,
            message: format!("验证了 {} 条记录", valid_records),
        });
        
        // 3. 序列化测试
        let serialize_start = std::time::Instant::now();
        let json_data = serde_json::to_string(&data).unwrap();
        let serialize_duration = serialize_start.elapsed();
        
        report.add_test_result(TestResult {
            test_name: "serialization".to_string(),
            passed: !json_data.is_empty(),
            duration: serialize_duration,
            message: format!("序列化数据大小: {} bytes", json_data.len()),
        });
        
        // 4. 处理测试
        let process_start = std::time::Instant::now();
        let filtered: Vec<_> = data.iter()
            .filter(|record| record.cost > 5.0)
            .cloned()
            .collect();
        let aggregated: HashMap<_, _> = filtered
            .into_iter()
            .map(|record| (record.model, record.cost))
            .fold(HashMap::new(), |mut acc, (model, cost)| {
                *acc.entry(model).or_insert(0.0) += cost;
                acc
            });
        let process_duration = process_start.elapsed();
        
        report.add_test_result(TestResult {
            test_name: "data_processing".to_string(),
            passed: !aggregated.is_empty(),
            duration: process_duration,
            message: format!("聚合了 {} 个模型", aggregated.len()),
        });
        
        // 5. 并发处理测试
        let concurrent_start = std::time::Instant::now();
        let concurrent_result = concurrency_utils::test_concurrent_processing(&data, 4);
        let concurrent_duration = concurrent_start.elapsed();
        
        report.add_test_result(TestResult {
            test_name: "concurrent_processing".to_string(),
            passed: concurrent_result > 0,
            duration: concurrent_duration,
            message: format!("并发处理结果: {}", concurrent_result),
        });
        
        // 生成报告
        let report_text = report.generate_report();
        println!("{}", report_text);
        
        // 验证所有测试都通过
        let all_passed = report.test_results.iter().all(|r| r.passed);
        assert!(all_passed);
    }
}