# ccusage-rs 项目修复验证和测试计划

## 概述

本测试计划旨在验证编译错误修复的有效性，确保修复后的代码质量、功能和性能达到预期标准。测试覆盖从基础编译验证到完整功能测试的各个层面。

## 测试目标

### 主要目标
1. **编译验证**: 确保项目在所有目标平台成功编译
2. **功能验证**: 验证所有核心功能正常工作
3. **性能验证**: 确保修复后性能不降级
4. **兼容性验证**: 验证与外部依赖的兼容性

### 质量目标
1. **测试覆盖率**: 代码覆盖率 > 80%
2. **错误处理**: 所有错误路径都有测试覆盖
3. **边界条件**: 所有边界条件都有测试验证
4. **回归测试**: 防止已修复问题重新出现

## 测试策略

### 1. 分层测试策略

```
测试层次:
├── 编译测试 (Compilation Tests)
│   ├── 基础编译检查
│   ├── 跨平台编译
│   └── 优化编译
├── 单元测试 (Unit Tests)
│   ├── 模块功能测试
│   ├── 错误处理测试
│   └── 边界条件测试
├── 集成测试 (Integration Tests)
│   ├── 模块间交互测试
│   ├── 数据流测试
│   └── 配置测试
├── 系统测试 (System Tests)
│   ├── 端到端功能测试
│   ├── 性能测试
│   └── 兼容性测试
└── 验收测试 (Acceptance Tests)
    ├── 用户场景测试
    ├── 数据完整性测试
    └── 部署测试
```

### 2. 风险驱动测试

基于编译错误分析，重点关注以下风险区域：

#### 高风险区域
1. **时间处理功能** - Chrono API变更
2. **类型转换** - Option和Result处理
3. **依赖管理** - 版本兼容性
4. **错误处理** - 统一错误类型

#### 中风险区域
1. **文件操作** - 路径处理和IO
2. **配置管理** - 访问控制和序列化
3. **CLI功能** - 命令行参数处理
4. **数据模型** - 序列化和反序列化

## 详细测试计划

### 阶段1: 编译验证测试

#### 1.1 基础编译测试
```bash
# 基础编译检查
cargo check

# 完整编译测试
cargo build

# 发布模式编译
cargo build --release

# 文档编译测试
cargo doc
```

#### 1.2 跨平台编译测试
```bash
# Linux平台
cargo check --target=x86_64-unknown-linux-gnu

# macOS平台
cargo check --target=x86_64-apple-darwin

# Windows平台
cargo check --target=x86_64-pc-windows-msvc
```

#### 1.3 特性编译测试
```bash
# 测试不同特性组合
cargo check --no-default-features
cargo check --features "core"
cargo check --features "monitoring"
cargo check --features "advanced"
cargo check --features "full"
```

### 阶段2: 单元测试

#### 2.1 时间处理测试
```rust
// tests/unit/temporal_tests.rs
mod temporal_tests;

use chrono::{DateTime, Utc, NaiveDate};
use crate::infrastructure::temporal::{TemporalOperations, DefaultTemporal};

#[test]
fn test_date_creation() {
    let result = SafeDate::new(2023, 12, 25);
    assert!(result.is_ok());
    
    let invalid_result = SafeDate::new(2023, 13, 32);
    assert!(invalid_result.is_err());
}

#[test]
fn test_date_parsing() {
    let temporal = DefaultTemporal;
    
    let result1 = temporal.parse_date("2023-12-25");
    assert!(result1.is_ok());
    
    let result2 = temporal.parse_date("2023/12/25");
    assert!(result2.is_ok());
    
    let result3 = temporal.parse_date("invalid-date");
    assert!(result3.is_err());
}

#[test]
fn test_date_arithmetic() {
    let date = SafeDate::new(2023, 12, 25).unwrap();
    let next_day = date.add_days(1);
    
    assert_eq!(next_day.year(), 2023);
    assert_eq!(next_day.month(), 12);
    assert_eq!(next_day.day(), 26);
}

#[test]
fn test_weekday_calculation() {
    let date = SafeDate::new(2023, 12, 25).unwrap(); // Monday
    assert_eq!(date.weekday(), chrono::Weekday::Mon);
}

#[test]
fn test_previous_day() {
    let date = SafeDate::new(2023, 12, 25).unwrap();
    let prev_day = date.previous_day().unwrap();
    
    assert_eq!(prev_day.year(), 2023);
    assert_eq!(prev_day.month(), 12);
    assert_eq!(prev_day.day(), 24);
}
```

