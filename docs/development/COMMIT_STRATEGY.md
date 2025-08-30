# ccusage-rs 项目修复Commit策略

## 概述

本文档定义了ccusage-rs项目修复工作的commit策略，确保修复过程可追踪、可回滚，并符合最佳实践。

## Commit原则

### 1. 原子性Commit
- 每个commit应该是一个逻辑上独立的变更
- 单个commit应该能够独立编译和测试
- 避免将多个不相关的变更混合在一个commit中

### 2. 语义化Commit消息
- 使用清晰的commit消息格式
- 包含必要的技术细节
- 便于代码审查和问题追踪

### 3. 渐进式修复
- 从基础修复开始，逐步推进到复杂修复
- 确保每个阶段都有可验证的成果
- 支持部分回滚而不影响整体进度

## Commit消息格式

### 标准格式
```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### 类型说明
- `fix`: 修复编译错误或bug
- `feat`: 新功能
- `docs`: 文档更新
- `style`: 代码格式化
- `refactor`: 重构代码
- `test`: 测试相关
- `chore`: 构建或工具变更

### 作用域说明
- `deps`: 依赖管理
- `chrono`: 时间处理
- `types`: 类型系统
- `config`: 配置管理
- `error`: 错误处理
- `cli`: 命令行界面
- `tests`: 测试代码

## 分阶段Commit计划

### 阶段1: 基础依赖修复 (Day 1)

#### Commit 1: 依赖版本更新
```bash
git commit -m "fix(deps): update dependency versions for compatibility

- Update chrono to ~0.4.38 with proper feature flags
- Update dirs to 5.0.1, which to 7.0.0, uuid to 1.11.0
- Add missing dependencies: anyhow, thiserror
- Use tilde constraints for better compatibility
- Prepare for Chrono 0.4+ API changes

Fixes #1
```

#### Commit 2: 依赖配置优化
```bash
git commit -m "fix(deps): optimize dependency configuration

- Separate development dependencies from production
- Add version pinning for critical dependencies
- Optimize feature flags compilation
- Add cargo-udeps configuration for unused dependency detection
- Improve build performance with selective feature compilation

Fixes #2
```

#### Commit 3: 编译基础修复
```bash
git commit -m "fix(build): resolve basic compilation issues

- Fix module declaration and import statements
- Resolve circular dependency issues
- Add missing trait implementations
- Fix visibility issues for public APIs
- Ensure all modules compile independently

Fixes #3
```

### 阶段2: Chrono API修复 (Day 2)

#### Commit 4: Chrono时间构造函数修复
```bash
git commit -m "fix(chrono): update time construction methods

- Replace from_ymd() with from_ymd_opt().unwrap()
- Replace from_hms() with from_hms_opt().unwrap()
- Update NaiveDateTime construction patterns
- Add safe wrappers for date/time creation
- Add validation for date/time ranges

Fixes #10, #11, #12
```

#### Commit 5: Chrono时间计算方法修复
```bash
git commit -m "fix(chrono): fix time calculation methods

- Replace pred() with pred_opt() for date operations
- Fix weekday() method usage and return types
- Update duration calculations for Chrono 0.4+
- Add safe date arithmetic operations
- Implement proper timezone handling

Fixes #13, #14, #15
```

#### Commit 6: 时间处理功能重构
```bash
git commit -m "refactor(chrono): introduce temporal abstraction layer

- Add TemporalOperations trait for time abstraction
- Implement SafeDate wrapper for type safety
- Create centralized date parsing and formatting
- Add time zone safety measures
- Improve error handling for time operations

Fixes #16, #17
```

### 阶段3: 类型安全修复 (Day 3)

#### Commit 7: Option类型处理修复
```bash
git commit -m "fix(types): improve Option type handling

- Fix Option<T> unwrap() usage patterns
- Replace direct field access with safe methods
- Add proper error propagation for Option types
- Implement safer date/time access patterns
- Add validation for optional field usage

Fixes #20, #21, #22
```

#### Commit 8: 强类型封装
```bash
git commit -m "feat(types): introduce strong type wrappers

- Add Amount type for monetary values
- Add TokenCount type for token quantities
- Implement safe type conversions
- Add validation for type invariants
- Improve type safety throughout the codebase

Fixes #25, #26
```

#### Commit 9: 错误处理统一
```bash
git commit -m "fix(error): standardize error handling

- Unify error types using thiserror
- Implement consistent error propagation
- Add error context and recovery strategies
- Improve error messages and user experience
- Add error handling best practices

Fixes #30, #31, #32
```

### 阶段4: 架构改进 (Day 4-5)

#### Commit 10: 配置管理重构
```bash
git commit -m "refactor(config): restructure configuration management

- Introduce AppConfig struct for centralized config
- Implement safe configuration loading and saving
- Add configuration validation
- Improve configuration error handling
- Add configuration versioning support

Fixes #35, #36
```

#### Commit 11: 模块化重构
```bash
git commit -m "refactor(modules): improve module structure

