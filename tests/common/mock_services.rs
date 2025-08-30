//! 模拟服务
//! 
//! 提供各种外部依赖的模拟实现，用于隔离测试

use mockall::*;
use std::collections::HashMap;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use serde_json::Value;
use async_trait::async_trait;

// 模拟数据加载器
mock! {
    pub DataLoader {
        pub fn new() -> Self;
        pub fn with_source(source_type: String, path: String) -> Self;
        
        pub async fn load_usage_data(&self) -> Result<Vec<crate::data::models::UsageRecord>, crate::error::CcusageError>;
        pub async fn load_config(&self) -> Result<crate::config::Config, crate::error::CcusageError>;
        pub async fn save_usage_data(&self, records: &[crate::data::models::UsageRecord]) -> Result<(), crate::error::CcusageError>;
    }
}

// 模拟配置管理器
mock! {
    pub ConfigManager {
        pub fn new() -> Result<Self, crate::error::CcusageError>;
        pub fn new_with_config(path: &PathBuf) -> Result<Self, crate::error::CcusageError>;
        
        pub fn get_config(&self) -> crate::config::Config;
        pub fn set_config(&mut self, config: crate::config::Config) -> Result<(), crate::error::CcusageError>;
        pub fn get_data_source_path(&self, source_type: &str) -> Option<String>;
        pub fn set_data_source_path(&mut self, source_type: &str, path: &str) -> Result<(), crate::error::CcusageError>;
        pub fn get_budget(&self) -> Option<crate::config::BudgetConfig>;
        pub fn set_budget(&mut self, budget: crate::config::BudgetConfig) -> Result<(), crate::error::CcusageError>;
        
        pub fn export_config(&self) -> Result<String, crate::error::CcusageError>;
        pub fn import_config(&mut self, config_str: &str) -> Result<(), crate::error::CcusageError>;
        pub fn reset_to_defaults(&mut self) -> Result<(), crate::error::CcusageError>;
        pub fn validate_config(&self) -> Result<(), crate::error::CcusageError>;
        pub fn save_current_config(&mut self) -> Result<(), crate::error::CcusageError>;
        
        pub fn set_verbose(&mut self, verbose: bool) -> Result<(), crate::error::CcusageError>;
    }
}

// 模拟输出格式化器
mock! {
    pub OutputFormatter {
        pub fn new(format: crate::output::OutputFormat) -> Self;
        
        pub fn output_usage_stats(&self, stats: &crate::analysis::statistics::UsageStats, output_path: Option<&str>) -> Result<(), crate::error::CcusageError>;
        pub fn output_cost_breakdown(&self, breakdown: &crate::analysis::cost::CostBreakdown, output_path: Option<&str>) -> Result<(), crate::error::CcusageError>;
        pub fn output_daily_report(&self, summary: &crate::data::models::DailySummary, comparison: Option<(&crate::data::models::DailySummary, &crate::data::models::DailySummary)>, output_path: Option<&str>) -> Result<(), crate::error::CcusageError>;
        pub fn output_weekly_report(&self, summaries: &[crate::data::models::WeeklySummary], output_path: Option<&str>) -> Result<(), crate::error::CcusageError>;
        pub fn output_monthly_report(&self, summary: &crate::data::models::MonthlySummary, comparison: Option<(&crate::data::models::MonthlySummary, &crate::data::models::MonthlySummary)>, output_path: Option<&str>) -> Result<(), crate::error::CcusageError>;
        pub fn output_session_analysis(&self, analysis: &crate::analysis::session::SessionAnalysis, output_path: Option<&str>) -> Result<(), crate::error::CcusageError>;
        pub fn output_budget_status(&self, budget: &crate::config::BudgetConfig, analysis: &crate::analysis::budget::BudgetAnalysis, output_path: Option<&str>) -> Result<(), crate::error::CcusageError>;
        pub fn output_insights(&self, insights: &[crate::analysis::insights::Insight], output_path: Option<&str>) -> Result<(), crate::error::CcusageError>;
        pub fn output_comprehensive_analysis(&self, breakdown: &crate::analysis::cost::CostBreakdown, stats: &crate::analysis::statistics::UsageStats, trends: &crate::analysis::trends::UsageTrends, output_path: Option<&str>) -> Result<(), crate::error::CcusageError>;
        pub fn export_data(&self, records: &[crate::data::models::UsageRecord], format: crate::output::ExportFormat, output_path: &PathBuf) -> Result<(), crate::error::CcusageError>;
    }
}

// 模拟HTTP客户端
#[async_trait]
pub trait MockHttpClient {
    async fn get(&self, url: &str) -> Result<Value, crate::error::CcusageError>;
    async fn post(&self, url: &str, data: &Value) -> Result<Value, crate::error::CcusageError>;
    async fn put(&self, url: &str, data: &Value) -> Result<Value, crate::error::CcusageError>;
    async fn delete(&self, url: &str) -> Result<Value, crate::error::CcusageError>;
}