#### 2.2 类型安全测试
```rust
// tests/unit/types_tests.rs
mod types_tests;

use crate::types::{Amount, TokenCount};

#[test]
fn test_amount_creation() {
    let valid_amount = Amount::new(100.0);
    assert!(valid_amount.is_ok());
    
    let invalid_amount = Amount::new(-10.0);
    assert!(invalid_amount.is_err());
}

#[test]
fn test_amount_from_str() {
    let amount = "100.50".parse::<Amount>();
    assert!(amount.is_ok());
    assert_eq!(amount.unwrap().value(), 100.50);
    
    let invalid_amount = "invalid".parse::<Amount>();
    assert!(invalid_amount.is_err());
}

#[test]
fn test_token_count_operations() {
    let tokens = TokenCount::new(1000);
    assert_eq!(tokens.value(), 1000);
    
    let more_tokens = TokenCount::new(2000);
    assert!(more_tokens > tokens);
}
```

#### 2.3 错误处理测试
```rust
// tests/unit/error_tests.rs
mod error_tests;

use crate::error::{CcusageError, Result};

#[test]
fn test_error_creation() {
    let config_error = CcusageError::Config("test config error".to_string());
    assert_eq!(config_error.to_string(), "Configuration error: test config error");
}

#[test]
fn test_error_conversion() {
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let ccusage_error: CcusageError = io_error.into();
    
    assert!(matches!(ccusage_error, CcusageError::Io(_)));
}

#[test]
fn test_result_operations() {
    let result: Result<i32> = Ok(42);
    assert_eq!(result.unwrap(), 42);
    
    let error_result: Result<i32> = Err(CcusageError::Validation("test error".to_string()));
    assert!(error_result.is_err());
}
```

#### 2.4 配置管理测试
```rust
// tests/unit/config_tests.rs
mod config_tests;

use crate::infrastructure::config::AppConfig;
use tempfile::TempDir;

#[test]
fn test_config_creation() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    
    let config = AppConfig::default();
    assert!(config.save_to(&config_path).is_ok());
    
    let loaded_config = AppConfig::load_from(&config_path);
    assert!(loaded_config.is_ok());
}

#[test]
fn test_config_validation() {
    let mut config = AppConfig::default();
    config.database.max_connections = 0;
    
    let result = config.validate();
    assert!(result.is_err());
}
```

### 阶段3: 集成测试

#### 3.1 数据流测试
```rust
// tests/integration/data_flow_tests.rs
mod data_flow_tests;

use crate::core::services::UsageService;
use crate::core::models::UsageRecord;
use chrono::Utc;

#[test]
fn test_usage_record_processing() {
    let service = UsageService::new();
    
    let record = UsageRecord::new(
        Utc::now(),
        "claude-3-sonnet".to_string(),
        1000,
        500,
        0.015,
    );
    
    let result = service.process_record(record);
    assert!(result.is_ok());
    
    let summary = service.get_daily_summary(Utc::now().date_naive());
    assert!(summary.is_ok());
}

#[test]
fn test_cost_calculation_flow() {
    let service = UsageService::new();
    
    let records = vec![
        UsageRecord::new(Utc::now(), "claude-3-sonnet".to_string(), 1000, 500, 0.015),
        UsageRecord::new(Utc::now(), "claude-3-sonnet".to_string(), 2000, 1000, 0.030),
    ];
    
    let total_cost = service.calculate_total_cost(&records);
    assert!(total_cost.is_ok());
    assert!(total_cost.unwrap() > 0.0);
}
```

#### 3.2 模块交互测试
```rust
// tests/integration/module_interaction_tests.rs
mod module_interaction_tests;

use crate::infrastructure::storage::StorageManager;
use crate::infrastructure::config::AppConfig;
use crate::core::services::UsageService;

#[test]
fn test_storage_service_integration() {
    let config = AppConfig::default();
    let storage = StorageManager::new(&config);
    let service = UsageService::with_storage(storage);
    
    // 测试存储和检索
    let record = create_test_record();
    let save_result = service.save_record(record.clone());
    assert!(save_result.is_ok());
    
    let retrieved_record = service.get_record(&record.id);
    assert!(retrieved_record.is_ok());
    assert_eq!(retrieved_record.unwrap().id, record.id);
}

#[test]
fn test_config_service_integration() {
    let config = AppConfig::default();
    let service = UsageService::with_config(config);
    
    let pricing = service.get_pricing_info("claude-3-sonnet");
    assert!(pricing.is_ok());
}
```

