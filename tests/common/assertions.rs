//! 自定义断言
//! 
//! 提供针对项目特定需求的断言函数

use chrono::{DateTime, Utc, Duration};
use serde_json::Value;
use crate::data::models::*;
use crate::error::CcusageError;
use pretty_assertions::assert_eq;

/// 使用记录断言
pub trait UsageRecordAssertions {
    fn assert_valid_usage_record(&self);
    fn assert_has_session_id(&self);
    fn assert_has_user_id(&self);
    fn assert_cost_positive(&self);
    fn assert_tokens_positive(&self);
    fn assert_model_is(&self, expected_model: &str);
    fn assert_is_within_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>);
}

impl UsageRecordAssertions for UsageRecord {
    fn assert_valid_usage_record(&self) {
        assert!(!self.id.is_empty(), "Record ID should not be empty");
        assert!(!self.model.is_empty(), "Model should not be empty");
        assert_cost_positive();
        assert_tokens_positive();
    }

    fn assert_has_session_id(&self) {
        assert!(self.session_id.is_some(), "Record should have session ID");
    }

    fn assert_has_user_id(&self) {
        assert!(self.user_id.is_some(), "Record should have user ID");
    }

    fn assert_cost_positive(&self) {
        assert!(self.cost >= 0.0, "Cost should be non-negative: {}", self.cost);
    }

    fn assert_tokens_positive(&self) {
        assert!(self.input_tokens > 0, "Input tokens should be positive: {}", self.input_tokens);
        assert!(self.output_tokens >= 0, "Output tokens should be non-negative: {}", self.output_tokens);
    }

    fn assert_model_is(&self, expected_model: &str) {
        assert_eq!(self.model, expected_model, "Model mismatch: expected {}, got {}", expected_model, self.model);
    }

    fn assert_is_within_date_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) {
        assert!(self.timestamp >= start, "Record timestamp {} is before start {}", self.timestamp, start);
        assert!(self.timestamp <= end, "Record timestamp {} is after end {}", self.timestamp, end);
    }
}

/// 会话断言
pub trait SessionAssertions {
    fn assert_valid_session(&self);
    fn assert_has_records(&self);
    fn assert_duration_calculated(&self);
    fn assert_total_cost_positive(&self);
    fn assert_request_count_positive(&self);
}

impl SessionAssertions for Session {
    fn assert_valid_session(&self) {
        assert!(!self.id.is_empty(), "Session ID should not be empty");
        assert_total_cost_positive();
        assert_request_count_positive();
    }

    fn assert_has_records(&self) {
        assert!(self.request_count > 0, "Session should have records: {}", self.request_count);
    }

    fn assert_duration_calculated(&self) {
        assert!(self.duration_seconds.is_some(), "Session duration should be calculated");
    }

    fn assert_total_cost_positive(&self) {
        assert!(self.total_cost >= 0.0, "Total cost should be non-negative: {}", self.total_cost);
    }

    fn assert_request_count_positive(&self) {
        assert!(self.request_count > 0, "Request count should be positive: {}", self.request_count);
    }
}

/// 错误断言
pub trait ErrorAssertions {
    fn assert_is_config_error(&self);
    fn assert_is_data_loading_error(&self);
    fn assert_is_file_system_error(&self);
    fn assert_is_network_error(&self);
    fn assert_is_validation_error(&self);
    fn assert_contains_message(&self, expected_message: &str);
}

impl ErrorAssertions for CcusageError {
    fn assert_is_config_error(&self) {
        match self {
            CcusageError::Config(_) => {},
            _ => panic!("Expected Config error, got: {:?}", self),
        }
    }

    fn assert_is_data_loading_error(&self) {
        match self {
            CcusageError::DataLoading(_) => {},
            _ => panic!("Expected DataLoading error, got: {:?}", self),
        }
    }

    fn assert_is_file_system_error(&self) {
        match self {
            CcusageError::FileSystem(_) => {},
            _ => panic!("Expected FileSystem error, got: {:?}", self),
        }
    }

    fn assert_is_network_error(&self) {
        match self {
            CcusageError::Network(_) => {},
            _ => panic!("Expected Network error, got: {:?}", self),
        }
    }

    fn assert_is_validation_error(&self) {
        match self {
            CcusageError::Validation(_) => {},
            _ => panic!("Expected Validation error, got: {:?}", self),
        }
    }

    fn assert_contains_message(&self, expected_message: &str) {
        let error_message = self.to_string();
        assert!(
            error_message.contains(expected_message),
            "Error message '{}' should contain '{}'",
            error_message,
            expected_message
        );
    }
}

