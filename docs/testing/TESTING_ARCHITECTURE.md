# ccusage-rs 测试架构设计

## 概述

本文档描述了 ccusage-rs 的测试架构设计，专注于提供高可测试性的架构和依赖注入模式，确保代码质量和系统稳定性。

## 测试架构原则

1. **可测试性优先**: 所有组件都设计为可单元测试
2. **依赖注入**: 使用依赖注入模式，便于测试
3. **模拟友好**: 设计模拟友好的接口
4. **测试覆盖**: 目标达到 95% 的测试覆盖率
5. **分层测试**: 单元测试、集成测试、端到端测试

## 依赖注入架构

### 核心接口设计
```rust
use std::sync::Arc;
use std::path::Path;

// 数据处理器接口
pub trait DataProcessor {
    async fn load_entries(&self, path: &Path) -> Result<Vec<UsageEntry>>;
    fn filter_by_date(&self, entries: &[UsageEntry], start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<UsageEntry>;
}

// 成本计算器接口
pub trait CostCalculator {
    fn calculate_entry_cost(&self, entry: &UsageEntry, mode: &CostMode) -> Result<f64>;
    fn calculate_total_cost(&self, entries: &[UsageEntry], mode: &CostMode) -> Result<f64>;
}

// 数据聚合器接口
pub trait DataAggregator {
    fn aggregate_daily(&self, entries: &[UsageEntry]) -> Vec<DailyUsage>;
    fn aggregate_monthly(&self, entries: &[UsageEntry]) -> Vec<MonthlyUsage>;
    fn aggregate_sessions(&self, entries: &[UsageEntry]) -> Vec<SessionUsage>;
}

// 输出格式化器接口
pub trait OutputFormatter {
    fn format_daily_report(&self, report: &DailyReport) -> Result<String>;
    fn format_monthly_report(&self, report: &MonthlyReport) -> Result<String>;
    fn format_session_report(&self, report: &SessionReport) -> Result<String>;
    fn format_json<T: Serialize>(&self, data: &T) -> Result<String>;
}
```

### 依赖注入容器
```rust
pub struct AppContainer {
    pub config: Config,
    pub data_processor: Arc<dyn DataProcessor>,
    pub cost_calculator: Arc<dyn CostCalculator>,
    pub data_aggregator: Arc<dyn DataAggregator>,
    pub output_formatter: Arc<dyn OutputFormatter>,
}

impl AppContainer {
    pub fn new(config: Config) -> Self {
        Self {
            data_processor: Arc::new(StdDataProcessor::new(config.clone())),
            cost_calculator: Arc::new(StdCostCalculator::new()),
            data_aggregator: Arc::new(StdDataAggregator),
            output_formatter: Arc::new(StdOutputFormatter::new()),
            config,
        }
    }
    
    // 用于测试的构造函数
    pub fn with_components(
        config: Config,
        data_processor: Arc<dyn DataProcessor>,
        cost_calculator: Arc<dyn CostCalculator>,
        data_aggregator: Arc<dyn DataAggregator>,
        output_formatter: Arc<dyn OutputFormatter>,
    ) -> Self {
        Self {
            config,
            data_processor,
            cost_calculator,
            data_aggregator,
            output_formatter,
        }
    }
}
```

### 标准实现
```rust
// 标准数据处理器实现
pub struct StdDataProcessor {
    config: Config,
}

impl DataProcessor for StdDataProcessor {
    async fn load_entries(&self, path: &Path) -> Result<Vec<UsageEntry>> {
        let content = tokio::fs::read_to_string(path).await?;
        let entries: Vec<UsageEntry> = content
            .lines()
            .filter_map(|line| serde_json::from_str(line).ok())
            .collect();
        Ok(entries)
    }
    
    fn filter_by_date(&self, entries: &[UsageEntry], start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<UsageEntry> {
        entries
            .iter()
            .filter(|entry| entry.timestamp >= start && entry.timestamp <= end)
            .cloned()
            .collect()
    }
}

// 标准成本计算器实现
pub struct StdCostCalculator {
    pricing_cache: std::collections::HashMap<String, f64>,
}

impl CostCalculator for StdCostCalculator {
    fn calculate_entry_cost(&self, entry: &UsageEntry, mode: &CostMode) -> Result<f64> {
        match mode {
            CostMode::Display => entry.cost_usd.ok_or_else(|| anyhow!("No pre-calculated cost available")),
            CostMode::Calculate => {
                let input_price = self.get_model_price(&entry.model, "input")?;
                let output_price = self.get_model_price(&entry.model, "output")?;
                
                let input_cost = entry.usage.input_tokens as f64 * input_price;
                let output_cost = entry.usage.output_tokens as f64 * output_price;
                
                Ok(input_cost + output_cost)
            }
            CostMode::Auto => {
                if let Some(cost) = entry.cost_usd {
                    Ok(cost)
                } else {
                    self.calculate_entry_cost(entry, &CostMode::Calculate)
                }
            }
        }
    }
    
    fn calculate_total_cost(&self, entries: &[UsageEntry], mode: &CostMode) -> Result<f64> {
        entries.iter().map(|entry| self.calculate_entry_cost(entry, mode)).sum()
    }
}
```

