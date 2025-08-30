//! 测试数据生成器
//! 
//! 提供各种测试数据的生成函数，包括使用记录、配置等

use chrono::{DateTime, Utc, NaiveDate};
use serde_json::{json, Value};
use fake::{Fake, Faker};
use fake::faker::chrono::en::DateTimeBetween;
use fake::faker::internet::en::SafeEmail;
use fake::faker::lorem::en::Sentence;
use fake::faker::number::en::Digit;
use rand::Rng;
use crate::test_utils::{generate_random_string, generate_random_timestamp};

/// 测试数据生成器
pub struct TestDataGenerator {
    rng: rand::rngs::ThreadRng,
}

impl TestDataGenerator {
    /// 创建新的测试数据生成器
    pub fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
        }
    }

    /// 生成单个使用记录
    pub fn generate_usage_record(&mut self) -> Value {
        let models = vec![
            "claude-3-sonnet-20240229",
            "claude-3-opus-20240229",
            "claude-3-haiku-20240307",
            "claude-2.1",
            "claude-instant-1.2",
        ];

        let model = models[self.rng.gen_range(0..models.len())];
        let input_tokens: u32 = self.rng.gen_range(10..5000);
        let output_tokens: u32 = self.rng.gen_range(10..2000);
        
        // 简化的成本计算
        let cost_per_input = match model {
            "claude-3-opus-20240229" => 0.015,
            "claude-3-sonnet-20240229" => 0.003,
            "claude-3-haiku-20240307" => 0.00025,
            "claude-2.1" => 0.008,
            "claude-instant-1.2" => 0.0016,
            _ => 0.003,
        };

        let cost_per_output = match model {
            "claude-3-opus-20240229" => 0.075,
            "claude-3-sonnet-20240229" => 0.015,
            "claude-3-haiku-20240307" => 0.00125,
            "claude-2.1" => 0.024,
            "claude-instant-1.2" => 0.0056,
            _ => 0.015,
        };

        let cost = (input_tokens as f64 / 1000.0) * cost_per_input + 
                   (output_tokens as f64 / 1000.0) * cost_per_output;

        json!({
            "id": generate_random_string(20),
            "timestamp": generate_random_timestamp().to_rfc3339(),
            "model": model,
            "input_tokens": input_tokens,
            "output_tokens": output_tokens,
            "cost": cost,
            "session_id": Some(generate_random_string(16)),
            "user_id": Some(format!("user_{}", self.rng.gen_range(1..100))),
            "metadata": {
                "request_type": "chat",
                "success": true,
                "response_time": self.rng.gen_range(100..5000)
            }
        })
    }

    /// 生成多个使用记录
    pub fn generate_usage_records(&mut self, count: usize) -> Vec<Value> {
        (0..count).map(|_| self.generate_usage_record()).collect()
    }

    /// 生成指定日期范围的使用记录
    pub fn generate_usage_records_for_date_range(
        &mut self,
        start_date: NaiveDate,
        end_date: NaiveDate,
        records_per_day: usize,
    ) -> Vec<Value> {
        let mut records = Vec::new();
        let mut current_date = start_date;

        while current_date <= end_date {
            for _ in 0..records_per_day {
                let mut record = self.generate_usage_record();
                
                // 设置记录的日期
                let hour = self.rng.gen_range(0..24);
                let minute = self.rng.gen_range(0..60);
                let second = self.rng.gen_range(0..60);
                
                let timestamp = DateTime::from_naive_utc_and_offset(
                    current_date.and_hms_opt(hour, minute, second).unwrap(),
                    Utc,
                );

                record["timestamp"] = json!(timestamp.to_rfc3339());
                records.push(record);
            }
            
            current_date += chrono::Duration::days(1);
        }

        records
    }

    /// 生成测试配置
    pub fn generate_config(&mut self) -> Value {
        json!({
            "app": {
                "verbose": self.rng.gen_bool(0.3),
                "log_level": ["info", "debug", "warn"][self.rng.gen_range(0..3)]
            },
            "data": {
                "default_source": "json",
                "cache_enabled": self.rng.gen_bool(0.8),
                "cache_ttl": self.rng.gen_range(300..3600)
            },
            "output": {
                "default_format": ["table", "json", "csv"][self.rng.gen_range(0..3)],
                "date_format": "%Y-%m-%d %H:%M:%S",
                "decimal_places": 6
            },
            "analysis": {
                "default_model_pricing": {
                    "claude-3-sonnet-20240229": {
                        "input_cost_per_1k": 0.003,
                        "output_cost_per_1k": 0.015,
                        "currency": "USD"
                    },
                    "claude-3-opus-20240229": {
                        "input_cost_per_1k": 0.015,
                        "output_cost_per_1k": 0.075,
                        "currency": "USD"
                    }
                }
            },
            "budget": {
                "monthly_limit": self.rng.gen_range(50.0..500.0),
                "currency": "USD",
                "warning_threshold": 80.0,
                "alert_threshold": 95.0,
                "enable_alerts": true
            }
        })
    }

    /// 生成预算配置
    pub fn generate_budget_config(&mut self) -> Value {
        json!({
            "monthly_limit": self.rng.gen_range(50.0..500.0),
            "currency": ["USD", "EUR", "CNY"][self.rng.gen_range(0..3)],
            "warning_threshold": self.rng.gen_range(70.0..90.0),
            "alert_threshold": self.rng.gen_range(90.0..99.0),
            "enable_alerts": self.rng.gen_bool(0.9)
        })
    }

    /// 生成定价信息
    pub fn generate_pricing_info(&mut self) -> Value {
        let models = vec![
            "claude-3-sonnet-20240229",
            "claude-3-opus-20240229",
            "claude-3-haiku-20240307",
        ];

        let mut pricing = json!({});
        
        for model in models {
            pricing[model] = json!({
                "input_cost_per_1k": self.rng.gen_range(0.001..0.02),
                "output_cost_per_1k": self.rng.gen_range(0.005..0.1),
                "currency": "USD",
                "effective_date": generate_random_timestamp().to_rfc3339(),
                "is_active": true
            });
        }

        pricing
    }

    /// 生成会话数据
    pub fn generate_session(&mut self) -> Value {
        let start_time = generate_random_timestamp();
        let duration_hours = self.rng.gen_range(1..24);
        let end_time = start_time + chrono::Duration::hours(duration_hours);

        json!({
            "id": generate_random_string(16),
            "start_time": start_time.to_rfc3339(),
            "end_time": end_time.to_rfc3339(),
            "user_id": format!("user_{}", self.rng.gen_range(1..100)),
            "total_cost": self.rng.gen_range(0.1..10.0),
            "total_input_tokens": self.rng.gen_range(1000..50000),
            "total_output_tokens": self.rng.gen_range(500..20000),
            "request_count": self.rng.gen_range(1..100),
            "duration_seconds": duration_hours * 3600,
            "metadata": {
                "client_version": "1.0.0",
                "platform": ["web", "desktop", "mobile"][self.rng.gen_range(0..3)]
            }
        })
    }

    /// 生成报告配置
    pub fn generate_report_config(&mut self) -> Value {
        json!({
            "report_type": ["daily", "weekly", "monthly", "custom"][self.rng.gen_range(0..4)],
            "output_format": ["table", "json", "csv", "markdown"][self.rng.gen_range(0..4)],
            "include_breakdown": self.rng.gen_bool(0.7),
            "include_charts": self.rng.gen_bool(0.5),
            "filters": {
                "models": vec![
                    "claude-3-sonnet-20240229",
                    "claude-3-opus-20240229"
                ],
                "date_range": {
                    "start": generate_random_timestamp().to_rfc3339(),
                    "end": generate_random_timestamp().to_rfc3339()
                },
                "cost_range": {
                    "min": 0.0,
                    "max": 100.0
                }
            }
        })
    }

    /// 生成错误测试数据
    pub fn generate_error_test_data(&mut self) -> Value {
        let error_types = vec![
            "invalid_json",
            "missing_required_field",
            "invalid_date_format",
            "negative_tokens",
            "invalid_cost",
            "file_not_found",
            "permission_denied",
            "network_error",
        ];

        json!({
            "test_cases": error_types.iter().map(|error_type| {
                json!({
                    "error_type": error_type,
                    "description": format!("Test case for {}", error_type),
                    "expected_error": true,
                    "data": self.generate_invalid_data_for_error(error_type)
                })
            }).collect::<Vec<_>>()
        })
    }

    /// 为特定错误类型生成无效数据
    fn generate_invalid_data_for_error(&mut self, error_type: &str) -> Value {
        match error_type {
            "invalid_json" => json!({"invalid": "json", "missing": "quote}),
            "missing_required_field" => json!({
                "timestamp": generate_random_timestamp().to_rfc3339(),
                "model": "claude-3-sonnet-20240229",
                // missing input_tokens, output_tokens, cost
            }),
            "invalid_date_format" => json!({
                "timestamp": "invalid-date",
                "model": "claude-3-sonnet-20240229",
                "input_tokens": 1000,
                "output_tokens": 500,
                "cost": 0.015
            }),
            "negative_tokens" => json!({
                "timestamp": generate_random_timestamp().to_rfc3339(),
                "model": "claude-3-sonnet-20240229",
                "input_tokens": -100,
                "output_tokens": 500,
                "cost": 0.015
            }),
            "invalid_cost" => json!({
                "timestamp": generate_random_timestamp().to_rfc3339(),
                "model": "claude-3-sonnet-20240229",
                "input_tokens": 1000,
                "output_tokens": 500,
                "cost": -0.015
            }),
            _ => json!({"error": "unknown error type"})
        }
    }

    /// 生成性能测试数据
    pub fn generate_performance_test_data(&mut self, size: usize) -> Value {
        let mut records = Vec::new();
        
        // 生成大量记录用于性能测试
        for i in 0..size {
            let mut record = self.generate_usage_record();
            record["id"] = json!(format!("perf_test_{}", i));
            records.push(record);
        }

        json!({
            "test_name": format!("performance_test_{}", size),
            "record_count": size,
            "estimated_size_mb": size * 0.001, // 估算大小
            "data": records
        })
    }
}

impl Default for TestDataGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// 预定义的测试数据集
pub mod predefined {
    use super::*;

    /// 获取小型测试数据集
    pub fn small_dataset() -> Vec<Value> {
        let mut generator = TestDataGenerator::new();
        generator.generate_usage_records(10)
    }

    /// 获取中型测试数据集
    pub fn medium_dataset() -> Vec<Value> {
        let mut generator = TestDataGenerator::new();
        generator.generate_usage_records(100)
    }

    /// 获取大型测试数据集
    pub fn large_dataset() -> Vec<Value> {
        let mut generator = TestDataGenerator::new();
        generator.generate_usage_records(1000)
    }

    /// 获取包含特定模型的测试数据
    pub fn model_specific_dataset(model: &str, count: usize) -> Vec<Value> {
        let mut generator = TestDataGenerator::new();
        (0..count).map(|_| {
            let mut record = generator.generate_usage_record();
            record["model"] = json!(model);
            record
        }).collect()
    }

    /// 获取时间序列测试数据
    pub fn time_series_dataset(days: usize, records_per_day: usize) -> Vec<Value> {
        let mut generator = TestDataGenerator::new();
        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end_date = start_date + chrono::Duration::days(days as i64 - 1);
        
        generator.generate_usage_records_for_date_range(start_date, end_date, records_per_day)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_usage_record() {
        let mut generator = TestDataGenerator::new();
        let record = generator.generate_usage_record();
        
        assert!(record.is_object());
        assert!(record["id"].is_string());
        assert!(record["timestamp"].is_string());
        assert!(record["model"].is_string());
        assert!(record["input_tokens"].is_number());
        assert!(record["output_tokens"].is_number());
        assert!(record["cost"].is_number());
    }

    #[test]
    fn test_generate_usage_records() {
        let mut generator = TestDataGenerator::new();
        let records = generator.generate_usage_records(5);
        
        assert_eq!(records.len(), 5);
        for record in records {
            assert!(record.is_object());
        }
    }

    #[test]
    fn test_generate_config() {
        let mut generator = TestDataGenerator::new();
        let config = generator.generate_config();
        
        assert!(config.is_object());
        assert!(config["app"].is_object());
        assert!(config["data"].is_object());
        assert!(config["output"].is_object());
    }

    #[test]
    fn test_predefined_small_dataset() {
        let dataset = predefined::small_dataset();
        assert_eq!(dataset.len(), 10);
    }

    #[test]
    fn test_predefined_model_specific_dataset() {
        let dataset = predefined::model_specific_dataset("claude-3-sonnet-20240229", 5);
        assert_eq!(dataset.len(), 5);
        
        for record in dataset {
            assert_eq!(record["model"], "claude-3-sonnet-20240229");
        }
    }
}