/// Result断言
pub trait ResultAssertions<T> {
    fn assert_ok(self) -> T;
    fn assert_err(self) -> CcusageError;
    fn assert_error_contains(self, expected_message: &str) -> T;
}

impl<T> ResultAssertions<T> for Result<T, CcusageError> {
    fn assert_ok(self) -> T {
        match self {
            Ok(value) => value,
            Err(e) => panic!("Expected Ok, got Err: {:?}", e),
        }
    }

    fn assert_err(self) -> CcusageError {
        match self {
            Ok(_) => panic!("Expected Err, got Ok"),
            Err(e) => e,
        }
    }

    fn assert_error_contains(self, expected_message: &str) -> T {
        match self {
            Ok(value) => value,
            Err(e) => {
                e.assert_contains_message(expected_message);
                panic!("Expected error containing '{}', but got different error", expected_message);
            }
        }
    }
}

/// Vec断言
pub trait VecAssertions<T> {
    fn assert_not_empty(self) -> Self;
    fn assert_has_length(self, expected_length: usize) -> Self;
    fn assert_contains(self, expected_item: &T) -> Self
    where
        T: PartialEq + std::fmt::Debug;
    fn assert_all_match<F>(self, predicate: F) -> Self
    where
        F: Fn(&T) -> bool;
}

impl<T> VecAssertions<T> for Vec<T> {
    fn assert_not_empty(self) -> Self {
        assert!(!self.is_empty(), "Vector should not be empty");
        self
    }

    fn assert_has_length(self, expected_length: usize) -> Self {
        assert_eq!(self.len(), expected_length, "Vector length mismatch: expected {}, got {}", expected_length, self.len());
        self
    }

    fn assert_contains(self, expected_item: &T) -> Self
    where
        T: PartialEq + std::fmt::Debug,
    {
        assert!(self.contains(expected_item), "Vector should contain item: {:?}", expected_item);
        self
    }

    fn assert_all_match<F>(self, predicate: F) -> Self
    where
        F: Fn(&T) -> bool,
    {
        for (index, item) in self.iter().enumerate() {
            assert!(predicate(item), "Item at index {} does not match predicate: {:?}", index, item);
        }
        self
    }
}

/// 数值断言
pub trait NumericAssertions {
    fn assert_positive(self) -> Self;
    fn assert_non_negative(self) -> Self;
    fn assert_in_range(self, min: f64, max: f64) -> Self;
    fn assert_approx_eq(self, expected: f64, tolerance: f64) -> Self;
}

impl NumericAssertions for f64 {
    fn assert_positive(self) -> Self {
        assert!(self > 0.0, "Value should be positive: {}", self);
        self
    }

    fn assert_non_negative(self) -> Self {
        assert!(self >= 0.0, "Value should be non-negative: {}", self);
        self
    }

    fn assert_in_range(self, min: f64, max: f64) -> Self {
        assert!(self >= min && self <= max, "Value {} should be in range [{}, {}]", self, min, max);
        self
    }

    fn assert_approx_eq(self, expected: f64, tolerance: f64) -> Self {
        let diff = (self - expected).abs();
        assert!(diff <= tolerance, "Value {} is not approximately equal to {} (tolerance: {})", self, expected, tolerance);
        self
    }
}

impl NumericAssertions for u32 {
    fn assert_positive(self) -> Self {
        assert!(self > 0, "Value should be positive: {}", self);
        self
    }

    fn assert_non_negative(self) -> Self {
        assert!(self >= 0, "Value should be non-negative: {}", self);
        self
    }

    fn assert_in_range(self, min: f64, max: f64) -> Self {
        assert!((self as f64) >= min && (self as f64) <= max, "Value {} should be in range [{}, {}]", self, min, max);
        self
    }

    fn assert_approx_eq(self, expected: f64, tolerance: f64) -> Self {
        let diff = (self as f64 - expected).abs();
        assert!(diff <= tolerance, "Value {} is not approximately equal to {} (tolerance: {})", self, expected, tolerance);
        self
    }
}

/// JSON断言
pub trait JsonAssertions {
    fn assert_is_object(self) -> Self;
    fn assert_is_array(self) -> Self;
    fn assert_has_key(self, key: &str) -> Self;
    fn assert_array_has_length(self, expected_length: usize) -> Self;
    fn assert_value_equals(self, key: &str, expected_value: &Value) -> Self;
}

impl JsonAssertions for Value {
    fn assert_is_object(self) -> Self {
        assert!(self.is_object(), "JSON value should be an object: {:?}", self);
        self
    }