## 测试模拟对象

### 模拟数据处理器
```rust
#[cfg(test)]
pub struct MockDataProcessor {
    entries: Vec<UsageEntry>,
    should_fail: bool,
}

#[cfg(test)]
impl MockDataProcessor {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            should_fail: false,
        }
    }
    
    pub fn with_entries(mut self, entries: Vec<UsageEntry>) -> Self {
        self.entries = entries;
        self
    }
    
    pub fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }
}

#[cfg(test)]
impl DataProcessor for MockDataProcessor {
    async fn load_entries(&self, _path: &Path) -> Result<Vec<UsageEntry>> {
        if self.should_fail {
            Err(anyhow!("Mock data processor failure"))
        } else {
            Ok(self.entries.clone())
        }
    }
    
    fn filter_by_date(&self, entries: &[UsageEntry], start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<UsageEntry> {
        entries
            .iter()
            .filter(|entry| entry.timestamp >= start && entry.timestamp <= end)
            .cloned()
            .collect()
    }
}
```

### 模拟成本计算器
```rust
#[cfg(test)]
pub struct MockCostCalculator {
    costs: std::collections::HashMap<String, f64>,
    should_fail: bool,
}

#[cfg(test)]
impl MockCostCalculator {
    pub fn new() -> Self {
        Self {
            costs: std::collections::HashMap::new(),
            should_fail: false,
        }
    }
    
    pub fn with_cost(mut self, model: &str, cost: f64) -> Self {
        self.costs.insert(model.to_string(), cost);
        self
    }
    
    pub fn with_failure(mut self) -> Self {
        self.should_fail = true;
        self
    }
}

#[cfg(test)]
impl CostCalculator for MockCostCalculator {
    fn calculate_entry_cost(&self, entry: &UsageEntry, _mode: &CostMode) -> Result<f64> {
        if self.should_fail {
            Err(anyhow!("Mock cost calculator failure"))
        } else {
            Ok(self.costs.get(&entry.model).copied().unwrap_or(0.0))
        }
    }
    
    fn calculate_total_cost(&self, entries: &[UsageEntry], mode: &CostMode) -> Result<f64> {
        entries.iter().map(|entry| self.calculate_entry_cost(entry, mode)).sum()
    }
}
```

## 测试工厂和辅助函数

### 测试数据工厂
```rust
#[cfg(test)]
pub struct TestDataFactory;

#[cfg(test)]
impl TestDataFactory {
    pub fn create_test_entry() -> UsageEntry {
        UsageEntry {
            timestamp: Utc::now(),
            model: "claude-3-sonnet-20240229".to_string(),
            usage: TokenUsage {
                input_tokens: 1000,
                output_tokens: 500,
                cache_creation_input_tokens: Some(200),
                cache_read_input_tokens: Some(100),
            },
            cost_usd: Some(0.015),
            session_id: Some("test-session".to_string()),
            project_path: Some("/test/project".to_string()),
            request_id: "req-123".to_string(),
            message_id: "msg-123".to_string(),
        }
    }
    
    pub fn create_test_entries(count: usize) -> Vec<UsageEntry> {
        (0..count).map(|i| {
            let mut entry = Self::create_test_entry();
            entry.timestamp = Utc::now() - chrono::Duration::days(i as i64);
            entry.request_id = format!("req-{}", i);
            entry.message_id = format!("msg-{}", i);
            entry
        }).collect()
    }
    
    pub fn create_test_config() -> Config {
        Config {
            data_dir: Some(PathBuf::from("/tmp")),
            cost_mode: CostMode::Calculate,
            timezone: "UTC".to_string(),
            locale: "en".to_string(),
            offline: true,
            compact: false,
            debug: false,
        }
    }
}
```

