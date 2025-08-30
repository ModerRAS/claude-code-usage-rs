# ccusage-rs 项目架构调整建议

## 概述

基于编译错误分析，项目需要进行一系列架构调整以提高代码质量、可维护性和未来兼容性。本文档详细描述了必要的架构改进措施。

## 当前架构问题分析

### 1. 依赖管理问题
- **问题**: 依赖版本不兼容，缺少版本锁定
- **影响**: 编译错误，难以重现构建环境
- **风险**: 未来升级时可能出现破坏性变更

### 2. API兼容性问题
- **问题**: Chrono 0.4+ API变更未适配
- **影响**: 大量编译错误集中在时间处理
- **风险**: 时间相关功能完全失效

### 3. 类型安全问题
- **问题**: Option处理不当，类型转换错误
- **影响**: 运行时panic风险
- **风险**: 数据丢失和程序崩溃

### 4. 模块化问题
- **问题**: 访问控制不当，模块职责不清
- **影响**: 代码耦合度高，难以维护
- **风险**: 功能扩展困难

## 架构调整建议

### 1. 依赖管理架构改进

#### 1.1 版本管理策略
```toml
# 在Cargo.toml中实现更严格的版本控制
[dependencies]
# 核心依赖 - 使用兼容版本号
chrono = { version = "~0.4.38", features = ["serde"] }
tokio = { version = "~1.42", features = ["full"] }
serde = { version = "~1.0", features = ["derive"] }

# 开发依赖 - 使用宽松版本
[dev-dependencies]
rand = "0.8"
criterion = "0.5"

# 依赖解析策略
[package.metadata.cargo-udeps]
ignore = ["tracing"]  # 忽略某些依赖检查
```

#### 1.2 依赖分层架构
```
依赖层次:
├── 核心层 (Core) - 基础运行时
│   ├── tokio (异步运行时)
│   ├── serde (序列化)
│   └── chrono (时间处理)
├── 业务层 (Business) - 功能依赖
│   ├── clap (CLI)
│   ├── config (配置)
│   └── anyhow (错误处理)
└── 工具层 (Utilities) - 辅助工具
    ├── dirs (目录操作)
    ├── walkdir (文件遍历)
    └── regex (正则表达式)
```

### 2. 时间处理架构重构

#### 2.1 时间抽象层
```rust
// src/temporal/mod.rs
pub mod temporal;

use chrono::{DateTime, Utc, NaiveDate, NaiveTime};

/// 时间处理抽象 trait
pub trait TemporalOperations {
    fn now() -> DateTime<Utc>;
    fn parse_date(date_str: &str) -> Result<NaiveDate>;
    fn format_date(date: NaiveDate) -> String;
    fn add_days(date: NaiveDate, days: i64) -> NaiveDate;
}

/// 默认实现
pub struct DefaultTemporal;

impl TemporalOperations for DefaultTemporal {
    fn now() -> DateTime<Utc> {
        Utc::now()
    }
    
    fn parse_date(date_str: &str) -> Result<NaiveDate> {
        // 统一的日期解析逻辑
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .or_else(|_| NaiveDate::parse_from_str(date_str, "%Y/%m/%d"))
            .map_err(|e| crate::error::CcusageError::Validation(e.to_string()))
    }
    
    fn format_date(date: NaiveDate) -> String {
        date.format("%Y-%m-%d").to_string()
    }
    
    fn add_days(date: NaiveDate, days: i64) -> NaiveDate {
        date.checked_add_days(chrono::Days::new(days as u64))
            .unwrap_or(date)
    }
}
```

#### 2.2 时间安全封装
```rust
// src/temporal/safe_date.rs
pub struct SafeDate {
    inner: NaiveDate,
}

impl SafeDate {
    pub fn new(year: i32, month: u32, day: u32) -> Result<Self> {
        let date = NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| crate::error::CcusageError::Validation(
                format!("Invalid date: {}-{}-{}", year, month, day)
            ))?;
        Ok(Self { inner: date })
    }
    
    pub fn year(&self) -> i32 { self.inner.year() }
    pub fn month(&self) -> u32 { self.inner.month() }
    pub fn day(&self) -> u32 { self.inner.day() }
    
    pub fn weekday(&self) -> chrono::Weekday {
        self.inner.weekday()
    }
    
    pub fn previous_day(&self) -> Option<Self> {
        self.inner.pred_opt().map(|d| Self { inner: d })
    }
}
```