    fn assert_is_array(self) -> Self {
        assert!(self.is_array(), "JSON value should be an array: {:?}", self);
        self
    }

    fn assert_has_key(self, key: &str) -> Self {
        assert!(self.get(key).is_some(), "JSON object should have key '{}': {:?}", key, self);
        self
    }

    fn assert_array_has_length(self, expected_length: usize) -> Self {
        if let Some(array) = self.as_array() {
            assert_eq!(array.len(), expected_length, "JSON array length mismatch: expected {}, got {}", expected_length, array.len());
        } else {
            panic!("JSON value should be an array: {:?}", self);
        }
        self
    }

    fn assert_value_equals(self, key: &str, expected_value: &Value) -> Self {
        if let Some(value) = self.get(key) {
            assert_eq!(value, expected_value, "JSON value for key '{}' mismatch: expected {:?}, got {:?}", key, expected_value, value);
        } else {
            panic!("JSON object should have key '{}': {:?}", key, self);
        }
        self
    }
}

/// 时间断言
pub trait TimeAssertions {
    fn assert_is_recent(self, within: Duration) -> Self;
    fn assert_is_before(self, other: DateTime<Utc>) -> Self;
    fn assert_is_after(self, other: DateTime<Utc>) -> Self;
    fn assert_is_between(self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self;
}

impl TimeAssertions for DateTime<Utc> {
    fn assert_is_recent(self, within: Duration) -> Self {
        let now = Utc::now();
        let diff = now.signed_duration_since(self);
        assert!(diff <= within, "Time {} is not recent (should be within {} of now {})", self, within, now);
        self
    }

    fn assert_is_before(self, other: DateTime<Utc>) -> Self {
        assert!(self < other, "Time {} should be before {}", self, other);
        self
    }

    fn assert_is_after(self, other: DateTime<Utc>) -> Self {
        assert!(self > other, "Time {} should be after {}", self, other);
        self
    }

    fn assert_is_between(self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        assert!(self >= start && self <= end, "Time {} should be between {} and {}", self, start, end);
        self
    }
}

/// 性能断言
pub trait PerformanceAssertions {
    fn assert_under_threshold_ms(self, threshold_ms: u64) -> Self;
    fn assert_under_threshold_sec(self, threshold_sec: f64) -> Self;
}

impl PerformanceAssertions for std::time::Duration {
    fn assert_under_threshold_ms(self, threshold_ms: u64) -> Self {
        assert!(self.as_millis() <= threshold_ms as u128, "Duration {:?} exceeds threshold {}ms", self, threshold_ms);
        self
    }

    fn assert_under_threshold_sec(self, threshold_sec: f64) -> Self {
        assert!(self.as_secs_f64() <= threshold_sec, "Duration {:?} exceeds threshold {}s", self, threshold_sec);
        self
    }
}

/// 文件系统断言
pub trait FileSystemAssertions {
    fn assert_file_exists(self);
    fn assert_file_not_exists(self);
    fn assert_dir_exists(self);
    fn assert_file_contains(self, expected_content: &str);
    fn assert_file_size_at_least(self, min_size: u64);
}

impl FileSystemAssertions for std::path::PathBuf {
    fn assert_file_exists(self) {
        assert!(self.exists(), "File should exist: {:?}", self);
        assert!(self.is_file(), "Path should be a file: {:?}", self);
    }

    fn assert_file_not_exists(self) {
        assert!(!self.exists(), "File should not exist: {:?}", self);
    }

    fn assert_dir_exists(self) {
        assert!(self.exists(), "Directory should exist: {:?}", self);
        assert!(self.is_dir(), "Path should be a directory: {:?}", self);
    }

    fn assert_file_contains(self, expected_content: &str) {
        let content = std::fs::read_to_string(&self)
            .expect(&format!("Failed to read file: {:?}", self));
        assert!(content.contains(expected_content), "File {:?} should contain '{}', but got: {}", self, expected_content, content);
    }

