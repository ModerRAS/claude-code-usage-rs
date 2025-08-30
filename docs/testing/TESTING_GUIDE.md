# ccusage-rs 测试文档

## 测试架构概述

ccusage-rs 项目采用分层测试架构，确保代码质量和系统稳定性：

- **单元测试** (Unit Tests) - 测试单个函数和模块
- **集成测试** (Integration Tests) - 测试模块间交互
- **端到端测试** (E2E Tests) - 测试完整用户工作流
- **性能测试** (Performance Tests) - 测试系统性能和扩展性
- **错误处理测试** (Error Tests) - 测试错误场景和恢复
- **基准测试** (Benchmarks) - 性能基准和回归测试

## 测试目录结构

```
tests/
├── common/                    # 公共测试工具
│   ├── mod.rs                # 测试模块导出
│   ├── test_utils.rs         # 测试工具函数
│   ├── test_data.rs          # 测试数据生成
│   ├── mock_services.rs      # 模拟服务
│   └── assertions.rs         # 自定义断言
├── unit_tests.rs             # 单元测试
├── integration_tests.rs      # 集成测试
├── e2e_tests.rs              # 端到端测试
├── performance_tests.rs      # 性能测试
└── error_tests.rs            # 错误处理测试

src/
├── data/tests.rs             # 数据模块测试
├── error/tests.rs            # 错误处理测试
└── cli/tests.rs              # CLI模块测试

benches/
├── performance_benchmarks.rs # 完整性能基准测试
├── quick_benchmarks.rs       # 快速基准测试
└── README.md                 # 基准测试说明
```

## 运行测试

### 运行所有测试
```bash
cargo test
```

### 运行特定测试类型
```bash
# 单元测试
cargo test unit_tests

# 集成测试
cargo test integration_tests

# 端到端测试
cargo test e2e_tests

# 性能测试
cargo test performance_tests

# 错误处理测试
cargo test error_tests
```

### 运行基准测试
```bash
# 完整基准测试
cargo bench --bench performance_benchmarks

# 快速基准测试
cargo bench --bench quick_benchmarks

# 特定基准测试组
cargo bench --bench performance_benchmarks -- data_loading
```

### 测试覆盖率
```bash
# 生成覆盖率报告
cargo tarpaulin --out Html

# 覆盖率报告保存在 tarpaulin-report.html
```

## 测试工具和依赖

### 核心测试框架
- **tokio-test** - 异步测试支持
- **pretty_assertions** - 美化的断言输出
- **criterion** - 基准测试框架
- **tarpaulin** - 代码覆盖率工具

### 测试数据生成
- **fake** - 生成真实测试数据
- **rand** - 随机数生成
- **tempfile** - 临时文件管理

### 模拟和隔离
- **mockall** - 通用模拟框架
- **mockito** - HTTP模拟
- **wiremock** - 网络模拟

### CLI测试
- **assert_cmd** - CLI命令测试
- **predicates** - 断言谓词

### 性能监控
- **memory-stats** - 内存使用统计
- **divan** - 轻量级基准测试

## 测试数据生成

### 使用TestDataGenerator
```rust
use tests::common::test_data::TestDataGenerator;

let mut generator = TestDataGenerator::new();
let record = generator.generate_usage_record();
let records = generator.generate_usage_records_for_date_range(
    start_date,
    end_date,
    100  // 每天记录数
);
```

### 支持的数据类型
- **UsageRecord** - 使用记录
- **配置数据** - 各种配置场景
- **错误数据** - 各种错误场景
- **边界数据** - 边界和异常情况

## 测试模式

### 单元测试模式
```rust
#[tokio::test]
async fn test_data_loader_creation() {
    let loader = DataLoader::new();
    assert!(loader.is_ok());
}
```

### 集成测试模式
```rust
#[tokio::test]
async fn test_full_data_pipeline() {
    // 1. 准备测试数据
    let test_data = create_test_data();
    
    // 2. 执行完整流程
    let result = process_full_pipeline(test_data).await;
    
    // 3. 验证结果
    assert!(result.is_ok());
    assert!(result.unwrap().len() > 0);
}
```

### 端到端测试模式
```rust
#[tokio::test]
async fn test_cli_analyze_command() {
    // 1. 准备环境
    let temp_dir = create_test_environment();
    
    // 2. 执行CLI命令
    let mut cmd = Command::cargo_bin("ccusage-rs").unwrap();
    cmd.args(&[
        "--data-dir", temp_dir.path().to_str().unwrap(),
        "analyze",
        "--start-date", "2024-01-01",
        "--end-date", "2024-12-31"
    ]);
    
    // 3. 验证结果
    cmd.assert().success();
}
```

### 性能测试模式
```rust
#[tokio::test]
async fn test_large_dataset_processing() {
    let dataset = generate_large_dataset(10000);
    
    let (result, duration) = measure_execution_time(|| {
        process_dataset(&dataset)
    });
    
    assert!(result.is_ok());
    assert!(duration.as_millis() < 1000); // 1秒内完成
}
```

## 自定义断言

### 使用自定义断言
```rust
use tests::common::assertions::*;

record.assert_valid_usage_record();
record.assert_cost_positive();
record.assert_tokens_positive();
```