mock! {
    pub HttpClient {}

    #[async_trait]
    impl MockHttpClient for HttpClient {
        async fn get(&self, url: &str) -> Result<Value, crate::error::CcusageError>;
        async fn post(&self, url: &str, data: &Value) -> Result<Value, crate::error::CcusageError>;
        async fn put(&self, url: &str, data: &Value) -> Result<Value, crate::error::CcusageError>;
        async fn delete(&self, url: &str) -> Result<Value, crate::error::CcusageError>;
    }
}

// 模拟文件系统
pub trait MockFileSystem {
    fn read_to_string(&self, path: &PathBuf) -> Result<String, crate::error::CcusageError>;
    fn write(&self, path: &PathBuf, content: &str) -> Result<(), crate::error::CcusageError>;
    fn exists(&self, path: &PathBuf) -> bool;
    fn create_dir_all(&self, path: &PathBuf) -> Result<(), crate::error::CcusageError>;
    fn remove_file(&self, path: &PathBuf) -> Result<(), crate::error::CcusageError>;
    fn metadata(&self, path: &PathBuf) -> Result<std::fs::Metadata, crate::error::CcusageError>;
}

mock! {
    pub FileSystem {}

    impl MockFileSystem for FileSystem {
        fn read_to_string(&self, path: &PathBuf) -> Result<String, crate::error::CcusageError>;
        fn write(&self, path: &PathBuf, content: &str) -> Result<(), crate::error::CcusageError>;
        fn exists(&self, path: &PathBuf) -> bool;
        fn create_dir_all(&self, path: &PathBuf) -> Result<(), crate::error::CcusageError>;
        fn remove_file(&self, path: &PathBuf) -> Result<(), crate::error::CcusageError>;
        fn metadata(&self, path: &PathBuf) -> Result<std::fs::Metadata, crate::error::CcusageError>;
    }
}

// 模拟环境变量
pub trait MockEnvironment {
    fn get_var(&self, key: &str) -> Option<String>;
    fn set_var(&mut self, key: &str, value: &str);
    fn remove_var(&mut self, key: &str);
    fn vars(&self) -> HashMap<String, String>;
}

mock! {
    pub Environment {}

    impl MockEnvironment for Environment {
        fn get_var(&self, key: &str) -> Option<String>;
        fn set_var(&mut self, key: &str, value: &str);
        fn remove_var(&mut self, key: &str);
        fn vars(&self) -> HashMap<String, String>;
    }
}

// 模拟日志记录器
pub trait MockLogger {
    fn info(&self, message: &str);
    fn warn(&self, message: &str);
    fn error(&self, message: &str);
    fn debug(&self, message: &str);
    fn trace(&self, message: &str);
}

mock! {
    pub Logger {}

    impl MockLogger for Logger {
        fn info(&self, message: &str);
        fn warn(&self, message: &str);
        fn error(&self, message: &str);
        fn debug(&self, message: &str);
        fn trace(&self, message: &str);
    }
}

// 模拟时间服务
pub trait MockTimeService {
    fn now(&self) -> DateTime<Utc>;
    fn sleep(&self, duration: std::time::Duration);
}

mock! {
    pub TimeService {}

    impl MockTimeService for TimeService {
        fn now(&self) -> DateTime<Utc>;
        fn sleep(&self, duration: std::time::Duration);
    }
}

// 模拟服务工厂
pub struct MockServiceFactory {
    pub data_loader: MockDataLoader,
    pub config_manager: MockConfigManager,
    pub output_formatter: MockOutputFormatter,
    pub http_client: MockHttpClient,
    pub file_system: MockFileSystem,
    pub environment: MockEnvironment,
    pub logger: MockLogger,
    pub time_service: MockTimeService,
}

impl MockServiceFactory {
    pub fn new() -> Self {
        Self {
            data_loader: MockDataLoader::new(),
            config_manager: MockConfigManager::new(),
            output_formatter: MockOutputFormatter::new(),
            http_client: MockHttpClient::new(),
            file_system: MockFileSystem::new(),
            environment: MockEnvironment::new(),
            logger: MockLogger::new(),
            time_service: MockTimeService::new(),
        }
    }

    pub fn setup_success_responses(&mut self) {
        // 设置默认的成功响应
        let test_records = crate::common::test_data::predefined::small_dataset();
        let usage_records: Vec<crate::data::models::UsageRecord> = test_records
            .into_iter()
            .map(|r| serde_json::from_value(r).unwrap())
            .collect();

        self.data_loader
            .expect_load_usage_data()
            .returning(move || Ok(usage_records.clone()));

        self.config_manager
            .expect_get_config()
            .returning(|| crate::config::Config::default());

        self.config_manager
            .expect_get_data_source_path()
            .returning(|_| Some("./data/test.json".to_string()));

        self.file_system
            .expect_exists()
            .returning(|_| true);

        self.http_client
            .expect_get()
            .returning(|_| Ok(json!({"status": "ok"})));
    }