### 阶段4: 系统测试

#### 4.1 端到端功能测试
```rust
// tests/system/e2e_tests.rs
mod e2e_tests;

use std::process::Command;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_cli_functionality() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");
    
    // 创建测试配置
    create_test_config(&config_path);
    
    // 测试帮助命令
    let help_output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .output()
        .expect("Failed to execute help command");
    
    assert!(help_output.status.success());
    assert!(String::from_utf8_lossy(&help_output.stdout).contains("ccusage-rs"));
    
    // 测试版本命令
    let version_output = Command::new("cargo")
        .args(&["run", "--", "--version"])
        .output()
        .expect("Failed to execute version command");
    
    assert!(version_output.status.success());
}

#[test]
fn test_data_analysis_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let data_path = temp_dir.path().join("test_data.json");
    
    // 创建测试数据
    create_test_data(&data_path);
    
    // 测试分析命令
    let analysis_output = Command::new("cargo")
        .args(&["run", "--", "analyze", "--data", &data_path.to_string_lossy()])
        .output()
        .expect("Failed to execute analysis command");
    
    assert!(analysis_output.status.success());
    let output = String::from_utf8_lossy(&analysis_output.stdout);
    assert!(output.contains("total_cost"));
    assert!(output.contains("total_tokens"));
}
```

#### 4.2 性能测试
```rust
// tests/system/performance_tests.rs
mod performance_tests;

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use crate::core::services::UsageService;
use crate::core::models::UsageRecord;
use chrono::Utc;

fn bench_record_processing(c: &mut Criterion) {
    let service = UsageService::new();
    let mut group = c.benchmark_group("record_processing");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::new("process_records", size), size, |b, &size| {
            let records: Vec<UsageRecord> = (0..size)
                .map(|i| UsageRecord::new(
                    Utc::now() + chrono::Duration::seconds(i as i64),
                    "claude-3-sonnet".to_string(),
                    1000 + i as u32,
                    500 + i as u32,
                    0.015,
                ))
                .collect();
            
            b.iter(|| service.process_batch(&records));
        });
    }
    
    group.finish();
}

fn bench_cost_calculation(c: &mut Criterion) {
    let service = UsageService::new();
    let records: Vec<UsageRecord> = (0..1000)
        .map(|i| UsageRecord::new(
            Utc::now(),
            "claude-3-sonnet".to_string(),
            1000 + i as u32,
            500 + i as u32,
            0.015,
        ))
        .collect();
    
    c.bench_function("calculate_total_cost", |b| {
        b.iter(|| service.calculate_total_cost(&records));
    });
}

criterion_group!(benches, bench_record_processing, bench_cost_calculation);
criterion_main!(benches);
```

### 阶段5: 兼容性测试

#### 5.1 依赖兼容性测试
```bash
# 测试不同版本的依赖兼容性
cargo update
cargo test

# 测试最小版本依赖
cargo build --Z minimal-versions

# 测试最新版本依赖
cargo update --aggressive
cargo test
```

#### 5.2 平台兼容性测试
```bash
# Docker多平台测试
docker build -t ccusage-test .
docker run --rm ccusage-test cargo test

# 交叉编译测试
cargo build --target=aarch64-unknown-linux-gnu
cargo build --target=armv7-unknown-linux-gnueabihf
```

## 测试自动化

### 1. CI/CD配置
```yaml
# .github/workflows/test.yml
name: Test Suite

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, beta]
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@master
      with:
        toolchain: ${{ matrix.rust }}
        components: rustfmt, clippy
    
    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Check formatting
      run: cargo fmt --all -- --check
    
    - name: Run clippy
      run: cargo clippy -- -D warnings
    
    - name: Run tests
      run: cargo test --all-features
    
    - name: Run tests without default features
      run: cargo test --no-default-features
    
    - name: Build documentation
      run: cargo doc --no-deps
```

### 2. 测试覆盖率配置
```toml
# Cargo.toml
[package.metadata.cargo-udeps]
ignore = ["tracing"]

[dev-dependencies]
cargo-tarpaulin = "0.27"
```