### 测试辅助函数
```rust
#[cfg(test)]
pub mod test_helpers {
    use super::*;
    
    pub fn create_temp_file_with_data(data: &str) -> tempfile::NamedTempFile {
        let mut file = tempfile::NamedTempFile::new().unwrap();
        write!(file, "{}", data).unwrap();
        file
    }
    
    pub fn create_temp_jsonl_file(entries: &[UsageEntry]) -> tempfile::NamedTempFile {
        let mut content = String::new();
        for entry in entries {
            content.push_str(&serde_json::to_string(entry).unwrap());
            content.push('\n');
        }
        create_temp_file_with_data(&content)
    }
    
    pub async fn setup_test_environment() -> tempfile::TempDir {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let data_dir = temp_dir.path().join("data");
        tokio::fs::create_dir_all(&data_dir).await.unwrap();
        temp_dir
    }
}
```

## 单元测试示例

### 数据处理器测试
```rust
#[cfg(test)]
mod data_processor_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_load_entries_success() {
        let entries = TestDataFactory::create_test_entries(5);
        let temp_file = test_helpers::create_temp_jsonl_file(&entries);
        
        let processor = StdDataProcessor::new(TestDataFactory::create_test_config());
        let result = processor.load_entries(temp_file.path()).await;
        
        assert!(result.is_ok());
        let loaded_entries = result.unwrap();
        assert_eq!(loaded_entries.len(), 5);
    }
    
    #[tokio::test]
    async fn test_load_entries_file_not_found() {
        let processor = StdDataProcessor::new(TestDataFactory::create_test_config());
        let result = processor.load_entries(Path::new("/nonexistent/file.jsonl")).await;
        
        assert!(result.is_err());
    }
    
    #[test]
    fn test_filter_by_date() {
        let entries = TestDataFactory::create_test_entries(10);
        let processor = StdDataProcessor::new(TestDataFactory::create_test_config());
        
        let start = Utc::now() - chrono::Duration::days(5);
        let end = Utc::now() - chrono::Duration::days(2);
        
        let filtered = processor.filter_by_date(&entries, start, end);
        assert_eq!(filtered.len(), 4); // days 2, 3, 4, 5
    }
}
```

### 成本计算器测试
```rust
#[cfg(test)]
mod cost_calculator_tests {
    use super::*;
    
    #[test]
    fn test_calculate_entry_cost_display_mode() {
        let entry = TestDataFactory::create_test_entry();
        let calculator = StdCostCalculator::new();
        
        let result = calculator.calculate_entry_cost(&entry, &CostMode::Display);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.015);
    }
    
    #[test]
    fn test_calculate_entry_cost_calculate_mode() {
        let mut entry = TestDataFactory::create_test_entry();
        entry.cost_usd = None; // Remove pre-calculated cost
        
        let calculator = StdCostCalculator::new();
        let result = calculator.calculate_entry_cost(&entry, &CostMode::Calculate);
        
        assert!(result.is_ok());
        assert!(result.unwrap() > 0.0);
    }
    
    #[test]
    fn test_calculate_total_cost() {
        let entries = TestDataFactory::create_test_entries(3);
        let calculator = StdCostCalculator::new();
        
        let result = calculator.calculate_total_cost(&entries, &CostMode::Display);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.045); // 0.015 * 3
    }
}
```

## 集成测试示例

### 完整工作流测试
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_full_daily_workflow() {
        // Setup
        let entries = TestDataFactory::create_test_entries(10);
        let temp_file = test_helpers::create_temp_jsonl_file(&entries);
        let config = TestDataFactory::create_test_config();
        
        // Create container with real components
        let container = AppContainer::new(config);
        
        // Load data
        let loaded_entries = container.data_processor.load_entries(temp_file.path()).await.unwrap();
        
        // Filter by date
        let start = Utc::now() - chrono::Duration::days(7);
        let end = Utc::now();
        let filtered_entries = container.data_processor.filter_by_date(&loaded_entries, start, end);
        
        // Calculate costs
        let total_cost = container.cost_calculator.calculate_total_cost(&filtered_entries, &CostMode::Calculate).unwrap();
        
        // Aggregate daily
        let daily_data = container.data_aggregator.aggregate_daily(&filtered_entries);
        
        // Assertions
        assert!(!filtered_entries.is_empty());
        assert!(total_cost > 0.0);
        assert!(!daily_data.is_empty());
    }
    
    #[tokio::test]
    async fn test_error_handling_workflow() {
        // Test with mock components that fail
        let mock_processor = Arc::new(MockDataProcessor::new().with_failure());
        let mock_calculator = Arc::new(MockCostCalculator::new().with_failure());
        let config = TestDataFactory::create_test_config();
        
        let container = AppContainer::with_components(
            config,
            mock_processor,
            mock_calculator,
            Arc::new(StdDataAggregator),
            Arc::new(StdOutputFormatter::new()),
        );
        
        // Test that errors are properly propagated
        let result = container.data_processor.load_entries(Path::new("/test/path")).await;
        assert!(result.is_err());
    }
}
```

## 端到端测试示例

### CLI 测试
```rust
#[cfg(test)]
mod e2e_tests {
    use super::*;
    use assert_cmd::Command;
    use predicates::prelude::*;
    
