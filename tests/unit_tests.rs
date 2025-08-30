//! 单元测试集合
//! 
//! 包含所有核心模块的单元测试，确保每个模块的功能正确性

mod common;

use common::*;
use ccusage_rs::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_error_module() {
        use crate::error::*;
        
        // 测试错误创建
        let config_err = config_error!("test config error");
        assert!(matches!(config_err, CcusageError::Config(_)));
        
        let data_err = data_error!("test data error");
        assert!(matches!(data_err, CcusageError::DataLoading(_)));
        
        let validation_err = validation_error!("test validation error");
        assert!(matches!(validation_err, CcusageError::Validation(_)));
        
        let app_err = app_error!("test app error");
        assert!(matches!(app_err, CcusageError::Application(_)));
    }

    #[tokio::test]
    async fn test_data_models() {
        use crate::data::models::*;
        use chrono::Utc;
        
        // 测试UsageRecord创建
        let record = UsageRecord::new(
            Utc::now(),
            "claude-3-sonnet".to_string(),
            1000,
            500,
            0.015,
        );
        
        record.assert_valid_usage_record();
        assert_eq!(record.total_tokens(), 1500);
        assert!(record.cost_per_token() > 0.0);
        
        // 测试Session创建
        let mut session = Session::new(
            "test_session".to_string(),
            Utc::now(),
            Some("test_user".to_string()),
        );
        
        session.add_record(&record);
        session.calculate_duration();
        
        session.assert_valid_session();
        session.assert_has_records();
        session.assert_duration_calculated();
        
        // 测试PricingInfo
        let pricing = PricingInfo {
            model: "claude-3-sonnet".to_string(),
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.015,
            currency: "USD".to_string(),
            effective_date: Utc::now(),
            is_active: true,
        };
        
        let cost = pricing.calculate_cost(1000, 500);
        assert_approx_eq!(cost, 0.0105, 0.0001);
        
        // 测试BudgetInfo
        let budget = BudgetInfo::new(100.0, "USD".to_string());
        assert!(!budget.is_warning_exceeded(70.0));
        assert!(budget.is_warning_exceeded(85.0));
        assert!(!budget.is_alert_exceeded(90.0));
        assert!(budget.is_alert_exceeded(98.0));
    }

    #[tokio::test]
    async fn test_cli_parsing() {
        use crate::cli::*;
        use std::path::PathBuf;
        
        // 测试基本CLI解析
        let app = App::try_parse_from(&["ccusage-rs", "--help"]);
        assert!(app.is_ok());
        
        // 测试analyze命令
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "analyze",
            "--analysis-type", "cost",
            "--date-range", "2024-01-01..2024-01-31",
            "--model", "claude-3-sonnet",
            "--detailed"
        ]);
        assert!(app.is_ok());
        
        // 测试daily命令
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "daily",
            "--date", "2024-01-01",
            "--compare"
        ]);
        assert!(app.is_ok());
        
        // 测试weekly命令
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "weekly",
            "--week-start", "2024-01-01",
            "--weeks", "4"
        ]);
        assert!(app.is_ok());
        
        // 测试monthly命令
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "monthly",
            "--year", "2024",
            "--month", "1",
            "--compare"
        ]);
        assert!(app.is_ok());
        
        // 测试budget命令
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "budget",
            "set",
            "--limit", "100.0",
            "--currency", "USD",
            "--warning", "80",
            "--alert", "95"
        ]);
        assert!(app.is_ok());
        
        // 测试config命令
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "config",
            "show"
        ]);
        assert!(app.is_ok());
        
        // 测试data命令
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "data",
            "load",
            "--source-type", "json",
            "--source", "./data/test.json"
        ]);
        assert!(app.is_ok());
        
        // 测试export命令
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "export",
            "--format", "json",
            "--start-date", "2024-01-01",
            "--end-date", "2024-01-31",
            "--output", "./export.json"
        ]);
        assert!(app.is_ok());
        
        // 测试insights命令
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "insights",
            "--count", "10",
            "--insight-type", "cost",
            "--insight-type", "usage"
        ]);
        assert!(app.is_ok());
        
        // 测试stats命令
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "stats",
            "--stats-type", "basic",
            "--group-by", "model"
        ]);
        assert!(app.is_ok());
    }

    #[tokio::test]
    async fn test_utils_functions() {
        use crate::utils::*;
        
        // 测试系统信息获取
        let info = get_system_info();
        assert!(info.contains_key("os"));
        assert!(info.contains_key("arch"));
        assert!(info.contains_key("version"));
        
        // 测试日期解析
        let date_str = "2024-01-01";
        let parsed_date = parse_date_flexible(date_str).unwrap();
        assert_eq!(parsed_date.date_naive().year(), 2024);
        assert_eq!(parsed_date.date_naive().month(), 1);
        assert_eq!(parsed_date.date_naive().day(), 1);
        
        // 测试文件大小格式化
        let size = format_file_size(1024);
        assert!(size.contains("KB"));
        
        let size = format_file_size(1024 * 1024);
        assert!(size.contains("MB"));
        
        let size = format_file_size(1024 * 1024 * 1024);
        assert!(size.contains("GB"));
        
        // 测试货币格式化
        let currency = format_currency(1234.56789, "USD", 2);
        assert!(currency.contains("$"));
        assert!(currency.contains("1,234.57"));
        
        // 测试百分比格式化
        let percentage = format_percentage(0.85432, 1);
        assert!(percentage.contains("85.4%"));
    }

    #[tokio::test]
    async fn test_config_management() {
        use crate::config::*;
        use tempfile::NamedTempFile;
        use std::io::Write;
        
        // 创建临时配置文件
        let mut temp_file = NamedTempFile::new().unwrap();
        let config_content = r#"
[app]
verbose = true
log_level = "debug"

[data]
default_source = "json"
cache_enabled = true
cache_ttl = 3600

[output]
default_format = "table"
decimal_places = 6

[budget]
monthly_limit = 100.0
currency = "USD"
warning_threshold = 80.0
alert_threshold = 95.0
enable_alerts = true
"#;
        temp_file.write_all(config_content.as_bytes()).unwrap();
        
        // 测试配置加载
        let config_manager = ConfigManager::new_with_config(temp_file.path()).unwrap();
        let config = config_manager.get_config();
        
        assert!(config.app.verbose);
        assert_eq!(config.app.log_level, "debug");
        assert_eq!(config.data.default_source, "json");
        assert!(config.data.cache_enabled);
        assert_eq!(config.output.default_format, "table");
        
        // 测试预算设置
        let budget = config_manager.get_budget().unwrap();
        assert_approx_eq!(budget.monthly_limit, 100.0, 0.01);
        assert_eq!(budget.currency, "USD");
        
        // 测试配置导出
        let exported_config = config_manager.export_config().unwrap();
        assert!(exported_config.contains("verbose = true"));
        
        // 测试配置重置
        let mut config_manager = config_manager;
        config_manager.reset_to_defaults().unwrap();
        let default_config = config_manager.get_config();
        assert!(!default_config.app.verbose); // 默认应该是false
    }

    #[tokio::test]
    async fn test_data_loading() {
        use crate::data::*;
        use crate::common::test_data::*;
        
        // 创建测试数据
        let mut generator = TestDataGenerator::new();
        let test_records = generator.generate_usage_records(10);
        let test_data_file = create_test_data_file(&json!(test_records), "json");
        
        // 测试数据加载
        let data_loader = DataLoader::with_source(
            DataSourceType::Json,
            test_data_file.to_string_lossy().to_string(),
        );
        
        let records = data_loader.load_usage_data().await.unwrap();
        assert_eq!(records.len(), 10);
        
        // 验证记录内容
        for record in records {
            record.assert_valid_usage_record();
            assert_positive!(record.input_tokens);
            assert_non_negative!(record.output_tokens);
            assert_positive!(record.cost);
        }
        
        // 清理测试文件
        cleanup_test_files(&[test_data_file]);
    }

    #[tokio::test]
    async fn test_analysis_modules() {
        use crate::analysis::*;
        use crate::common::test_data::*;
        use chrono::Utc;
        
        // 创建测试数据
        let mut generator = TestDataGenerator::new();
        let test_records = generator.generate_usage_records(50);
        let usage_records: Vec<UsageRecord> = test_records
            .into_iter()
            .map(|r| serde_json::from_value(r).unwrap())
            .collect();
        
        // 测试成本计算
        let calculator = CostCalculator::default();
        let total_cost = calculator.calculate_total_cost(&usage_records).unwrap();
        assert_positive!(total_cost);
        
        let breakdown = calculator.calculate_detailed_breakdown(&usage_records).unwrap();
        assert!(breakdown.total_cost > 0.0);
        assert!(!breakdown.model_breakdown.is_empty());
        
        // 测试统计计算
        let stats = StatisticsCalculator::calculate_usage_stats(&usage_records);
        assert_positive!(stats.total_requests);
        assert_positive!(stats.total_tokens);
        assert_positive!(stats.total_cost);
        assert!(!stats.model_usage.is_empty());
        
        // 测试趋势分析
        let analyzer = TrendAnalyzer::default();
        let trends = analyzer.analyze_trends(&usage_records).unwrap();
        assert!(!trends.daily_costs.is_empty());
        assert!(!trends.daily_tokens.is_empty());
        
        // 测试洞见生成
        let mut engine = InsightsEngine::default();
        let insights = engine.generate_insights(&usage_records, None).unwrap();
        assert!(!insights.is_empty());
        
        // 验证洞见类型
        for insight in insights {
            assert!(!insight.title.is_empty());
            assert!(!insight.description.is_empty());
            assert!(matches!(insight.insight_type, crate::analysis::insights::InsightType::Cost | crate::analysis::insights::InsightType::Usage | crate::analysis::insights::InsightType::Trends));
        }
    }

    #[tokio::test]
    async fn test_output_formatting() {
        use crate::output::*;
        use crate::common::test_data::*;
        use tempfile::NamedTempFile;
        
        // 创建测试数据
        let mut generator = TestDataGenerator::new();
        let test_records = generator.generate_usage_records(10);
        let usage_records: Vec<UsageRecord> = test_records
            .into_iter()
            .map(|r| serde_json::from_value(r).unwrap())
            .collect();
        
        // 测试统计计算
        let stats = crate::analysis::statistics::StatisticsCalculator::calculate_usage_stats(&usage_records);
        
        // 测试表格格式输出
        let formatter = OutputFormatter::new(OutputFormat::Table);
        let mut temp_file = NamedTempFile::new().unwrap();
        
        let result = formatter.output_usage_stats(&stats, Some(temp_file.path().to_str().unwrap()));
        assert!(result.is_ok());
        
        // 验证输出文件
        let output_content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(output_content.contains("Total Requests"));
        assert!(output_content.contains("Total Tokens"));
        assert!(output_content.contains("Total Cost"));
        
        // 测试JSON格式输出
        let formatter = OutputFormatter::new(OutputFormat::Json);
        let mut temp_file = NamedTempFile::new().unwrap();
        
        let result = formatter.output_usage_stats(&stats, Some(temp_file.path().to_str().unwrap()));
        assert!(result.is_ok());
        
        // 验证JSON输出
        let output_content = std::fs::read_to_string(temp_file.path()).unwrap();
        let json_value: serde_json::Value = serde_json::from_str(&output_content).unwrap();
        assert!(json_value.is_object());
        assert!(json_value.get("total_requests").is_some());
        assert!(json_value.get("total_tokens").is_some());
        assert!(json_value.get("total_cost").is_some());
        
        // 测试CSV格式输出
        let formatter = OutputFormatter::new(OutputFormat::Csv);
        let mut temp_file = NamedTempFile::new().unwrap();
        
        let result = formatter.output_usage_stats(&stats, Some(temp_file.path().to_str().unwrap()));
        assert!(result.is_ok());
        
        // 验证CSV输出
        let output_content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(output_content.contains("total_requests"));
        assert!(output_content.contains("total_tokens"));
        assert!(output_content.contains("total_cost"));
    }

    #[tokio::test]
    async fn test_parsing_module() {
        use crate::data::parser::*;
        use chrono::Utc;
        
        // 测试日期范围解析
        let date_range = "2024-01-01..2024-01-31";
        let (start, end) = parse_date_range(date_range).unwrap();
        
        assert_eq!(start.date_naive().year(), 2024);
        assert_eq!(start.date_naive().month(), 1);
        assert_eq!(start.date_naive().day(), 1);
        
        assert_eq!(end.date_naive().year(), 2024);
        assert_eq!(end.date_naive().month(), 1);
        assert_eq!(end.date_naive().day(), 31);
        
        // 测试无效日期范围
        let invalid_date_range = "invalid-date-range";
        let result = parse_date_range(invalid_date_range);
        assert!(result.is_err());
        
        // 测试JSON解析
        let json_data = r#"
        [
            {
                "id": "test-1",
                "timestamp": "2024-01-01T12:00:00Z",
                "model": "claude-3-sonnet",
                "input_tokens": 1000,
                "output_tokens": 500,
                "cost": 0.015
            },
            {
                "id": "test-2",
                "timestamp": "2024-01-01T13:00:00Z",
                "model": "claude-3-opus",
                "input_tokens": 2000,
                "output_tokens": 1000,
                "cost": 0.045
            }
        ]
        "#;
        
        let records = parse_json_data(json_data).unwrap();
        assert_eq!(records.len(), 2);
        
        for record in records {
            record.assert_valid_usage_record();
        }
        
        // 测试CSV解析
        let csv_data = r#"
id,timestamp,model,input_tokens,output_tokens,cost
test-1,2024-01-01T12:00:00Z,claude-3-sonnet,1000,500,0.015
test-2,2024-01-01T13:00:00Z,claude-3-opus,2000,1000,0.045
"#;
        
        let records = parse_csv_data(csv_data).unwrap();
        assert_eq!(records.len(), 2);
        
        for record in records {
            record.assert_valid_usage_record();
        }
    }

    #[tokio::test]
    async fn test_app_integration() {
        use crate::cli::*;
        use crate::common::test_data::*;
        use tempfile::NamedTempFile;
        use std::io::Write;
        
        // 创建测试数据文件
        let mut generator = TestDataGenerator::new();
        let test_records = generator.generate_usage_records(20);
        let mut temp_file = NamedTempFile::new().unwrap();
        
        let json_data = serde_json::to_string_pretty(&json!(test_records)).unwrap();
        temp_file.write_all(json_data.as_bytes()).unwrap();
        
        // 测试应用初始化
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "--data", temp_file.path().to_str().unwrap(),
            "--format", "json",
            "analyze",
            "--analysis-type", "cost"
        ]).unwrap();
        
        // 测试配置管理器创建
        let config_manager = ConfigManager::new().unwrap();
        let config = config_manager.get_config();
        
        // 验证配置
        assert!(!config.app.verbose); // 默认值
        assert_eq!(config.data.default_source, "json");
        
        // 测试应用运行（这里只是验证结构，实际运行需要更多设置）
        // 注意：在实际测试中，你可能需要设置更多的模拟环境
        assert!(app.data.is_some());
        assert!(app.format.is_some());
    }

    #[tokio::test]
    async fn test_logging_module() {
        use crate::logging::*;
        
        // 测试日志初始化
        let result = init_logging(false);
        assert!(result.is_ok());
        
        let result = init_logging(true);
        assert!(result.is_ok());
        
        // 测试日志记录
        log::info!("Test info message");
        log::warn!("Test warning message");
        log::error!("Test error message");
        log::debug!("Test debug message");
        log::trace!("Test trace message");
    }

    #[tokio::test]
    async fn test_memory_safety() {
        use crate::data::models::*;
        use chrono::Utc;
        
        // 测试大量数据的内存使用
        let mut records = Vec::new();
        
        for i in 0..1000 {
            let record = UsageRecord::new(
                Utc::now(),
                format!("model-{}", i % 5),
                (i * 10) as u32,
                (i * 5) as u32,
                (i * 0.001) as f64,
            );
            records.push(record);
        }
        
        // 验证所有记录都有效
        for record in records {
            record.assert_valid_usage_record();
        }
        
        // 测试清理
        drop(records);
        
        // 测试会话的内存管理
        let mut session = Session::new(
            "test_session".to_string(),
            Utc::now(),
            Some("test_user".to_string()),
        );
        
        for i in 0..100 {
            let record = UsageRecord::new(
                Utc::now(),
                format!("model-{}", i % 3),
                (i * 10) as u32,
                (i * 5) as u32,
                (i * 0.001) as f64,
            );
            session.add_record(&record);
        }
        
        session.calculate_duration();
        session.assert_valid_session();
        
        // 测试清理
        drop(session);
    }

    #[tokio::test]
    async fn test_error_handling() {
        use crate::error::*;
        use std::io;
        
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
        
        // 测试配置错误转换
        let config_error = config::ConfigError::NotFound("config not found".to_string());
        let ccusage_error: CcusageError = config_error.into();
        assert!(matches!(ccusage_error, CcusageError::ConfigParse(_)));
        
        // 测试错误链
        let result: Result<i32> = Err(CcusageError::DataLoading("Data loading failed".to_string()));
        let chained_result = crate::error::chain_results(vec![result]);
        assert!(chained_result.is_err());
        
        // 测试错误上下文
        let result: std::result::Result<i32, &str> = Err("original error");
        let contextual_result = result.with_context(|| "additional context");
        assert!(contextual_result.is_err());
        
        if let Err(e) = contextual_result {
            assert!(e.to_string().contains("additional context"));
            assert!(e.to_string().contains("original error"));
        }
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        use crate::data::models::*;
        use crate::common::test_data::*;
        use tokio::task::JoinSet;
        use chrono::Utc;
        
        // 测试并发创建使用记录
        let mut join_set = JoinSet::new();
        
        for i in 0..10 {
            join_set.spawn(async move {
                let record = UsageRecord::new(
                    Utc::now(),
                    format!("model-{}", i),
                    (i * 100) as u32,
                    (i * 50) as u32,
                    (i * 0.01) as f64,
                );
                record.assert_valid_usage_record();
                record
            });
        }
        
        let mut records = Vec::new();
        while let Some(result) = join_set.join_next().await {
            records.push(result.unwrap());
        }
        
        assert_eq!(records.len(), 10);
        
        // 测试并发会话操作
        let mut join_set = JoinSet::new();
        
        for i in 0..5 {
            let record = records[i].clone();
            join_set.spawn(async move {
                let mut session = Session::new(
                    format!("session-{}", i),
                    Utc::now(),
                    Some(format!("user-{}", i)),
                );
                session.add_record(&record);
                session.calculate_duration();
                session.assert_valid_session();
                session
            });
        }
        
        let mut sessions = Vec::new();
        while let Some(result) = join_set.join_next().await {
            sessions.push(result.unwrap());
        }
        
        assert_eq!(sessions.len(), 5);
    }
}