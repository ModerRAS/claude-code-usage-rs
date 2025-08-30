//! CLI测试
//! 
//! 测试命令行接口和参数解析功能

use ccusage_rs::cli::*;
use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::assertions::*;

    #[test]
    fn test_app_creation() {
        let app = App::try_parse_from(&["ccusage-rs", "--help"]);
        assert!(app.is_ok());
    }

    #[test]
    fn test_verbose_flag() {
        let app = App::try_parse_from(&["ccusage-rs", "--verbose", "analyze"]).unwrap();
        assert!(app.verbose);

        let app = App::try_parse_from(&["ccusage-rs", "-v", "analyze"]).unwrap();
        assert!(app.verbose);

        let app = App::try_parse_from(&["ccusage-rs", "analyze"]).unwrap();
        assert!(!app.verbose);
    }

    #[test]
    fn test_config_flag() {
        let config_path = "/path/to/config.toml";
        let app = App::try_parse_from(&["ccusage-rs", "--config", config_path, "analyze"]).unwrap();
        assert_eq!(app.config, Some(PathBuf::from(config_path)));

        let app = App::try_parse_from(&["ccusage-rs", "-c", config_path, "analyze"]).unwrap();
        assert_eq!(app.config, Some(PathBuf::from(config_path)));

        let app = App::try_parse_from(&["ccusage-rs", "analyze"]).unwrap();
        assert!(app.config.is_none());
    }

    #[test]
    fn test_data_flag() {
        let data_path = "/path/to/data.json";
        let app = App::try_parse_from(&["ccusage-rs", "--data", data_path, "analyze"]).unwrap();
        assert_eq!(app.data, Some(PathBuf::from(data_path)));

        let app = App::try_parse_from(&["ccusage-rs", "-d", data_path, "analyze"]).unwrap();
        assert_eq!(app.data, Some(PathBuf::from(data_path)));

        let app = App::try_parse_from(&["ccusage-rs", "analyze"]).unwrap();
        assert!(app.data.is_none());
    }

    #[test]
    fn test_format_flag() {
        let app = App::try_parse_from(&["ccusage-rs", "--format", "json", "analyze"]).unwrap();
        assert_eq!(app.format, Some(OutputFormat::Json));

        let app = App::try_parse_from(&["ccusage-rs", "-f", "csv", "analyze"]).unwrap();
        assert_eq!(app.format, Some(OutputFormat::Csv));

        let app = App::try_parse_from(&["ccusage-rs", "analyze"]).unwrap();
        assert!(app.format.is_none());
    }

    #[test]
    fn test_output_flag() {
        let output_path = "/path/to/output.json";
        let app = App::try_parse_from(&["ccusage-rs", "--output", output_path, "analyze"]).unwrap();
        assert_eq!(app.output, Some(PathBuf::from(output_path)));

        let app = App::try_parse_from(&["ccusage-rs", "-o", output_path, "analyze"]).unwrap();
        assert_eq!(app.output, Some(PathBuf::from(output_path)));

        let app = App::try_parse_from(&["ccusage-rs", "analyze"]).unwrap();
        assert!(app.output.is_none());
    }

    #[test]
    fn test_analyze_command() {
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "analyze",
            "--analysis-type", "cost",
            "--date-range", "2024-01-01..2024-01-31",
            "--model", "claude-3-sonnet",
            "--model", "claude-3-opus",
            "--detailed"
        ]).unwrap();

        if let Commands::Analyze { analysis_type, date_range, model, detailed } = app.command {
            assert_eq!(analysis_type, Some(AnalysisType::Cost));
            assert_eq!(date_range, Some("2024-01-01..2024-01-31".to_string()));
            assert_eq!(model, Some(vec!["claude-3-sonnet".to_string(), "claude-3-opus".to_string()]));
            assert!(detailed);
        } else {
            panic!("Expected Analyze command");
        }
    }

    #[test]
    fn test_analyze_command_default_values() {
        let app = App::try_parse_from(&["ccusage-rs", "analyze"]).unwrap();

        if let Commands::Analyze { analysis_type, date_range, model, detailed } = app.command {
            assert!(analysis_type.is_none());
            assert!(date_range.is_none());
            assert!(model.is_none());
            assert!(!detailed);
        } else {
            panic!("Expected Analyze command");
        }
    }

    #[test]
    fn test_daily_command() {
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "daily",
            "--date", "2024-01-01",
            "--compare"
        ]).unwrap();

        if let Commands::Daily { date, compare } = app.command {
            assert_eq!(date, Some("2024-01-01".to_string()));
            assert!(compare);
        } else {
            panic!("Expected Daily command");
        }
    }

    #[test]
    fn test_daily_command_default_values() {
        let app = App::try_parse_from(&["ccusage-rs", "daily"]).unwrap();

        if let Commands::Daily { date, compare } = app.command {
            assert!(date.is_none());
            assert!(!compare);
        } else {
            panic!("Expected Daily command");
        }
    }

    #[test]
    fn test_weekly_command() {
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "weekly",
            "--week-start", "2024-01-01",
            "--weeks", "4"
        ]).unwrap();

        if let Commands::Weekly { week_start, weeks } = app.command {
            assert_eq!(week_start, Some("2024-01-01".to_string()));
            assert_eq!(weeks, 4);
        } else {
            panic!("Expected Weekly command");
        }
    }

    #[test]
    fn test_weekly_command_default_values() {
        let app = App::try_parse_from(&["ccusage-rs", "weekly"]).unwrap();

        if let Commands::Weekly { week_start, weeks } = app.command {
            assert!(week_start.is_none());
            assert_eq!(weeks, 1);
        } else {
            panic!("Expected Weekly command");
        }
    }

    #[test]
    fn test_monthly_command() {
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "monthly",
            "--year", "2024",
            "--month", "1",
            "--compare"
        ]).unwrap();

        if let Commands::Monthly { year, month, compare } = app.command {
            assert_eq!(year, Some(2024));
            assert_eq!(month, Some(1));
            assert!(compare);
        } else {
            panic!("Expected Monthly command");
        }
    }

    #[test]
    fn test_monthly_command_default_values() {
        let app = App::try_parse_from(&["ccusage-rs", "monthly"]).unwrap();

        if let Commands::Monthly { year, month, compare } = app.command {
            assert!(year.is_none());
            assert!(month.is_none());
            assert!(!compare);
        } else {
            panic!("Expected Monthly command");
        }
    }

    #[test]
    fn test_session_command_list() {
        let app = App::try_parse_from(&["ccusage-rs", "session", "--list"]).unwrap();

        if let Commands::Session { session_id, list } = app.command {
            assert!(session_id.is_none());
            assert!(list);
        } else {
            panic!("Expected Session command");
        }
    }

    #[test]
    fn test_session_command_with_id() {
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "session",
            "--session-id", "test-session-123"
        ]).unwrap();

        if let Commands::Session { session_id, list } = app.command {
            assert_eq!(session_id, Some("test-session-123".to_string()));
            assert!(!list);
        } else {
            panic!("Expected Session command");
        }
    }

    #[test]
    fn test_budget_command_status() {
        let app = App::try_parse_from(&["ccusage-rs", "budget", "status"]).unwrap();

        if let Commands::Budget { action } = app.command {
            assert!(matches!(action, BudgetAction::Status));
        } else {
            panic!("Expected Budget command");
        }
    }

    #[test]
    fn test_budget_command_set() {
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "budget",
            "set",
            "--limit", "100.0",
            "--currency", "USD",
            "--warning", "80",
            "--alert", "95"
        ]).unwrap();

        if let Commands::Budget { action } = app.command {
            if let BudgetAction::Set { limit, currency, warning, alert } = action {
                assert_eq!(limit, 100.0);
                assert_eq!(currency, "USD");
                assert_eq!(warning, 80.0);
                assert_eq!(alert, 95.0);
            } else {
                panic!("Expected BudgetAction::Set");
            }
        } else {
            panic!("Expected Budget command");
        }
    }

    #[test]
    fn test_budget_command_history() {
        let app = App::try_parse_from(&["ccusage-rs", "budget", "history"]).unwrap();

        if let Commands::Budget { action } = app.command {
            assert!(matches!(action, BudgetAction::History));
        } else {
            panic!("Expected Budget command");
        }
    }

    #[test]
    fn test_budget_command_clear() {
        let app = App::try_parse_from(&["ccusage-rs", "budget", "clear"]).unwrap();

        if let Commands::Budget { action } = app.command {
            assert!(matches!(action, BudgetAction::Clear));
        } else {
            panic!("Expected Budget command");
        }
    }

    #[test]
    fn test_config_command_show() {
        let app = App::try_parse_from(&["ccusage-rs", "config", "show"]).unwrap();

        if let Commands::Config { action } = app.command {
            assert!(matches!(action, ConfigAction::Show));
        } else {
            panic!("Expected Config command");
        }
    }

    #[test]
    fn test_config_command_set() {
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "config",
            "set",
            "--key", "verbose",
            "--value", "true"
        ]).unwrap();

        if let Commands::Config { action } = app.command {
            if let ConfigAction::Set { key, value } = action {
                assert_eq!(key, "verbose");
                assert_eq!(value, "true");
            } else {
                panic!("Expected ConfigAction::Set");
            }
        } else {
            panic!("Expected Config command");
        }
    }

    #[test]
    fn test_config_command_reset() {
        let app = App::try_parse_from(&["ccusage-rs", "config", "reset"]).unwrap();

        if let Commands::Config { action } = app.command {
            assert!(matches!(action, ConfigAction::Reset));
        } else {
            panic!("Expected Config command");
        }
    }

    #[test]
    fn test_config_command_export() {
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "config",
            "export",
            "--output", "/path/to/export.toml"
        ]).unwrap();

        if let Commands::Config { action } = app.command {
            if let ConfigAction::Export { output } = action {
                assert_eq!(output, PathBuf::from("/path/to/export.toml"));
            } else {
                panic!("Expected ConfigAction::Export");
            }
        } else {
            panic!("Expected Config command");
        }
    }

    #[test]
    fn test_config_command_import() {
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "config",
            "import",
            "--input", "/path/to/import.toml"
        ]).unwrap();

        if let Commands::Config { action } = app.command {
            if let ConfigAction::Import { input } = action {
                assert_eq!(input, PathBuf::from("/path/to/import.toml"));
            } else {
                panic!("Expected ConfigAction::Import");
            }
        } else {
            panic!("Expected Config command");
        }
    }

    #[test]
    fn test_data_command_load() {
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "data",
            "load",
            "--source-type", "json",
            "--source", "/path/to/data.json"
        ]).unwrap();

        if let Commands::Data { action } = app.command {
            if let DataAction::Load { source_type, source } = action {
                assert_eq!(source_type, DataSourceType::Json);
                assert_eq!(source, PathBuf::from("/path/to/data.json"));
            } else {
                panic!("Expected DataAction::Load");
            }
        } else {
            panic!("Expected Data command");
        }
    }

    #[test]
    fn test_data_command_validate() {
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "data",
            "validate",
            "--data-file", "/path/to/data.json"
        ]).unwrap();

        if let Commands::Data { action } = app.command {
            if let DataAction::Validate { data_file } = action {
                assert_eq!(data_file, PathBuf::from("/path/to/data.json"));
            } else {
                panic!("Expected DataAction::Validate");
            }
        } else {
            panic!("Expected Data command");
        }
    }

    #[test]
    fn test_data_command_info() {
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "data",
            "info",
            "--data-file", "/path/to/data.json"
        ]).unwrap();

        if let Commands::Data { action } = app.command {
            if let DataAction::Info { data_file } = action {
                assert_eq!(data_file, PathBuf::from("/path/to/data.json"));
            } else {
                panic!("Expected DataAction::Info");
            }
        } else {
            panic!("Expected Data command");
        }
    }

    #[test]
    fn test_data_command_clean() {
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "data",
            "clean",
            "--days", "30"
        ]).unwrap();

        if let Commands::Data { action } = app.command {
            if let DataAction::Clean { days } = action {
                assert_eq!(days, 30);
            } else {
                panic!("Expected DataAction::Clean");
            }
        } else {
            panic!("Expected Data command");
        }
    }

    #[test]
    fn test_export_command() {
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "export",
            "--format", "json",
            "--start-date", "2024-01-01",
            "--end-date", "2024-01-31",
            "--output", "/path/to/export.json"
        ]).unwrap();

        if let Commands::Export { format, start_date, end_date, output } = app.command {
            assert_eq!(format, ExportFormat::Json);
            assert_eq!(start_date, "2024-01-01");
            assert_eq!(end_date, "2024-01-31");
            assert_eq!(output, PathBuf::from("/path/to/export.json"));
        } else {
            panic!("Expected Export command");
        }
    }

    #[test]
    fn test_server_command() {
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "server",
            "--port", "8080",
            "--host", "0.0.0.0",
            "--auth"
        ]).unwrap();

        if let Commands::Server { port, host, auth } = app.command {
            assert_eq!(port, 8080);
            assert_eq!(host, "0.0.0.0");
            assert!(auth);
        } else {
            panic!("Expected Server command");
        }
    }

    #[test]
    fn test_server_command_default_values() {
        let app = App::try_parse_from(&["ccusage-rs", "server"]).unwrap();

        if let Commands::Server { port, host, auth } = app.command {
            assert_eq!(port, 8080);
            assert_eq!(host, "127.0.0.1");
            assert!(!auth);
        } else {
            panic!("Expected Server command");
        }
    }

    #[test]
    fn test_insights_command() {
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "insights",
            "--count", "20",
            "--insight-type", "cost",
            "--insight-type", "usage"
        ]).unwrap();

        if let Commands::Insights { count, insight_type } = app.command {
            assert_eq!(count, 20);
            assert_eq!(insight_type, Some(vec![
                InsightTypeFilter::Cost,
                InsightTypeFilter::Usage
            ]));
        } else {
            panic!("Expected Insights command");
        }
    }

    #[test]
    fn test_insights_command_default_values() {
        let app = App::try_parse_from(&["ccusage-rs", "insights"]).unwrap();

        if let Commands::Insights { count, insight_type } = app.command {
            assert_eq!(count, 10);
            assert!(insight_type.is_none());
        } else {
            panic!("Expected Insights command");
        }
    }

    #[test]
    fn test_stats_command() {
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "stats",
            "--stats-type", "detailed",
            "--group-by", "model"
        ]).unwrap();

        if let Commands::Stats { stats_type, group_by } = app.command {
            assert_eq!(stats_type, Some(StatsType::Detailed));
            assert_eq!(group_by, Some("model".to_string()));
        } else {
            panic!("Expected Stats command");
        }
    }

    #[test]
    fn test_stats_command_default_values() {
        let app = App::try_parse_from(&["ccusage-rs", "stats"]).unwrap();

        if let Commands::Stats { stats_type, group_by } = app.command {
            assert_eq!(stats_type, Some(StatsType::Basic));
            assert!(group_by.is_none());
        } else {
            panic!("Expected Stats command");
        }
    }

    #[test]
    fn test_invalid_output_format() {
        let app = App::try_parse_from(&["ccusage-rs", "--format", "invalid", "analyze"]);
        assert!(app.is_err());
    }

    #[test]
    fn test_invalid_analysis_type() {
        let app = App::try_parse_from(&["ccusage-rs", "analyze", "--analysis-type", "invalid"]);
        assert!(app.is_err());
    }

    #[test]
    fn test_invalid_export_format() {
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "export",
            "--format", "invalid",
            "--start-date", "2024-01-01",
            "--end-date", "2024-01-31",
            "--output", "/path/to/export.json"
        ]);
        assert!(app.is_err());
    }

    #[test]
    fn test_invalid_stats_type() {
        let app = App::try_parse_from(&["ccusage-rs", "stats", "--stats-type", "invalid"]);
        assert!(app.is_err());
    }

    #[test]
    fn test_missing_required_arguments() {
        // 测试缺少必需参数的情况
        let app = App::try_parse_from(&["ccusage-rs", "export"]);
        assert!(app.is_err());

        let app = App::try_parse_from(&["ccusage-rs", "config", "set"]);
        assert!(app.is_err());

        let app = App::try_parse_from(&["ccusage-rs", "config", "export"]);
        assert!(app.is_err());

        let app = App::try_parse_from(&["ccusage-rs", "config", "import"]);
        assert!(app.is_err());
    }

    #[test]
    fn test_multiple_global_flags() {
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "--verbose",
            "--config", "/path/to/config.toml",
            "--data", "/path/to/data.json",
            "--format", "json",
            "--output", "/path/to/output.json",
            "analyze"
        ]).unwrap();

        assert!(app.verbose);
        assert_eq!(app.config, Some(PathBuf::from("/path/to/config.toml")));
        assert_eq!(app.data, Some(PathBuf::from("/path/to/data.json")));
        assert_eq!(app.format, Some(OutputFormat::Json));
        assert_eq!(app.output, Some(PathBuf::from("/path/to/output.json")));
    }

    #[test]
    fn test_help_flag() {
        let app = App::try_parse_from(&["ccusage-rs", "--help"]);
        assert!(app.is_ok());

        let app = App::try_parse_from(&["ccusage-rs", "analyze", "--help"]);
        assert!(app.is_ok());

        let app = App::try_parse_from(&["ccusage-rs", "budget", "set", "--help"]);
        assert!(app.is_ok());
    }

    #[test]
    fn test_version_flag() {
        let app = App::try_parse_from(&["ccusage-rs", "--version"]);
        assert!(app.is_ok());
    }

    #[test]
    fn test_complex_command_line() {
        // 测试复杂的命令行组合
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "--verbose",
            "--config", "/path/to/config.toml",
            "--data", "/path/to/data.json",
            "--format", "json",
            "--output", "/path/to/output.json",
            "analyze",
            "--analysis-type", "comprehensive",
            "--date-range", "2024-01-01..2024-01-31",
            "--model", "claude-3-sonnet",
            "--model", "claude-3-opus",
            "--model", "claude-3-haiku",
            "--detailed"
        ]);

        assert!(app.is_ok());
    }

    #[test]
    fn test_edge_cases() {
        // 测试边界情况
        
        // 测试空字符串参数
        let app = App::try_parse_from(&["ccusage-rs", "analyze", "--date-range", ""]);
        assert!(app.is_err());

        // 测试负数参数
        let app = App::try_parse_from(&["ccusage-rs", "budget", "set", "--limit", "-100.0"]);
        assert!(app.is_err());

        // 测试零值参数
        let app = App::try_parse_from(&["ccusage-rs", "budget", "set", "--limit", "0.0"]);
        assert!(app.is_ok());

        // 测试极大值参数
        let app = App::try_parse_from(&["ccusage-rs", "insights", "--count", "999999"]);
        assert!(app.is_ok());
    }

    #[test]
    fn test_unicode_arguments() {
        // 测试Unicode参数
        let app = App::try_parse_from(&[
            "ccusage-rs",
            "analyze",
            "--model", "claude-3-sonnet-中文版",
            "--output", "/路径/输出.json"
        ]);

        assert!(app.is_ok());
    }

    #[test]
    fn test_subcommand_aliases() {
        // 测试子命令别名（如果支持）
        // 这里主要测试基本的子命令识别
        let commands = vec![
            "analyze", "daily", "weekly", "monthly", "session",
            "budget", "config", "data", "export", "server", "insights", "stats"
        ];

        for cmd in commands {
            let app = App::try_parse_from(&["ccusage-rs", cmd]);
            assert!(app.is_ok(), "Command '{}' should be valid", cmd);
        }
    }
}