    #[tokio::test]
    async fn test_cli_daily_command() {
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        
        // Create test data
        let entries = TestDataFactory::create_test_entries(5);
        let temp_file = test_helpers::create_temp_jsonl_file(&entries);
        
        // Run command
        cmd.arg("daily")
           .arg("--data")
           .arg(temp_file.path())
           .arg("--json");
        
        cmd.assert()
           .success()
           .stdout(predicate::str::contains("daily"))
           .stdout(predicate::str::contains("totals"));
    }
    
    #[tokio::test]
    async fn test_cli_error_handling() {
        let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
        
        // Test with non-existent file
        cmd.arg("daily")
           .arg("--data")
           .arg("/nonexistent/file.jsonl");
        
        cmd.assert()
           .failure()
           .stderr(predicate::str::contains("Data file not found"));
    }
}
```

## 测试覆盖率策略

### 测试覆盖率目标
- **单元测试**: 90%+ 覆盖率
- **集成测试**: 80%+ 覆盖率
- **端到端测试**: 70%+ 覆盖率
- **总体覆盖率**: 85%+ 目标

### 覆盖率工具配置
```toml
[dev-dependencies]
tarpaulin = "0.27"
```

### 运行覆盖率测试
```bash
# 运行单元测试覆盖率
cargo tarpaulin --lib --out html

# 运行集成测试覆盖率
cargo tarpaulin --test integration --out html

# 运行完整测试覆盖率
cargo tarpaulin --all --out html --exclude-files /*tests/*
```

## 性能测试

### 基准测试示例
```rust
#[cfg(test)]
mod benchmarks {
    use super::*;
    use criterion::{criterion_group, criterion_main, Criterion};
    
    fn bench_data_processing(c: &mut Criterion) {
        let entries = TestDataFactory::create_test_entries(1000);
        let processor = StdDataProcessor::new(TestDataFactory::create_test_config());
        
        c.bench_function("data_processing", |b| {
            b.iter(|| {
                processor.filter_by_date(&entries, Utc::now() - chrono::Duration::days(30), Utc::now())
            })
        });
    }
    
    fn bench_cost_calculation(c: &mut Criterion) {
        let entries = TestDataFactory::create_test_entries(1000);
        let calculator = StdCostCalculator::new();
        
        c.bench_function("cost_calculation", |b| {
            b.iter(|| {
                calculator.calculate_total_cost(&entries, &CostMode::Calculate)
            })
        });
    }
    
    criterion_group!(benches, bench_data_processing, bench_cost_calculation);
    criterion_main!(benches);
}
```

## 测试环境配置

### 测试环境变量
```bash
# 设置测试环境
export RUST_TEST_THREADS=1
export RUST_BACKTRACE=1
export RUST_LOG=debug

# 运行特定测试
cargo test data_processor_tests
cargo test --test integration_tests
cargo test --test e2e_tests

# 运行所有测试
cargo test --all
```

### CI/CD 测试配置
```yaml
# GitHub Actions 示例
name: Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
    - name: Run tests
      run: |
        cargo test --all
        cargo tarpaulin --all --out xml
    - name: Upload coverage
      uses: codecov/codecov-action@v3
```

## 总结

这个测试架构设计提供了：

1. **完整的依赖注入**: 所有组件都可以通过接口替换
2. **丰富的模拟对象**: 便于测试各种场景
3. **分层测试策略**: 单元测试、集成测试、端到端测试
4. **测试覆盖率**: 目标达到 95% 的测试覆盖率
5. **性能测试**: 包含基准测试和性能监控
6. **CI/CD 集成**: 自动化测试和覆盖率报告

通过这个测试架构，可以确保 ccusage-rs 项目的代码质量和系统稳定性，同时为未来的功能扩展提供可靠的测试基础。