    fn assert_file_size_at_least(self, min_size: u64) {
        let metadata = std::fs::metadata(&self)
            .expect(&format!("Failed to get metadata for: {:?}", self));
        assert!(metadata.len() >= min_size, "File {:?} size {} should be at least {}", self, metadata.len(), min_size);
    }
}

/// 测试断言宏
#[macro_export]
macro_rules! assert_result_ok {
    ($result:expr) => {
        match $result {
            Ok(value) => value,
            Err(e) => panic!("Expected Ok, got Err: {:?}", e),
        }
    };
}

#[macro_export]
macro_rules! assert_result_err {
    ($result:expr) => {
        match $result {
            Ok(_) => panic!("Expected Err, got Ok"),
            Err(e) => e,
        }
    };
}

#[macro_export]
macro_rules! assert_vec_not_empty {
    ($vec:expr) => {
        assert!(!$vec.is_empty(), "Vector should not be empty");
    };
}

#[macro_export]
macro_rules! assert_vec_has_length {
    ($vec:expr, $length:expr) => {
        assert_eq!($vec.len(), $length, "Vector length mismatch");
    };
}

#[macro_export]
macro_rules! assert_positive {
    ($value:expr) => {
        assert!($value > 0, "Value should be positive: {}", $value);
    };
}

#[macro_export]
macro_rules! assert_non_negative {
    ($value:expr) => {
        assert!($value >= 0, "Value should be non-negative: {}", $value);
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_usage_record_assertions() {
        let record = UsageRecord::new(
            Utc::now(),
            "claude-3-sonnet".to_string(),
            1000,
            500,
            0.015,
        );

        record.assert_valid_usage_record();
        record.assert_cost_positive();
        record.assert_tokens_positive();
        record.assert_model_is("claude-3-sonnet");
    }

    #[test]
    fn test_session_assertions() {
        let mut session = Session::new(
            "test_session".to_string(),
            Utc::now(),
            Some("test_user".to_string()),
        );

        let record = UsageRecord::new(
            Utc::now(),
            "claude-3-sonnet".to_string(),
            1000,
            500,
            0.015,
        );

        session.add_record(&record);
        session.calculate_duration();

        session.assert_valid_session();
        session.assert_has_records();
        session.assert_duration_calculated();
        session.assert_total_cost_positive();
        session.assert_request_count_positive();
    }

    #[test]
    fn test_error_assertions() {
        let error = CcusageError::Config("test error".to_string());
        error.assert_is_config_error();
        error.assert_contains_message("test error");

        let error = CcusageError::DataLoading("data error".to_string());
        error.assert_is_data_loading_error();
        error.assert_contains_message("data error");
    }

    #[test]
    fn test_result_assertions() {
        let result: Result<i32, CcusageError> = Ok(42);
        let value = result.assert_ok();
        assert_eq!(value, 42);

        let result: Result<i32, CcusageError> = Err(CcusageError::Config("error".to_string()));
        let error = result.assert_err();
        error.assert_is_config_error();
    }

    #[test]
    fn test_vec_assertions() {
        let vec = vec![1, 2, 3, 4, 5];
        
        vec.clone()
            .assert_not_empty()
            .assert_has_length(5)
            .assert_contains(&3)
            .assert_all_match(|&x| x > 0);
    }

    #[test]
    fn test_numeric_assertions() {
        42.0.assert_positive();
        0.0.assert_non_negative();
        5.0.assert_in_range(1.0, 10.0);
        5.001.assert_approx_eq(5.0, 0.01);

        42u32.assert_positive();
        0u32.assert_non_negative();
        5u32.assert_in_range(1.0, 10.0);
    }

    #[test]
    fn test_json_assertions() {
        let json_value = json!({
            "name": "test",
            "value": 42,
            "items": [1, 2, 3]
        });

        json_value.clone()
            .assert_is_object()
            .assert_has_key("name")
            .assert_value_equals("name", &json!("test"));

        let array_value = json!([1, 2, 3]);
        array_value.clone()
            .assert_is_array()
            .assert_array_has_length(3);
    }

    #[test]
    fn test_time_assertions() {
        let now = Utc::now();
        let past = now - Duration::hours(1);
        let future = now + Duration::hours(1);

        past.assert_is_before(now);
        future.assert_is_after(now);
        now.assert_is_between(past, future);
        now.assert_is_recent(Duration::minutes(1));
    }

    #[test]
    fn test_performance_assertions() {
        let duration = std::time::Duration::from_millis(100);
        duration.assert_under_threshold_ms(200);
        duration.assert_under_threshold_sec(0.5);
    }

    #[test]
    fn test_macros() {
        let result: Result<i32, CcusageError> = Ok(42);
        let value = assert_result_ok!(result);
        assert_eq!(value, 42);

        let result: Result<i32, CcusageError> = Err(CcusageError::Config("error".to_string()));
        let _error = assert_result_err!(result);

        let vec = vec![1, 2, 3];
        assert_vec_not_empty!(vec);
        assert_vec_has_length!(vec, 3);
        assert_positive!(42);
        assert_non_negative!(0);
    }
}