- Reorganize code into logical layers (core, infrastructure, adapters)
- Implement proper dependency injection
- Add module interfaces and contracts
- Improve code organization and maintainability
- Reduce coupling between modules

Fixes #40, #41
```

#### Commit 12: 访问控制修复
```bash
git commit -m "fix(modules): fix access control issues

- Replace private field access with public methods
- Implement proper encapsulation patterns
- Add safe getters and setters
- Improve module visibility rules
- Fix configuration access patterns

Fixes #45, #46, #47
```

### 阶段5: 测试和验证 (Day 5-7)

#### Commit 13: 测试框架完善
```bash
git commit -m "test(tests): comprehensive test framework

- Add unit tests for core functionality
- Implement integration tests for module interactions
- Add system tests for end-to-end scenarios
- Include performance benchmarks
- Add test utilities and mocking framework

Fixes #50, #51
```

#### Commit 14: CI/CD配置
```bash
git commit -m "ci(ci): continuous integration setup

- Add GitHub Actions workflow for automated testing
- Configure multi-platform testing
- Add code quality checks (fmt, clippy)
- Implement test coverage reporting
- Add automated deployment pipeline

Fixes #55, #56
```

#### Commit 15: 文档和最终验证
```bash
git commit -m "docs(docs): update documentation and final validation

- Update API documentation with new types and methods
- Add architecture documentation
- Include usage examples and best practices
- Final validation of all functionality
- Prepare for release

Fixes #60, #61
```

## 分支策略

### 开发分支
```bash
# 创建功能分支
git checkout -b fix/compilation-errors main

# 按阶段创建里程碑分支
git checkout -b fix/phase1-dependencies fix/compilation-errors
git checkout -b fix/phase2-chrono fix/compilation-errors
git checkout -b fix/phase3-types fix/compilation-errors
git checkout -b fix/phase4-architecture fix/compilation-errors
git checkout -b fix/phase5-testing fix/compilation-errors
```

### 合并策略
```bash
# 阶段完成后合并到主修复分支
git checkout fix/compilation-errors
git merge --no-ff fix/phase1-dependencies
git merge --no-ff fix/phase2-chrono
git merge --no-ff fix/phase3-types
git merge --no-ff fix/phase4-architecture
git merge --no-ff fix/phase5-testing

# 最终合并到主分支
git checkout main
git merge --no-ff fix/compilation-errors
```

## 质量检查

### 每次Commit前的检查
```bash
# 编译检查
cargo check
cargo build --release

# 格式化检查
cargo fmt --all -- --check

# 代码质量检查
cargo clippy -- -D warnings

# 测试检查
cargo test
cargo test --all-features
cargo test --no-default-features
```

### 自动化质量门禁
```yaml
# .github/workflows/quality.yml
name: Quality Checks

on: [push, pull_request]

jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        components: rustfmt, clippy
    
    - name: Check formatting
      run: cargo fmt --all -- --check
    
    - name: Run clippy
      run: cargo clippy -- -D warnings
    
    - name: Run tests
      run: cargo test --all-features
```

## 回滚策略

### 单个Commit回滚
```bash
# 回滚特定commit
git revert <commit-hash>

# 或使用reset（如果commit尚未推送）
git reset --hard HEAD~1
```

### 阶段回滚
```bash
# 回滚整个阶段
git revert --no-commit <start-commit>..<end-commit>
git commit -m "revert: rollback phase X due to issues"

# 或使用分支重置
git checkout fix/phaseX-previous
git branch -D fix/phaseX-current
git checkout -b fix/phaseX-current fix/phaseX-previous
```

## 发布准备

### 版本标记
```bash
# 标记修复版本
git tag -a v0.1.0-fix -m "Fix compilation errors and improve architecture"

# 推送标签
git push origin v0.1.0-fix
```

### 发布说明
```markdown
# Release v0.1.0-fix

## Fixed Issues
- Fixed 71 compilation errors
- Resolved Chrono 0.4+ API compatibility issues
- Improved type safety and error handling
- Enhanced architecture and maintainability

## Breaking Changes
- Updated dependency versions (chrono 0.4.38+)
- Changed time construction API (now using _opt variants)
- Introduced strong type wrappers (Amount, TokenCount)

## Migration Guide
1. Update Cargo.toml dependencies
2. Replace time construction calls with safe variants
3. Update type usage to new strong types
4. Follow new error handling patterns
```

## 监控和维护

### 后续监控
```bash
# 监控编译状态
cargo check --target=x86_64-unknown-linux-gnu
cargo check --target=x86_64-apple-darwin
cargo check --target=x86_64-pc-windows-msvc

# 监控测试状态
cargo test --release
cargo test --all-features

# 监控性能
cargo bench
```

### 持续改进
- 收集用户反馈
- 监控错误报告
- 定期更新依赖
- 持续优化性能

---

此commit策略确保修复工作的可追踪性、可维护性和质量。每个commit都有明确的目标和范围，便于代码审查和问题追踪。策略支持渐进式修复和灵活的回滚机制。