### 3. 类型安全架构

#### 3.1 强类型封装
```rust
// src/types/mod.rs
pub mod types;

use std::str::FromStr;

/// 强类型封装货币金额
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Amount(f64);

impl Amount {
    pub fn new(value: f64) -> Result<Self> {
        if value < 0.0 {
            return Err(crate::error::CcusageError::Validation(
                "Amount cannot be negative".to_string()
            ));
        }
        Ok(Self(value))
    }
    
    pub fn value(&self) -> f64 { self.0 }
}

impl FromStr for Amount {
    type Err = crate::error::CcusageError;
    
    fn from_str(s: &str) -> Result<Self> {
        let value: f64 = s.parse().map_err(|_| {
            crate::error::CcusageError::Validation(
                format!("Invalid amount: {}", s)
            )
        })?;
        Self::new(value)
    }
}

/// 强类型封装Token数量
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TokenCount(u32);

impl TokenCount {
    pub fn new(value: u32) -> Self {
        Self(value)
    }
    
    pub fn value(&self) -> u32 { self.0 }
}
```

#### 3.2 Option处理模式
```rust
// src/types/option_ext.rs
pub trait OptionExt<T> {
    fn or_else_with<F>(self, f: F) -> T
    where
        F: FnOnce() -> T;
    
    fn map_or_else_with<U, D, E>(self, default: D, f: F) -> U
    where
        F: FnOnce(T) -> U,
        D: FnOnce() -> U;
}

impl<T> OptionExt<T> for Option<T> {
    fn or_else_with<F>(self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        self.unwrap_or_else(f)
    }
    
    fn map_or_else_with<U, D, E>(self, default: D, f: F) -> U
    where
        F: FnOnce(T) -> U,
        D: FnOnce() -> U,
    {
        self.map_or_else(default, f)
    }
}
```

### 4. 模块化架构改进

#### 4.1 分层架构设计
```
src/
├── core/              # 核心业务逻辑
│   ├── models/        # 数据模型
│   ├── services/      # 业务服务
│   └── repositories/   # 数据访问层
├── infrastructure/    # 基础设施
│   ├── temporal/      # 时间处理
│   ├── storage/       # 存储抽象
│   └── config/        # 配置管理
├── adapters/          # 适配器层
│   ├── cli/           # CLI适配器
│   ├── api/           # API适配器
│   └── database/     # 数据库适配器
└── utils/             # 工具函数
```

#### 4.2 配置管理重构
```rust
// src/infrastructure/config/mod.rs
pub mod config;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub pricing: PricingConfig,
    pub budget: BudgetConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: PathBuf,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingConfig {
    pub default_model: String,
    pub currency: String,
    pub update_interval_hours: u32,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;
        let content = std::fs::read_to_string(config_path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;
        let content = toml::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;
        Ok(())
    }
    
    fn get_config_path() -> Result<PathBuf> {
        let mut config_dir = dirs::config_dir()
            .ok_or_else(|| crate::error::CcusageError::Config(
                "Could not determine config directory".to_string()
            ))?;
        config_dir.push("ccusage");
        config_dir.push("config.toml");
        Ok(config_dir)
    }
}
```

### 5. 错误处理架构

#### 5.1 统一错误类型
```rust
// src/error/mod.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CcusageError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("File system error: {0}")]
    FileSystem(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Chrono error: {0}")]
    Chrono(#[from] chrono::ParseError),
}

pub type Result<T> = std::result::Result<T, CcusageError>;
```

#### 5.2 错误处理中间件
```rust
// src/error/handler.rs
pub trait ErrorHandler {
    fn handle_error(&self, error: &CcusageError) -> ErrorAction;
}

#[derive(Debug)]
pub enum ErrorAction {
    Retry,
    Abort,
    LogAndContinue,
    NotifyAdmin,
}

pub struct DefaultErrorHandler;

impl ErrorHandler for DefaultErrorHandler {
    fn handle_error(&self, error: &CcusageError) -> ErrorAction {
        match error {
            CcusageError::FileSystem(_) => ErrorAction::Retry,
            CcusageError::Network(_) => ErrorAction::Retry,
            CcusageError::Validation(_) => ErrorAction::LogAndContinue,
            CcusageError::Config(_) => ErrorAction::Abort,
            _ => ErrorAction::LogAndContinue,
        }
    }
}
```