```bash
# 运行覆盖率测试
cargo tarpaulin --out html --output-dir coverage/
cargo tarpaulin --out Xml --output-dir coverage/
```

### 3. 性能基准测试
```bash
# 运行性能测试
cargo bench

# 生成性能报告
cargo bench -- --output-format bencher
```

## 测试数据管理

### 1. 测试数据生成
```rust
// tests/data/test_data_generator.rs
pub struct TestDataGenerator {
    rng: rand::rngs::ThreadRng,
}

impl TestDataGenerator {
    pub fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
        }
    }
    
    pub fn generate_usage_records(&mut self, count: usize) -> Vec<UsageRecord> {
        (0..count)
            .map(|i| {
                let timestamp = Utc::now() - chrono::Duration::hours(i as i64);
                let model = self.random_model();
                let input_tokens = self.random_token_count();
                let output_tokens = self.random_token_count();
                let cost = self.calculate_cost(input_tokens, output_tokens);
                
                UsageRecord::new(timestamp, model, input_tokens, output_tokens, cost)
            })
            .collect()
    }
    
    fn random_model(&mut self) -> String {
        let models = [
            "claude-3-sonnet",
            "claude-3-opus",
            "claude-3-haiku",
        ];
        models[self.rng.gen_range(0..models.len())].to_string()
    }
    
    fn random_token_count(&mut self) -> u32 {
        self.rng.gen_range(100..10000)
    }
    
    fn calculate_cost(&self, input_tokens: u32, output_tokens: u32) -> f64 {
        // 简化的成本计算
        (input_tokens as f64 * 0.003 / 1000.0) + (output_tokens as f64 * 0.015 / 1000.0)
    }
}
```

### 2. 测试数据清理
```rust
// tests/data/test_data_cleaner.rs
pub struct TestDataCleaner;

impl TestDataCleaner {
    pub fn cleanup_temp_files() {
        // 清理临时测试文件
        let temp_dir = std::env::temp_dir();
        let ccusage_temp_pattern = temp_dir.join("ccusage_test_*");
        
        if let Ok(entries) = std::fs::read_dir(temp_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.starts_with("ccusage_test_") {
                        let _ = std::fs::remove_file(&path);
                    }
                }
            }
        }
    }
    
    pub fn cleanup_test_database() {
        // 清理测试数据库
        let test_db_paths = [
            "test.db",
            "test_data.db",
            "ccusage_test.db",
        ];
        
        for db_path in test_db_paths {
            if std::path::Path::new(db_path).exists() {
                let _ = std::fs::remove_file(db_path);
            }
        }
    }
}
```

## 验收标准

### 1. 编译验收标准
- [ ] 所有目标平台编译成功
- [ ] 所有特性组合编译成功
- [ ] 文档编译成功
- [ ] 零编译警告

### 2. 功能验收标准
- [ ] 所有单元测试通过
- [ ] 所有集成测试通过
- [ ] 所有系统测试通过
- [ ] 测试覆盖率 > 80%

### 3. 性能验收标准
- [ ] 性能基准测试通过
- [ ] 内存使用在预期范围内
- [ ] 响应时间满足要求
- [ ] 无性能回归

### 4. 质量验收标准
- [ ] 代码格式化通过
- [ ] Clippy检查通过
- [ ] 文档完整
- [ ] 错误处理完善

## 报告和文档

### 1. 测试报告模板
```markdown
# 测试报告

## 测试摘要
- 测试时间: 2024-XX-XX
- 测试版本: v0.1.0
- 测试环境: Linux/macOS/Windows

## 测试结果
- 编译测试: ✅ 通过
- 单元测试: ✅ 通过 (覆盖率: 85%)
- 集成测试: ✅ 通过
- 系统测试: ✅ 通过
- 性能测试: ✅ 通过

## 发现的问题
1. 问题1: 描述和解决方案
2. 问题2: 描述和解决方案

## 建议和改进
1. 建议1: 具体改进建议
2. 建议2: 具体改进建议
```

### 2. 持续监控
- 编译状态监控
- 测试覆盖率监控
- 性能指标监控
- 错误率监控

---

本测试计划确保修复工作的质量和可靠性，通过系统性的测试验证，确保项目在修复后达到预期的质量和性能标准。测试过程将自动化，并提供详细的测试报告和监控指标。