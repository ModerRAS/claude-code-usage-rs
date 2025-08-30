//! 测试工具函数
//! 
//! 提供各种测试辅助函数，包括文件操作、临时目录管理等

use std::path::{Path, PathBuf};
use tempfile::{TempDir, NamedTempFile};
use std::fs;
use chrono::{DateTime, Utc, NaiveDate};
use serde_json::json;
use crate::test_data::TestDataGenerator;

/// 创建临时目录
pub fn create_temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temporary directory")
}

/// 创建临时文件
pub fn create_temp_file(content: &str) -> NamedTempFile {
    NamedTempFile::new()
        .expect("Failed to create temporary file")
        .write_all(content.as_bytes())
        .expect("Failed to write to temporary file")
}

/// 创建测试数据文件
pub fn create_test_data_file(data: &serde_json::Value, format: &str) -> PathBuf {
    let temp_dir = create_temp_dir();
    let file_path = temp_dir.path().join(format!("test_data.{}", format));
    
    match format {
        "json" => {
            fs::write(&file_path, serde_json::to_string_pretty(data).unwrap())
                .expect("Failed to write JSON test data");
        },
        "csv" => {
            // 简化的CSV格式用于测试
            let csv_content = generate_csv_from_json(data);
            fs::write(&file_path, csv_content)
                .expect("Failed to write CSV test data");
        },
        _ => panic!("Unsupported format: {}", format),
    }
    
    file_path
}

/// 从JSON生成CSV内容（简化实现）
fn generate_csv_from_json(data: &serde_json::Value) -> String {
    if let Some(records) = data.as_array() {
        if records.is_empty() {
            return String::new();
        }
        
        // 生成CSV头
        let headers = vec![
            "id", "timestamp", "model", "input_tokens", "output_tokens", "cost"
        ];
        let mut csv_content = headers.join(",") + "\n";
        
        // 生成CSV行
        for record in records {
            if let Some(obj) = record.as_object() {
                let row = vec![
                    obj.get("id").and_then(|v| v.as_str()).unwrap_or(""),
                    obj.get("timestamp").and_then(|v| v.as_str()).unwrap_or(""),
                    obj.get("model").and_then(|v| v.as_str()).unwrap_or(""),
                    obj.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0).to_string(),
                    obj.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0).to_string(),
                    obj.get("cost").and_then(|v| v.as_f64()).unwrap_or(0.0).to_string(),
                ];
                csv_content.push_str(&row.join(","));
                csv_content.push('\n');
            }
        }
        
        csv_content
    } else {
        String::new()
    }
}

/// 创建测试配置文件
pub fn create_test_config(config: &serde_json::Value) -> PathBuf {
    let temp_dir = create_temp_dir();
    let config_path = temp_dir.path().join("config.toml");
    
    let toml_content = generate_toml_from_json(config);
    fs::write(&config_path, toml_content)
        .expect("Failed to write test config");
    
    config_path
}

/// 从JSON生成TOML内容（简化实现）
fn generate_toml_from_json(config: &serde_json::Value) -> String {
    let mut toml_content = String::new();
    
    if let Some(obj) = config.as_object() {
        for (key, value) in obj {
            match value {
                serde_json::Value::String(s) => {
                    toml_content.push_str(&format!("{} = \"{}\"\n", key, s));
                },
                serde_json::Value::Number(n) => {
                    toml_content.push_str(&format!("{} = {}\n", key, n));
                },
                serde_json::Value::Bool(b) => {
                    toml_content.push_str(&format!("{} = {}\n", key, b));
                },
                serde_json::Value::Object(obj) => {
                    toml_content.push_str(&format!("[{}]\n", key));
                    for (sub_key, sub_value) in obj {
                        if let Some(sub_str) = sub_value.as_str() {
                            toml_content.push_str(&format!("{} = \"{}\"\n", sub_key, sub_str));
                        }
                    }
                },
                _ => {}
            }
        }
    }
    
    toml_content
}

/// 清理测试文件
pub fn cleanup_test_files(paths: &[PathBuf]) {
    for path in paths {
        if path.exists() {
            if path.is_dir() {
                fs::remove_dir_all(path).ok();
            } else {
                fs::remove_file(path).ok();
            }
        }
    }
}

/// 等待异步操作完成
pub async fn wait_for_async_operation() {
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
}

/// 测量函数执行时间
pub fn measure_execution_time<F, R>(f: F) -> (R, std::time::Duration)
where
    F: FnOnce() -> R,
{
    let start = std::time::Instant::now();
    let result = f();
    let duration = start.elapsed();
    (result, duration)
}

/// 测量异步函数执行时间
pub async fn measure_async_execution_time<F, R>(f: F) -> (R, std::time::Duration)
where
    F: std::future::Future<Output = R>,
{
    let start = std::time::Instant::now();
    let result = f.await;
    let duration = start.elapsed();
    (result, duration)
}

/// 创建测试环境变量
pub fn set_test_env_var(key: &str, value: &str) {
    std::env::set_var(key, value);
}

/// 清理测试环境变量
pub fn cleanup_test_env_vars(keys: &[&str]) {
    for key in keys {
        std::env::remove_var(key);
    }
}

/// 验证文件是否存在
pub fn assert_file_exists(path: &Path) {
    assert!(path.exists(), "File should exist: {:?}", path);
}

/// 验证文件内容
pub fn assert_file_content(path: &Path, expected_content: &str) {
    let content = fs::read_to_string(path)
        .expect(&format!("Failed to read file: {:?}", path));
    assert_eq!(content, expected_content, "File content mismatch");
}

/// 生成随机字符串
pub fn generate_random_string(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// 生成随机日期
pub fn generate_random_date() -> NaiveDate {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    let year = rng.gen_range(2020..=2024);
    let month = rng.gen_range(1..=12);
    let day = rng.gen_range(1..=28); // 简化处理，避免月份天数问题
    
    NaiveDate::from_ymd_opt(year, month, day)
        .expect("Invalid date generated")
}

/// 生成随机时间戳
pub fn generate_random_timestamp() -> DateTime<Utc> {
    let date = generate_random_date();
    DateTime::from_naive_utc_and_offset(
        date.and_hms_opt(12, 0, 0).unwrap(),
        Utc,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_temp_dir() {
        let temp_dir = create_temp_dir();
        assert!(temp_dir.path().exists());
        assert!(temp_dir.path().is_dir());
    }

    #[test]
    fn test_create_temp_file() {
        let content = "test content";
        let temp_file = create_temp_file(content);
        assert!(temp_file.path().exists());
        assert!(temp_file.path().is_file());
        
        let read_content = fs::read_to_string(temp_file.path()).unwrap();
        assert_eq!(read_content, content);
    }

    #[test]
    fn test_generate_random_string() {
        let s1 = generate_random_string(10);
        let s2 = generate_random_string(10);
        
        assert_eq!(s1.len(), 10);
        assert_eq!(s2.len(), 10);
        assert_ne!(s1, s2); // 很大概率不相同
    }

    #[test]
    fn test_generate_random_date() {
        let date = generate_random_date();
        assert!(date.year() >= 2020);
        assert!(date.year() <= 2024);
        assert!(date.month() >= 1);
        assert!(date.month() <= 12);
        assert!(date.day() >= 1);
        assert!(date.day() <= 28);
    }

    #[test]
    fn test_measure_execution_time() {
        let (result, duration) = measure_execution_time(|| {
            std::thread::sleep(std::time::Duration::from_millis(10));
            42
        });
        
        assert_eq!(result, 42);
        assert!(duration >= std::time::Duration::from_millis(10));
    }
}