### 6. 测试架构改进

#### 6.1 测试分层策略
```rust
// tests/test_utils.rs
pub mod test_utils;

use tempfile::TempDir;
use std::path::PathBuf;

pub struct TestEnvironment {
    pub temp_dir: TempDir,
    pub config_path: PathBuf,
    pub data_path: PathBuf,
}

impl TestEnvironment {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        let data_path = temp_dir.path().join("data");
        
        std::fs::create_dir_all(&data_path).unwrap();
        
        Self {
            temp_dir,
            config_path,
            data_path,
        }
    }
    
    pub fn create_test_config(&self) -> AppConfig {
        AppConfig {
            database: DatabaseConfig {
                path: self.data_path.join("test.db"),
                max_connections: 1,
            },
            pricing: PricingConfig {
                default_model: "claude-3-sonnet".to_string(),
                currency: "USD".to_string(),
                update_interval_hours: 24,
            },
            budget: BudgetConfig {
                monthly_limit: 100.0,
                warning_threshold: 80.0,
                alert_threshold: 95.0,
            },
            logging: LoggingConfig {
                level: "debug".to_string(),
                file: None,
            },
        }
    }
}
```

#### 6.2 Mock架构
```rust
// tests/mocks.rs
pub mod mocks;

use mockall::mock;
use crate::infrastructure::temporal::TemporalOperations;

mock! {
    pub TemporalMock {}
    
    impl TemporalOperations for TemporalMock {
        fn now() -> chrono::DateTime<chrono::Utc>;
        fn parse_date(date_str: &str) -> Result<chrono::NaiveDate>;
        fn format_date(date: chrono::NaiveDate) -> String;
        fn add_days(date: chrono::NaiveDate, days: i64) -> chrono::NaiveDate;
    }
}
```

## 实施计划

### 阶段1：基础架构调整（第1-2天）
1. **依赖管理重构**
   - 更新Cargo.toml版本策略
   - 实现依赖分层
   - 添加版本锁定

2. **时间处理抽象层**
   - 创建时间抽象trait
   - 实现安全封装
   - 重构现有时间代码

### 阶段2：类型安全改进（第3-4天）
1. **强类型封装**
   - 创建强类型定义
   - 实现类型转换
   - 重构现有代码

2. **错误处理统一**
   - 统一错误类型
   - 实现错误处理中间件
   - 重构错误处理逻辑

### 阶段3：模块化重构（第5-6天）
1. **分层架构**
   - 重新组织模块结构
   - 实现依赖注入
   - 重构业务逻辑

2. **测试架构**
   - 建立测试工具
   - 实现Mock框架
   - 添加测试用例

### 阶段4：验证和优化（第7天）
1. **全面测试**
   - 运行所有测试
   - 性能基准测试
   - 集成测试

2. **文档更新**
   - 更新架构文档
   - API文档生成
   - 用户手册更新

## 成功指标

### 技术指标
- **编译错误**: 0个
- **测试覆盖率**: >80%
- **代码重复率**: <5%
- **圈复杂度**: <10 per function

### 质量指标
- **静态检查**: clippy零警告
- **格式化**: rustfmt通过
- **文档覆盖率**: >90%
- **性能回归**: <5%

### 维护性指标
- **模块耦合度**: 低
- **代码可读性**: 高
- **扩展性**: 新功能易于添加
- **测试友好性**: Mock和测试工具完善

## 风险控制

### 技术风险
1. **兼容性风险**: 保持向后兼容，使用特性标记
2. **性能风险**: 建立性能基准，监控关键指标
3. **复杂性风险**: 渐进式重构，避免大规模重写

### 进度风险
1. **时间风险**: 分阶段交付，确保每个阶段独立可用
2. **质量风险**: 持续集成，自动化测试
3. **资源风险**: 合理分配资源，避免关键路径阻塞

## 长期维护策略

### 1. 持续集成
- 建立CI/CD流程
- 自动化测试和部署
- 代码审查流程

### 2. 监控和告警
- 编译状态监控
- 性能指标监控
- 错误率监控

### 3. 文档维护
- 自动化文档生成
- 变更日志管理
- 用户反馈收集

---

本架构调整建议旨在从根本上解决当前项目的技术债务，提高代码质量和可维护性。通过系统性的架构改进，项目将具备更好的扩展性、稳定性和开发效率。