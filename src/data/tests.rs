//! 数据模型测试
//! 
//! 测试所有数据结构和相关功能

use ccusage_rs::data::models::*;
use ccusage_rs::data::parser::*;
use ccusage_rs::data::loader::*;
use ccusage_rs::data::storage::*;
use chrono::{DateTime, Utc, NaiveDate, NaiveDateTime};
use serde_json::json;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::test_data::*;
    use crate::common::test_utils::*;
    use crate::common::assertions::*;

    #[test]
    fn test_usage_record_creation() {
        let timestamp = Utc::now();
        let record = UsageRecord::new(
            timestamp,
            "claude-3-sonnet".to_string(),
            1000,
            500,
            0.015,
        );

        assert!(!record.id.is_empty());
        assert_eq!(record.timestamp, timestamp);
        assert_eq!(record.model, "claude-3-sonnet");
        assert_eq!(record.input_tokens, 1000);
        assert_eq!(record.output_tokens, 500);
        assert_eq!(record.cost, 0.015);
        assert!(record.session_id.is_none());
        assert!(record.user_id.is_none());
        assert!(record.metadata.is_empty());
    }

    #[test]
    fn test_usage_record_total_tokens() {
        let record = UsageRecord::new(
            Utc::now(),
            "claude-3-sonnet".to_string(),
            1000,
            500,
            0.015,
        );

        assert_eq!(record.total_tokens(), 1500);
    }

    #[test]
    fn test_usage_record_cost_per_token() {
        let record = UsageRecord::new(
            Utc::now(),
            "claude-3-sonnet".to_string(),
            1000,
            500,
            0.015,
        );

        let cost_per_token = record.cost_per_token();
        assert_approx_eq!(cost_per_token, 0.00001, 0.000001);
    }

    #[test]
    fn test_usage_record_date_checks() {
        let timestamp = DateTime::from_naive_utc_and_offset(
            NaiveDateTime::new(
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
            ),
            Utc,
        );

        let record = UsageRecord::new(
            timestamp,
            "claude-3-sonnet".to_string(),
            1000,
            500,
            0.015,
        );

        assert!(record.is_on_date(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()));
        assert!(!record.is_on_date(NaiveDate::from_ymd_opt(2024, 1, 2).unwrap()));

        let start = timestamp - chrono::Duration::hours(1);
        let end = timestamp + chrono::Duration::hours(1);
        assert!(record.is_within_range(start, end));

        let far_future = timestamp + chrono::Duration::days(1);
        assert!(!record.is_within_range(start, far_future));
    }

    #[test]
    fn test_session_creation() {
        let start_time = Utc::now();
        let session = Session::new(
            "test_session".to_string(),
            start_time,
            Some("test_user".to_string()),
        );

        assert_eq!(session.id, "test_session");
        assert_eq!(session.start_time, start_time);
        assert!(session.end_time.is_none());
        assert_eq!(session.user_id, Some("test_user".to_string()));
        assert_eq!(session.total_cost, 0.0);
        assert_eq!(session.total_input_tokens, 0);
        assert_eq!(session.total_output_tokens, 0);
        assert_eq!(session.request_count, 0);
        assert!(session.duration_seconds.is_none());
        assert!(session.metadata.is_empty());
    }

    #[test]
    fn test_session_add_record() {
        let start_time = Utc::now();
        let mut session = Session::new(
            "test_session".to_string(),
            start_time,
            Some("test_user".to_string()),
        );

        let record1 = UsageRecord::new(
            start_time,
            "claude-3-sonnet".to_string(),
            1000,
            500,
            0.015,
        );

        let record2 = UsageRecord::new(
            start_time + chrono::Duration::minutes(30),
            "claude-3-opus".to_string(),
            2000,
            1000,
            0.045,
        );

        session.add_record(&record1);
        session.add_record(&record2);

        assert_eq!(session.request_count, 2);
        assert_eq!(session.total_cost, 0.06);
        assert_eq!(session.total_input_tokens, 3000);
        assert_eq!(session.total_output_tokens, 1500);
        assert_eq!(session.end_time, Some(record2.timestamp));
    }

    #[test]
    fn test_session_calculate_duration() {
        let start_time = Utc::now();
        let end_time = start_time + chrono::Duration::hours(2);
        let mut session = Session::new(
            "test_session".to_string(),
            start_time,
            Some("test_user".to_string()),
        );

        session.end_time = Some(end_time);
        session.calculate_duration();

        assert_eq!(session.duration_seconds, Some(7200));
    }

    #[test]
    fn test_session_averages() {
        let start_time = Utc::now();
        let mut session = Session::new(
            "test_session".to_string(),
            start_time,
            Some("test_user".to_string()),
        );

        let record = UsageRecord::new(
            start_time,
            "claude-3-sonnet".to_string(),
            1000,
            500,
            0.015,
        );

        session.add_record(&record);

        assert_approx_eq!(session.avg_cost_per_request(), 0.015, 0.0001);
        assert_approx_eq!(session.avg_tokens_per_request(), 1500.0, 0.1);
    }

    #[test]
    fn test_pricing_info() {
        let pricing = PricingInfo {
            model: "claude-3-sonnet".to_string(),
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.015,
            currency: "USD".to_string(),
            effective_date: Utc::now(),
            is_active: true,
        };

        let cost = pricing.calculate_cost(1000, 500);
        let expected_cost = (1000.0 / 1000.0) * 0.003 + (500.0 / 1000.0) * 0.015;
        assert_approx_eq!(cost, expected_cost, 0.0001);

        let test_date = Utc::now() + chrono::Duration::days(1);
        assert!(!pricing.is_valid_for_date(test_date));

        let past_date = Utc::now() - chrono::Duration::days(1);
        assert!(pricing.is_valid_for_date(past_date));
    }

    #[test]
    fn test_daily_summary() {
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let mut summary = DailySummary::new(date);

        assert_eq!(summary.date, date);
        assert_eq!(summary.total_cost, 0.0);
        assert_eq!(summary.total_input_tokens, 0);
        assert_eq!(summary.total_output_tokens, 0);
        assert_eq!(summary.request_count, 0);
        assert_eq!(summary.session_count, 0);
        assert!(summary.most_used_model.is_none());
        assert!(summary.peak_hour.is_none());
        assert_eq!(summary.avg_cost_per_request, 0.0);
        assert!(summary.model_breakdown.is_empty());

        let record = UsageRecord::new(
            DateTime::from_naive_utc_and_offset(
                NaiveDateTime::new(date, chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap()),
                Utc,
            ),
            "claude-3-sonnet".to_string(),
            1000,
            500,
            0.015,
        );

        summary.add_record(&record);

        assert_eq!(summary.total_cost, 0.015);
        assert_eq!(summary.total_input_tokens, 1000);
        assert_eq!(summary.total_output_tokens, 500);
        assert_eq!(summary.request_count, 1);
        assert_eq!(summary.most_used_model, Some("claude-3-sonnet".to_string()));
        assert_eq!(summary.model_breakdown.len(), 1);

        summary.calculate_avg_cost();
        assert_approx_eq!(summary.avg_cost_per_request, 0.015, 0.0001);
    }

    #[test]
    fn test_weekly_summary() {
        let week_start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let mut summary = WeeklySummary::new(week_start);

        assert_eq!(summary.week_start, week_start);
        assert_eq!(summary.week_end, week_start + chrono::Duration::days(6));
        assert_eq!(summary.total_cost, 0.0);
        assert_eq!(summary.total_input_tokens, 0);
        assert_eq!(summary.total_output_tokens, 0);
        assert_eq!(summary.request_count, 0);
        assert_eq!(summary.session_count, 0);
        assert!(summary.daily_breakdown.is_empty());
        assert_eq!(summary.avg_daily_cost, 0.0);
        assert!(summary.most_expensive_day.is_none());

        let daily_summary = DailySummary::new(week_start);
        summary.add_daily_summary(daily_summary);

        assert_eq!(summary.daily_breakdown.len(), 1);
        assert_eq!(summary.total_cost, 0.0);
        assert_eq!(summary.total_input_tokens, 0);
        assert_eq!(summary.total_output_tokens, 0);
        assert_eq!(summary.request_count, 0);
        assert_eq!(summary.session_count, 0);

        summary.calculate_avg_daily_cost();
        assert_eq!(summary.avg_daily_cost, 0.0);
        assert!(summary.most_expensive_day.is_none());
    }

    #[test]
    fn test_monthly_summary() {
        let mut summary = MonthlySummary::new(2024, 1);

        assert_eq!(summary.year, 2024);
        assert_eq!(summary.month, 1);
        assert_eq!(summary.total_cost, 0.0);
        assert_eq!(summary.total_input_tokens, 0);
        assert_eq!(summary.total_output_tokens, 0);
        assert_eq!(summary.request_count, 0);
        assert_eq!(summary.session_count, 0);
        assert!(summary.weekly_breakdown.is_empty());
        assert_eq!(summary.avg_weekly_cost, 0.0);
        assert!(summary.most_expensive_week.is_none());
        assert!(summary.budget.is_none());

        let budget = BudgetInfo::new(100.0, "USD".to_string());
        summary.budget = Some(budget);

        assert!(!summary.is_budget_exceeded());
        assert_eq!(summary.budget_usage_percentage(), Some(0.0));

        summary.total_cost = 80.0;
        assert!(!summary.is_budget_exceeded());
        assert_approx_eq!(summary.budget_usage_percentage().unwrap(), 80.0, 0.1);

        summary.total_cost = 120.0;
        assert!(summary.is_budget_exceeded());
        assert_approx_eq!(summary.budget_usage_percentage().unwrap(), 120.0, 0.1);
    }

    #[test]
    fn test_budget_info() {
        let budget = BudgetInfo::new(100.0, "USD".to_string());

        assert_eq!(budget.monthly_limit, 100.0);
        assert_eq!(budget.currency, "USD");
        assert_approx_eq!(budget.warning_threshold, 80.0, 0.1);
        assert_approx_eq!(budget.alert_threshold, 95.0, 0.1);

        assert!(!budget.is_warning_exceeded(70.0));
        assert!(budget.is_warning_exceeded(85.0));
        assert!(!budget.is_alert_exceeded(90.0));
        assert!(budget.is_alert_exceeded(98.0));
    }

    #[test]
    fn test_analysis_results() {
        let period = AnalysisPeriod::Daily { 
            date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap() 
        };
        let mut results = AnalysisResults::new(period);

        assert!(matches!(results.period, AnalysisPeriod::Daily { .. }));
        assert_eq!(results.total_cost, 0.0);
        assert_eq!(results.total_tokens, 0);
        assert_eq!(results.request_count, 0);
        assert_eq!(results.session_count, 0);
        assert_eq!(results.avg_cost_per_request, 0.0);
        assert_eq!(results.avg_tokens_per_request, 0.0);
        assert!(results.cost_by_model.is_empty());
        assert!(results.trends.daily_costs.is_empty());
        assert!(results.trends.daily_tokens.is_empty());
        assert!(results.trends.model_trends.is_empty());
        assert!(results.trends.cost_growth_rate.is_none());
        assert!(results.trends.token_growth_rate.is_none());
        assert!(results.insights.is_empty());
    }

    #[test]
    fn test_usage_trends() {
        let mut trends = UsageTrends::new();

        assert!(trends.daily_costs.is_empty());
        assert!(trends.daily_tokens.is_empty());
        assert!(trends.model_trends.is_empty());
        assert!(trends.cost_growth_rate.is_none());
        assert!(trends.token_growth_rate.is_none());
    }

    #[test]
    fn test_report_config() {
        let filters = ReportFilters::default();

        assert!(filters.models.is_empty());
        assert!(filters.date_range.is_none());
        assert!(filters.cost_range.is_none());
        assert!(filters.session_ids.is_empty());
        assert!(filters.user_ids.is_empty());
    }

    #[test]
    fn test_date_range_parsing() {
        let date_range = "2024-01-01..2024-01-31";
        let (start, end) = parse_date_range(date_range).unwrap();

        assert_eq!(start.date_naive(), NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
        assert_eq!(end.date_naive(), NaiveDate::from_ymd_opt(2024, 1, 31).unwrap());
    }

    #[test]
    fn test_invalid_date_range_parsing() {
        let invalid_date_range = "invalid-date-range";
        let result = parse_date_range(invalid_date_range);

        assert!(result.is_err());
    }

    #[test]
    fn test_json_data_parsing() {
        let json_data = r#"
        [
            {
                "id": "test-1",
                "timestamp": "2024-01-01T12:00:00Z",
                "model": "claude-3-sonnet",
                "input_tokens": 1000,
                "output_tokens": 500,
                "cost": 0.015
            }
        ]
        "#;

        let records = parse_json_data(json_data).unwrap();
        assert_eq!(records.len(), 1);

        let record = &records[0];
        assert_eq!(record.id, "test-1");
        assert_eq!(record.model, "claude-3-sonnet");
        assert_eq!(record.input_tokens, 1000);
        assert_eq!(record.output_tokens, 500);
        assert_approx_eq!(record.cost, 0.015, 0.0001);
    }

    #[test]
    fn test_csv_data_parsing() {
        let csv_data = r#"
id,timestamp,model,input_tokens,output_tokens,cost
test-1,2024-01-01T12:00:00Z,claude-3-sonnet,1000,500,0.015
test-2,2024-01-01T13:00:00Z,claude-3-opus,2000,1000,0.045
"#;

        let records = parse_csv_data(csv_data).unwrap();
        assert_eq!(records.len(), 2);

        assert_eq!(records[0].id, "test-1");
        assert_eq!(records[1].id, "test-2");
    }

    #[test]
    fn test_data_loader_creation() {
        let loader = DataLoader::with_source(
            DataSourceType::Json,
            "./data/test.json".to_string(),
        );

        // 这是一个简化的测试，实际使用时需要实现DataLoader的具体逻辑
        // 这里只是验证构造函数能够正常工作
        drop(loader);
    }

    #[test]
    fn test_model_usage() {
        let mut model_usage = ModelUsage::new("claude-3-sonnet".to_string());

        assert_eq!(model_usage.model, "claude-3-sonnet");
        assert_eq!(model_usage.total_cost, 0.0);
        assert_eq!(model_usage.total_input_tokens, 0);
        assert_eq!(model_usage.total_output_tokens, 0);
        assert_eq!(model_usage.request_count, 0);
        assert_eq!(model_usage.avg_cost_per_request, 0.0);

        let record = UsageRecord::new(
            Utc::now(),
            "claude-3-sonnet".to_string(),
            1000,
            500,
            0.015,
        );

        model_usage.add_record(&record);

        assert_eq!(model_usage.total_cost, 0.015);
        assert_eq!(model_usage.total_input_tokens, 1000);
        assert_eq!(model_usage.total_output_tokens, 500);
        assert_eq!(model_usage.request_count, 1);

        model_usage.calculate_avg_cost();
        assert_approx_eq!(model_usage.avg_cost_per_request, 0.015, 0.0001);
    }

    #[test]
    fn test_serialization() {
        let record = UsageRecord::new(
            Utc::now(),
            "claude-3-sonnet".to_string(),
            1000,
            500,
            0.015,
        );

        // 测试JSON序列化
        let json_str = serde_json::to_string(&record).unwrap();
        let deserialized: UsageRecord = serde_json::from_str(&json_str).unwrap();

        assert_eq!(record.id, deserialized.id);
        assert_eq!(record.model, deserialized.model);
        assert_eq!(record.input_tokens, deserialized.input_tokens);
        assert_eq!(record.output_tokens, deserialized.output_tokens);
        assert_approx_eq!(record.cost, deserialized.cost, 0.0001);
    }

    #[test]
    fn test_edge_cases() {
        // 测试零令牌情况
        let record = UsageRecord::new(
            Utc::now(),
            "claude-3-sonnet".to_string(),
            0,
            0,
            0.0,
        );

        assert_eq!(record.total_tokens(), 0);
        assert_eq!(record.cost_per_token(), 0.0);

        // 测试空会话
        let session = Session::new(
            "empty_session".to_string(),
            Utc::now(),
            None,
        );

        assert_eq!(session.request_count, 0);
        assert_eq!(session.avg_cost_per_request(), 0.0);
        assert_eq!(session.avg_tokens_per_request(), 0.0);

        // 测试空预算
        let budget = BudgetInfo::new(0.0, "USD".to_string());
        assert!(!budget.is_warning_exceeded(0.0));
        assert!(!budget.is_alert_exceeded(0.0));
    }

    #[test]
    fn test_error_conditions() {
        // 测试无效的日期范围格式
        let result = parse_date_range("invalid-format");
        assert!(result.is_err());

        // 测试无效的JSON数据
        let result = parse_json_data("invalid json");
        assert!(result.is_err());

        // 测试无效的CSV数据
        let result = parse_csv_data("invalid,csv,data\nwith,wrong,columns");
        assert!(result.is_err());
    }

    #[test]
    fn test_performance_considerations() {
        let mut records = Vec::new();
        
        // 创建大量记录来测试性能
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

        // 测试序列化性能
        let start = std::time::Instant::now();
        let _json_str = serde_json::to_string(&records).unwrap();
        let duration = start.elapsed();
        assert!(duration.as_millis() < 100); // 应该在100ms内完成

        // 测试反序列化性能
        let json_str = serde_json::to_string(&records).unwrap();
        let start = std::time::Instant::now();
        let _deserialized: Vec<UsageRecord> = serde_json::from_str(&json_str).unwrap();
        let duration = start.elapsed();
        assert!(duration.as_millis() < 100); // 应该在100ms内完成
    }
}