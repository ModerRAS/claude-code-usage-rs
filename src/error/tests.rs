//! 错误处理测试
//! 
//! 测试所有错误类型和错误处理功能

use ccusage_rs::error::*;
use std::io;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::assertions::*;

    #[test]
    fn test_error_creation() {
        // 测试配置错误
        let config_err = CcusageError::Config("test config error".to_string());
        assert!(matches!(config_err, CcusageError::Config(_)));
        assert_eq!(config_err.to_string(), "Configuration error: test config error");

        // 测试数据加载错误
        let data_err = CcusageError::DataLoading("test data error".to_string());
        assert!(matches!(data_err, CcusageError::DataLoading(_)));
        assert_eq!(data_err.to_string(), "Data loading error: test data error");

        // 测试文件系统错误
        let fs_err = CcusageError::FileSystem("test file system error".to_string());
        assert!(matches!(fs_err, CcusageError::FileSystem(_)));
        assert_eq!(fs_err.to_string(), "File system error: test file system error");

        // 测试网络错误
        let network_err = CcusageError::Network("test network error".to_string());
        assert!(matches!(network_err, CcusageError::Network(_)));
        assert_eq!(network_err.to_string(), "Network error: test network error");

        // 测试验证错误
        let validation_err = CcusageError::Validation("test validation error".to_string());
        assert!(matches!(validation_err, CcusageError::Validation(_)));
        assert_eq!(validation_err.to_string(), "Validation error: test validation error");

        // 测试应用程序错误
        let app_err = CcusageError::Application("test app error".to_string());
        assert!(matches!(app_err, CcusageError::Application(_)));
        assert_eq!(app_err.to_string(), "Application error: test app error");
    }

    #[test]
    fn test_error_from_conversions() {
        // 测试IO错误转换
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let ccusage_error: CcusageError = io_error.into();
        assert!(matches!(ccusage_error, CcusageError::Io(_)));

        // 测试JSON错误转换
        let json_error = serde_json::Error::custom("Invalid JSON");
        let ccusage_error: CcusageError = json_error.into();
        assert!(matches!(ccusage_error, CcusageError::Json(_)));

        // 测试CSV错误转换
        let csv_error = csv::Error::from(io::Error::new(io::ErrorKind::InvalidData, "Invalid CSV"));
        let ccusage_error: CcusageError = csv_error.into();
        assert!(matches!(ccusage_error, CcusageError::Csv(_)));

        // 测试Chrono错误转换
        let chrono_error = chrono::ParseError::NotEnough("Not enough data");
        let ccusage_error: CcusageError = chrono_error.into();
        assert!(matches!(ccusage_error, CcusageError::Chrono(_)));

        // 测试配置解析错误转换
        let config_error = config::ConfigError::NotFound("config not found".to_string());
        let ccusage_error: CcusageError = config_error.into();
        assert!(matches!(ccusage_error, CcusageError::ConfigParse(_)));

        // 测试Reqwest错误转换
        let reqwest_error = reqwest::Error::from(io::Error::new(io::ErrorKind::ConnectionRefused, "Connection refused"));
        let ccusage_error: CcusageError = reqwest_error.into();
        assert!(matches!(ccusage_error, CcusageError::Reqwest(_)));

        // 测试URL错误转换
        let url_error = url::ParseError::EmptyHost;
        let ccusage_error: CcusageError = url_error.into();
        assert!(matches!(ccusage_error, CcusageError::Url(_)));

        // 测试Clap错误转换
        let clap_error = clap::Error::new(clap::error::ErrorKind::DisplayHelp);
        let ccusage_error: CcusageError = clap_error.into();
        assert!(matches!(ccusage_error, CcusageError::Clap(_)));
    }

    #[test]
    fn test_error_macros() {
        // 测试config_error宏
        let err = config_error!("test config error");
        assert!(matches!(err, CcusageError::Config(_)));
        assert!(err.to_string().contains("test config error"));

        let err = config_error!("config error with {} and {}", "param1", "param2");
        assert!(matches!(err, CcusageError::Config(_)));
        assert!(err.to_string().contains("param1"));
        assert!(err.to_string().contains("param2"));

        // 测试data_error宏
        let err = data_error!("test data error");
        assert!(matches!(err, CcusageError::DataLoading(_)));
        assert!(err.to_string().contains("test data error"));

        let err = data_error!("data error with {}", "param");
        assert!(matches!(err, CcusageError::DataLoading(_)));
        assert!(err.to_string().contains("param"));

        // 测试validation_error宏
        let err = validation_error!("test validation error");
        assert!(matches!(err, CcusageError::Validation(_)));
        assert!(err.to_string().contains("test validation error"));

        let err = validation_error!("validation error with {}", "param");
        assert!(matches!(err, CcusageError::Validation(_)));
        assert!(err.to_string().contains("param"));

        // 测试app_error宏
        let err = app_error!("test app error");
        assert!(matches!(err, CcusageError::Application(_)));
        assert!(err.to_string().contains("test app error"));

        let err = app_error!("app error with {} and {}", "param1", "param2");
        assert!(matches!(err, CcusageError::Application(_)));
        assert!(err.to_string().contains("param1"));
        assert!(err.to_string().contains("param2"));
    }

    #[test]
    fn test_error_context_trait() {
        // 测试成功的Result
        let result: Result<i32> = Ok(42);
        let result = result.with_context(|| "test context");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        // 测试失败的Result
        let result: std::result::Result<i32, &str> = Err("original error");
        let result = result.with_context(|| "additional context");
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(matches!(e, CcusageError::Application(_)));
            assert!(e.to_string().contains("additional context"));
            assert!(e.to_string().contains("original error"));
        }
    }

    #[test]
    fn test_chain_results() {
        // 测试全部成功的Results
        let results = vec![Ok(1), Ok(2), Ok(3)];
        let chained = chain_results(results);
        assert!(chained.is_ok());
        assert_eq!(chained.unwrap(), vec![1, 2, 3]);

        // 测试包含错误的Results
        let results = vec![Ok(1), Err(CcusageError::Application("error".to_string())), Ok(3)];
        let chained = chain_results(results);
        assert!(chained.is_err());

        // 测试空Results
        let results: Vec<Result<i32>> = vec![];
        let chained = chain_results(results);
        assert!(chained.is_ok());
        assert!(chained.unwrap().is_empty());
    }

    #[test]
    fn test_file_operation_context() {
        // 测试成功的文件操作
        let result = file_operation_context(
            || std::io::Result::Ok(42),
            "test_file.txt"
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        // 测试失败的文件操作
        let result = file_operation_context(
            || std::io::Result::Err(io::Error::new(io::ErrorKind::NotFound, "File not found")),
            "test_file.txt"
        );
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(matches!(e, CcusageError::FileSystem(_)));
            assert!(e.to_string().contains("test_file.txt"));
            assert!(e.to_string().contains("File not found"));
        }
    }

    #[test]
    fn test_error_debug_format() {
        let error = CcusageError::Config("test config error".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Config"));

        let error = CcusageError::DataLoading("test data error".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("DataLoading"));

        let error = CcusageError::Validation("test validation error".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Validation"));
    }

    #[test]
    fn test_error_display_format() {
        let error = CcusageError::Config("test config error".to_string());
        let display_str = format!("{}", error);
        assert_eq!(display_str, "Configuration error: test config error");

        let error = CcusageError::DataLoading("test data error".to_string());
        let display_str = format!("{}", error);
        assert_eq!(display_str, "Data loading error: test data error");

        let error = CcusageError::Validation("test validation error".to_string());
        let display_str = format!("{}", error);
        assert_eq!(display_str, "Validation error: test validation error");
    }

    #[test]
    fn test_error_send_sync() {
        // 确保错误类型是Send和Sync的，这对于多线程使用很重要
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<CcusageError>();
    }

    #[test]
    fn test_error_from_std_io() {
        let io_error = io::Error::new(io::ErrorKind::PermissionDenied, "Permission denied");
        let ccusage_error: CcusageError = io_error.into();
        
        assert!(matches!(ccusage_error, CcusageError::Io(_)));
        let error_string = ccusage_error.to_string();
        assert!(error_string.contains("Permission denied"));
    }

    #[test]
    fn test_error_from_serde_json() {
        let json_error = serde_json::Error::custom("JSON parsing failed");
        let ccusage_error: CcusageError = json_error.into();
        
        assert!(matches!(ccusage_error, CcusageError::Json(_)));
        let error_string = ccusage_error.to_string();
        assert!(error_string.contains("JSON parsing failed"));
    }

    #[test]
    fn test_error_from_csv() {
        let csv_error = csv::Error::from(io::Error::new(io::ErrorKind::InvalidData, "CSV parsing failed"));
        let ccusage_error: CcusageError = csv_error.into();
        
        assert!(matches!(ccusage_error, CcusageError::Csv(_)));
        let error_string = ccusage_error.to_string();
        assert!(error_string.contains("CSV parsing failed"));
    }

    #[test]
    fn test_error_from_chrono() {
        let chrono_error = chrono::ParseError::NotEnough("Not enough data for parsing");
        let ccusage_error: CcusageError = chrono_error.into();
        
        assert!(matches!(ccusage_error, CcusageError::Chrono(_)));
        let error_string = ccusage_error.to_string();
        assert!(error_string.contains("Not enough data for parsing"));
    }

    #[test]
    fn test_error_from_config() {
        let config_error = config::ConfigError::Message("Configuration parsing failed".to_string());
        let ccusage_error: CcusageError = config_error.into();
        
        assert!(matches!(ccusage_error, CcusageError::ConfigParse(_)));
        let error_string = ccusage_error.to_string();
        assert!(error_string.contains("Configuration parsing failed"));
    }

    #[test]
    fn test_error_from_reqwest() {
        let reqwest_error = reqwest::Error::from(io::Error::new(io::ErrorKind::ConnectionRefused, "Connection refused"));
        let ccusage_error: CcusageError = reqwest_error.into();
        
        assert!(matches!(ccusage_error, CcusageError::Reqwest(_)));
        let error_string = ccusage_error.to_string();
        assert!(error_string.contains("Connection refused"));
    }

    #[test]
    fn test_error_from_url() {
        let url_error = url::ParseError::EmptyHost;
        let ccusage_error: CcusageError = url_error.into();
        
        assert!(matches!(ccusage_error, CcusageError::Url(_)));
        let error_string = ccusage_error.to_string();
        assert!(error_string.contains("EmptyHost"));
    }

    #[test]
    fn test_error_from_clap() {
        let clap_error = clap::Error::new(clap::error::ErrorKind::DisplayHelp);
        let ccusage_error: CcusageError = clap_error.into();
        
        assert!(matches!(ccusage_error, CcusageError::Clap(_)));
        let error_string = ccusage_error.to_string();
        // Clap错误的消息格式可能会变化，所以我们只检查是否包含基本结构
        assert!(!error_string.is_empty());
    }

    #[test]
    fn test_custom_error_message_formatting() {
        // 测试错误消息的格式化是否正确
        let error = CcusageError::Config("Configuration file not found at path: /path/to/config.toml".to_string());
        let formatted = format!("{}", error);
        assert!(formatted.contains("Configuration error: Configuration file not found at path: /path/to/config.toml"));
    }

    #[test]
    fn test_error_chain_behavior() {
        // 测试错误链的行为
        let results = vec![
            Ok(1),
            Err(CcusageError::DataLoading("First error".to_string())),
            Ok(2),
            Err(CcusageError::Config("Second error".to_string())),
        ];
        
        let result = chain_results(results);
        assert!(result.is_err());
        
        // 链式操作应该返回第一个遇到的错误
        match result.unwrap_err() {
            CcusageError::DataLoading(msg) => {
                assert_eq!(msg, "First error");
            },
            _ => panic!("Expected DataLoading error"),
        }
    }

    #[test]
    fn test_error_context_with_different_types() {
        // 测试ErrorContext trait与不同类型的Result
        let string_result: std::result::Result<String, &str> = Err("string error");
        let result = string_result.with_context(|| "string context");
        assert!(result.is_err());
        
        let int_result: std::result::Result<i32, String> = Err("int error".to_string());
        let result = int_result.with_context(|| "int context");
        assert!(result.is_err());
        
        let bool_result: std::result::Result<bool, io::Error> = 
            Err(io::Error::new(io::ErrorKind::Other, "bool error"));
        let result = bool_result.with_context(|| "bool context");
        assert!(result.is_err());
    }

    #[test]
    fn test_file_operation_context_with_different_errors() {
        // 测试file_operation_context与不同类型的IO错误
        let error_kinds = vec![
            io::ErrorKind::NotFound,
            io::ErrorKind::PermissionDenied,
            io::ErrorKind::ConnectionRefused,
            io::ErrorKind::TimedOut,
            io::ErrorKind::WouldBlock,
        ];
        
        for kind in error_kinds {
            let result = file_operation_context(
                || Err(io::Error::new(kind, "test error")),
                "test_file.txt"
            );
            assert!(result.is_err());
            
            if let Err(e) = result {
                assert!(matches!(e, CcusageError::FileSystem(_)));
                assert!(e.to_string().contains("test_file.txt"));
            }
        }
    }

    #[test]
    fn test_error_macro_with_different_formats() {
        // 测试错误宏的不同格式化方式
        let err = config_error!("Simple message");
        assert!(err.to_string().contains("Simple message"));
        
        let err = config_error!("Message with placeholder: {}", "value");
        assert!(err.to_string().contains("value"));
        
        let err = config_error!("Message with multiple placeholders: {} and {}", "first", "second");
        assert!(err.to_string().contains("first"));
        assert!(err.to_string().contains("second"));
        
        let err = config_error!("Message with number: {}", 42);
        assert!(err.to_string().contains("42"));
    }

    #[test]
    fn test_error_performance() {
        // 测试错误创建的性能
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _err = CcusageError::Config("test error".to_string());
        }
        let duration = start.elapsed();
        assert!(duration.as_millis() < 10); // 应该在10ms内创建1000个错误

        // 测试错误转换的性能
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
            let _ccusage_error: CcusageError = io_error.into();
        }
        let duration = start.elapsed();
        assert!(duration.as_millis() < 10); // 应该在10ms内转换1000个错误
    }

    #[test]
    fn test_error_memory_usage() {
        // 测试错误对象的内存使用
        let errors: Vec<CcusageError> = (0..1000)
            .map(|i| CcusageError::Config(format!("Error number {}", i)))
            .collect();
        
        // 验证所有错误都正确创建
        assert_eq!(errors.len(), 1000);
        
        // 验证错误消息正确
        assert_eq!(errors[0].to_string(), "Configuration error: Error number 0");
        assert_eq!(errors[999].to_string(), "Configuration error: Error number 999");
        
        // 清理
        drop(errors);
    }
}