    pub fn setup_error_responses(&mut self) {
        // 设置默认的错误响应
        self.data_loader
            .expect_load_usage_data()
            .returning(|| Err(crate::error::CcusageError::DataLoading("Mock error".to_string())));

        self.config_manager
            .expect_get_config()
            .returning(|| Err(crate::error::CcusageError::Config("Mock config error".to_string())));

        self.file_system
            .expect_exists()
            .returning(|_| false);
    }
}

impl Default for MockServiceFactory {
    fn default() -> Self {
        Self::new()
    }
}

// 测试辅助函数
pub mod helpers {
    use super::*;

    /// 创建配置好的模拟服务工厂
    pub fn create_mock_factory() -> MockServiceFactory {
        let mut factory = MockServiceFactory::new();
        factory.setup_success_responses();
        factory
    }

    /// 创建错误模拟服务工厂
    pub fn create_error_mock_factory() -> MockServiceFactory {
        let mut factory = MockServiceFactory::new();
        factory.setup_error_responses();
        factory
    }

    /// 创建测试用的使用记录
    pub fn create_test_usage_records(count: usize) -> Vec<crate::data::models::UsageRecord> {
        let mut generator = crate::common::test_data::TestDataGenerator::new();
        let json_records = generator.generate_usage_records(count);
        
        json_records
            .into_iter()
            .map(|r| serde_json::from_value(r).unwrap())
            .collect()
    }

    /// 创建测试配置
    pub fn create_test_config() -> crate::config::Config {
        let mut generator = crate::common::test_data::TestDataGenerator::new();
        let json_config = generator.generate_config();
        serde_json::from_value(json_config).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_service_factory_creation() {
        let factory = MockServiceFactory::new();
        // 验证所有模拟服务都被创建
        // 这里只是验证编译通过，实际使用时需要设置期望
    }

    #[tokio::test]
    async fn test_mock_data_loader() {
        let mut mock_loader = MockDataLoader::new();
        
        let test_records = create_test_usage_records(5);
        mock_loader.expect_load_usage_data()
            .returning(move || Ok(test_records.clone()));

        let result = mock_loader.load_usage_data().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 5);
    }

    #[test]
    fn test_mock_environment() {
        let mut mock_env = MockEnvironment::new();
        
        mock_env.expect_get_var()
            .returning(|key| {
                match key {
                    "TEST_VAR" => Some("test_value".to_string()),
                    _ => None,
                }
            });

        mock_env.expect_set_var()
            .returning(|_, _| ());

        mock_env.expect_remove_var()
            .returning(|_| ());

        mock_env.expect_vars()
            .returning(|| HashMap::new());

        // 测试环境变量操作
        assert_eq!(mock_env.get_var("TEST_VAR"), Some("test_value".to_string()));
        assert_eq!(mock_env.get_var("NON_EXISTENT"), None);
        
        mock_env.set_var("NEW_VAR", "new_value");
        mock_env.remove_var("OLD_VAR");
        
        let vars = mock_env.vars();
        assert!(vars.is_empty());
    }

    #[test]
    fn test_mock_file_system() {
        let mut mock_fs = MockFileSystem::new();
        
        mock_fs.expect_exists()
            .returning(|path| path.ends_with("exists.txt"));

        mock_fs.expect_read_to_string()
            .returning(|_| Ok("file content".to_string()));

        mock_fs.expect_write()
            .returning(|_, _| Ok(()));

        mock_fs.expect_create_dir_all()
            .returning(|_| Ok(()));

        mock_fs.expect_remove_file()
            .returning(|_| Ok(()));

        // 测试文件系统操作
        let exists_path = PathBuf::from("exists.txt");
        let not_exists_path = PathBuf::from("not_exists.txt");
        
        assert!(mock_fs.exists(&exists_path));
        assert!(!mock_fs.exists(&not_exists_path));
        
        let content = mock_fs.read_to_string(&exists_path).unwrap();
        assert_eq!(content, "file content");
        
        mock_fs.write(&PathBuf::from("test.txt"), "new content").unwrap();
        mock_fs.create_dir_all(&PathBuf::from("test_dir")).unwrap();
        mock_fs.remove_file(&PathBuf::from("old.txt")).unwrap();
    }

    #[test]
    fn test_helpers() {
        let records = create_test_usage_records(3);
        assert_eq!(records.len(), 3);
        
        let config = create_test_config();
        // 验证配置创建成功
        drop(config); // 避免未使用变量警告
    }
}