//! 独立测试模块
//! 不依赖于项目lib.rs的测试

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
        use std::thread;
        use std::sync::Arc;
        
        let mut generator = TestDataGenerator::new();
        let data = Arc::new(generator.generate_large_dataset(10000));
        
        let mut handles = vec![];
        
        // 创建4个线程并发处理数据
        for i in 0..4 {
            let data_clone = data.clone();
            let handle = thread::spawn(move || {
                let chunk_size = data_clone.len() / 4;
                let start = i * chunk_size;
                let end = if i == 3 { data_clone.len() } else { start + chunk_size };
                
                data_clone[start..end]
                    .iter()
                    .filter(|record| record.input_tokens + record.output_tokens > 5000)
                    .count()
            });
            handles.push(handle);
        }
        
        let total_count: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();
        println!("并发处理结果: {} 条记录满足条件", total_count);
        
        // 验证结果
        let expected_count = data.iter()
            .filter(|record| record.input_tokens + record.output_tokens > 5000)
            .count();
        
        assert_eq!(total_count, expected_count);
    }
    
    #[test]
    fn test_error_handling() {
        // 测试无效数据创建
        let timestamp = Utc::now();
        let record = TestUsageRecord::new(timestamp, "test".to_string(), 0, 0, 0.0);
        assert!(!test_utils::validate_usage_record(&record));
        
        // 测试JSON序列化错误处理
        let invalid_json = "{ invalid json }";
        let result: Result<Vec<TestUsageRecord>, _> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
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
}