### 可用断言
- `assert_valid_usage_record()` - 验证使用记录完整性
- `assert_cost_positive()` - 验证成本为正数
- `assert_tokens_positive()` - 验证令牌数为正数
- `assert_date_in_range()` - 验证日期在指定范围内

## 模拟服务

### DataLoader模拟
```rust
use tests::common::mock_services::*;
use mockall::predicate::*;

let mut mock_loader = MockDataLoader::new();
mock_loader.expect_load_usage_data()
    .returning(|| Ok(vec![test_record]));

// 使用模拟进行测试
let result = some_function_using_loader(&mock_loader).await;
assert!(result.is_ok());
```

### HTTP服务模拟
```rust
use mockito::{mock, Server};

let mut server = Server::new();
let mock = server.mock("GET", "/api/data")
    .with_status(200)
    .with_header("content-type", "application/json")
    .with_body(r#"{"data": [{"id": "1", "cost": 10.0}]}"#)
    .create();

// 测试HTTP客户端
let client = HttpClient::new(server.url());
let result = client.get_data().await;
assert!(result.is_ok());
```

## 测试最佳实践

### 1. 测试命名约定
- 使用描述性的测试名称
- 遵循 `test_what_happens_when_expected_behavior` 模式
- 对于错误测试，使用 `test_what_happens_when_error_condition`

### 2. 测试组织
- 按功能模块组织测试
- 使用 `mod` 结构组织相关测试
- 共享测试设置和清理代码

### 3. 异步测试
- 使用 `#[tokio::test]` 进行异步测试
- 正确处理异步生命周期
- 避免在测试中阻塞

### 4. 错误测试
- 测试所有错误路径
- 验证错误消息的准确性
- 测试错误恢复机制

### 5. 性能测试
- 设置合理的性能阈值
- 考虑不同数据规模
- 监控内存使用

## 测试覆盖率目标

### 覆盖率要求
- **整体覆盖率**: ≥ 80%
- **核心模块**: ≥ 90%
- **错误处理**: ≥ 85%
- **CLI模块**: ≥ 85%

### 覆盖率检查
```bash
# 生成覆盖率报告
cargo tarpaulin --out Html --output-dir coverage/

# 检查特定模块覆盖率
cargo tarpaulin --out Html --src-files src/data/*

# 持续集成检查
cargo tarpaulin --ignore-tests --out Xml --output-dir coverage/
```

## CI/CD集成

### GitHub Actions配置
```yaml
name: Test Suite
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Run tests
        run: cargo test --verbose
        
      - name: Run benchmarks
        run: cargo bench --bench quick_benchmarks
        
      - name: Generate coverage
        run: cargo tarpaulin --out Xml
        
      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

### 性能回归检测
```yaml
- name: Check performance regression
  run: |
    cargo bench --bench quick_benchmarks -- --save-baseline main
    # 比较基准测试结果
    cargo bench --bench quick_benchmarks -- --baseline main
```

## 测试数据管理

### 测试数据文件
- 存储在 `tests/fixtures/` 目录
- 使用版本控制管理测试数据
- 定期更新测试数据以反映真实场景

### 临时测试数据
- 使用 `tempfile` 创建临时目录和文件
- 确保测试后清理临时数据
- 避免测试之间的相互影响

## 常见测试问题

### 1. 异步测试死锁
- 问题：测试在等待异步操作时死锁
- 解决：使用 `tokio::test` 并正确处理异步生命周期

### 2. 测试环境隔离
- 问题：测试之间相互影响
- 解决：使用临时目录和独立配置

### 3. 性能测试不稳定
- 问题：基准测试结果波动大
- 解决：多次运行取平均值，在安静环境中测试

### 4. 模拟设置复杂
- 问题：模拟服务配置过于复杂
- 解决：创建模拟构建器模式简化设置

## 测试维护

### 定期维护任务
1. 更新测试数据以反映API变化
2. 检查测试覆盖率
3. 更新基准测试阈值
4. 清理过时的测试

### 测试重构
- 当代码结构变化时更新测试
- 保持测试与生产代码同步
- 重构测试以提高可维护性

## 贡献指南

### 添加新测试
1. 遵循现有测试模式
2. 使用公共测试工具
3. 确保测试覆盖率和质量
4. 添加必要的文档

### 调试测试
- 使用 `cargo test -- --nocapture` 查看详细输出
- 使用 `dbg!` 宏进行调试
- 使用日志记录器记录测试执行过程

## 性能基准

### 关键性能指标
- **数据加载**: 10000条记录 < 100ms
- **数据处理**: 1000条记录 < 10ms
- **CLI响应**: 命令执行 < 1s
- **内存使用**: 10000条记录 < 10MB

### 性能测试执行
```bash
# 运行完整性能测试
cargo bench --bench performance_benchmarks

# 检查性能回归
cargo bench --bench quick_benchmarks -- --compare-with main
```

## 总结

ccusage-rs 项目的测试架构提供了全面的测试覆盖，确保代码质量和系统稳定性。通过分层测试策略、丰富的测试工具和自动化集成，项目能够快速迭代并保持高质量的代码标准。

测试不仅验证功能正确性，还确保性能、可靠性和用户体验。定期运行测试和基准测试有助于及时发现和解决问题，保持项目的健康发展。