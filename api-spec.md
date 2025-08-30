# ccusage-rs API 规范 (简化版)

## 概述

本文档定义了 ccusage-rs 的**简化API规范**，专注于核心功能，确保与原版 ccusage 的兼容性，同时提供清晰的错误处理和可测试的接口。

## 设计原则

1. **简单性**: 只实现核心功能，减少不必要的复杂性
2. **一致性**: 统一的错误处理和数据格式
3. **可测试性**: 所有接口都设计为可单元测试
4. **性能**: 针对大数据集进行优化
5. **兼容性**: 与原版 ccusage 的数据格式兼容

## 命令行接口规范 (简化版)

### 全局选项 (简化版)

| 选项 | 类型 | 默认值 | 描述 |
|------|------|--------|------|
| `--json` | flag | false | 以 JSON 格式输出 |
| `--mode` | enum | auto | 成本计算模式 (auto/calculate/display) |
| `--since` | date | - | 开始日期过滤 (YYYY-MM-DD) |
| `--until` | date | - | 结束日期过滤 (YYYY-MM-DD) |
| `--timezone` | string | UTC | 时区设置 |
| `--locale` | string | en | 地区设置 |
| `--offline` | flag | false | 离线模式 |
| `--compact` | flag | false | 紧凑显示模式 |
| `--debug` | flag | false | 调试模式 |
| `--help` | flag | false | 显示帮助信息 |
| `--version` | flag | false | 显示版本信息 |

### 核心命令 (仅实现必要功能)

#### daily 命令
```bash
ccusage-rs daily [OPTIONS]
```

**选项**:
- `--date` <date>: 指定日期 (YYYY-MM-DD)
- `--project` <name>: 按项目过滤
- `--breakdown` / `-b`: 显示模型详细分解

**示例**:
```bash
# 基本日报
ccusage-rs daily

# 指定日期
ccusage-rs daily --date 2025-01-01

# 按项目过滤
ccusage-rs daily --project myproject

# 显示详细分解
ccusage-rs daily --breakdown

# JSON 输出
ccusage-rs daily --json
```

#### monthly 命令
```bash
ccusage-rs monthly [OPTIONS]
```

**选项**:
- `--year` <year>: 指定年份
- `--month` <month>: 指定月份 (1-12)
- `--breakdown` / `-b`: 显示模型详细分解

**示例**:
```bash
# 基本月报
ccusage-rs monthly

# 指定年月
ccusage-rs monthly --year 2025 --month 1

# 当前年份
ccusage-rs monthly --year 2025

# 显示详细分解
ccusage-rs monthly --breakdown
```

#### session 命令
```bash
ccusage-rs session [OPTIONS]
```

**选项**:
- `--session-id` <id>: 显示指定会话
- `--list`: 列出所有会话

**示例**:
```bash
# 列出所有会话
ccusage-rs session --list

# 显示指定会话
ccusage-rs session --session-id session_123

# JSON 输出
ccusage-rs session --list --json
```

#### blocks 命令
```bash
ccusage-rs blocks [OPTIONS]
```

**选项**:
- `--active`: 只显示活跃的计费块
- `--token-limit` <limit>: 设置 token 限制

**示例**:
```bash
# 基本计费块
ccusage-rs blocks

# 只显示活跃块
ccusage-rs blocks --active

# 设置 token 限制
ccusage-rs blocks --token-limit 500000
```

## 数据模型规范 (简化版)

### 输入数据格式 (JSONL) - 与原版兼容

#### 单条使用记录
```json
{
  "timestamp": "2025-01-01T12:00:00Z",
  "model": "claude-3-sonnet-20240229",
  "usage": {
    "input_tokens": 1000,
    "output_tokens": 500,
    "cache_creation_input_tokens": 200,
    "cache_read_input_tokens": 100
  },
  "cost_usd": 0.015,
  "session_id": "session_123",
  "project_path": "/path/to/project",
  "request_id": "req_123",
  "message_id": "msg_123"
}
```

**字段说明**:
- `timestamp`: ISO 8601 格式的时间戳 (必需)
- `model`: Claude 模型名称 (必需)
- `usage`: Token 使用统计 (必需)
- `cost_usd`: 预计算的成本 (可选)
- `session_id`: 会话标识符 (可选)
- `project_path`: 项目路径 (可选)
- `request_id`: 请求标识符 (必需)
- `message_id`: 消息标识符 (必需)

### 输出数据格式 (简化版)

#### 日报输出 (JSON)
```json
{
  "daily": [
    {
      "date": "2025-01-01",
      "inputTokens": 15000,
      "outputTokens": 8000,
      "cacheCreationTokens": 2000,
      "cacheReadTokens": 1000,
      "totalTokens": 26000,
      "totalCost": 0.39,
      "modelsUsed": ["claude-3-sonnet-20240229"]
    }
  ],
  "totals": {
    "inputTokens": 15000,
    "outputTokens": 8000,
    "cacheCreationTokens": 2000,
    "cacheReadTokens": 1000,
    "totalTokens": 26000,
    "totalCost": 0.39,
    "days": 1
  }
}
```

#### 月报输出 (JSON)
```json
{
  "monthly": [
    {
      "month": "2025-01",
      "inputTokens": 450000,
      "outputTokens": 240000,
      "cacheCreationTokens": 60000,
      "cacheReadTokens": 30000,
      "totalTokens": 780000,
      "totalCost": 11.70,
      "modelsUsed": ["claude-3-sonnet-20240229", "claude-3-opus-20240229"]
    }
  ],
  "totals": {
    "inputTokens": 450000,
    "outputTokens": 240000,
    "cacheCreationTokens": 60000,
    "cacheReadTokens": 30000,
    "totalTokens": 780000,
    "totalCost": 11.70,
    "months": 1
  }
}
```

#### 会话报告输出 (JSON)
```json
{
  "sessions": [
    {
      "sessionId": "session_123",
      "startTime": "2025-01-01T10:00:00Z",
      "endTime": "2025-01-01T12:00:00Z",
      "duration": 7200,
      "inputTokens": 15000,
      "outputTokens": 8000,
      "totalTokens": 23000,
      "totalCost": 0.345,
      "projectPath": "/path/to/project",
      "modelsUsed": ["claude-3-sonnet-20240229"]
    }
  ],
  "totals": {
    "inputTokens": 15000,
    "outputTokens": 8000,
    "totalTokens": 23000,
    "totalCost": 0.345,
    "sessions": 1
  }
}
```

#### 计费块输出 (JSON)
```json
{
  "blocks": [
    {
      "blockId": "block_123",
      "startTime": "2025-01-01T10:00:00Z",
      "endTime": "2025-01-01T15:00:00Z",
      "duration": 18000,
      "inputTokens": 150000,
      "outputTokens": 80000,
      "totalTokens": 230000,
      "totalCost": 3.45,
      "isActive": false,
      "gapBefore": null,
      "gapAfter": 3600
    }
  ],
  "totals": {
    "inputTokens": 150000,
    "outputTokens": 80000,
    "totalTokens": 230000,
    "totalCost": 3.45,
    "blocks": 1
  }
}
```

## 配置文件规范 (简化版)

### 配置文件格式 (JSON)
```json
{
  "dataDir": "~/.claude",
  "costMode": "auto",
  "timezone": "UTC",
  "locale": "en",
  "offline": false,
  "compact": false,
  "debug": false
}
```

**配置字段说明**:
- `dataDir`: Claude Code 数据目录路径
- `costMode`: 成本计算模式 (auto/calculate/display)
- `timezone`: 时区设置
- `locale`: 地区设置
- `offline`: 离线模式
- `compact`: 紧凑显示模式
- `debug`: 调试模式

### 环境变量 (简化版)

| 环境变量 | 描述 |
|----------|------|
| `CLAUDE_CONFIG_DIR` | Claude Code 配置目录路径 |
| `CCUSAGE_DATA_DIR` | ccusage 数据目录路径 |
| `CCUSAGE_COST_MODE` | 默认成本计算模式 |
| `CCUSAGE_TIMEZONE` | 默认时区 |
| `CCUSAGE_LOCALE` | 默认地区 |
| `CCUSAGE_OFFLINE` | 离线模式 |
| `CCUSAGE_DEBUG` | 调试模式 |

## 错误处理规范 (简化版)

### 统一错误格式

#### 命令行错误输出
```bash
# 格式错误
Error: Invalid date format. Expected YYYY-MM-DD

# 文件错误
Error: Data file not found: /path/to/file.jsonl

# 权限错误
Error: Permission denied: /path/to/file.jsonl

# 数据错误
Error: Invalid data format in line 123
```

#### JSON 错误输出
```json
{
  "error": {
    "code": "INVALID_DATE_FORMAT",
    "message": "Invalid date format. Expected YYYY-MM-DD",
    "details": {
      "input": "2025/01/01",
      "expected": "YYYY-MM-DD"
    }
  }
}
```

### 错误代码规范

#### 命令行错误代码
| 代码 | 描述 |
|------|------|
| 0 | 成功 |
| 1 | 一般错误 |
| 2 | 参数错误 |
| 3 | 数据文件错误 |
| 4 | 网络错误 |
| 5 | 配置错误 |
| 6 | 权限错误 |

#### 错误类型映射
| 错误类型 | 代码 | HTTP 状态码 |
|----------|------|------------|
| `Config` | 5 | 400 |
| `DataLoading` | 3 | 404 |
| `FileSystem` | 3 | 404 |
| `Parse` | 2 | 400 |
| `Io` | 3 | 500 |
| `Network` | 4 | 503 |
| `Validation` | 2 | 400 |

## 数据验证规范 (简化版)

### 日期格式验证
- 支持 ISO 8601 格式: `YYYY-MM-DD`
- 支持相对日期: `today`, `yesterday`
- 错误格式示例: `2025/01/01`, `01-01-2025`

### 路径验证
- 绝对路径: `/path/to/data`
- 相对路径: `./data`
- 用户目录: `~/.claude`
- 错误路径示例: `../data`, `~/../data`

### 数据完整性验证
- JSONL 格式验证 (每行一个有效的 JSON 对象)
- 必需字段检查 (timestamp, model, usage, request_id, message_id)
- 数据类型验证 (timestamp 为 ISO 8601, tokens 为正整数)
- 范围检查 (token 数量合理, cost 为非负数)

## 性能指标规范 (简化版)

### 响应时间指标
- 冷启动时间: < 2 秒
- 热启动时间: < 1 秒
- 数据加载时间: < 5 秒 (100MB 数据)
- 报告生成时间: < 2 秒

### 内存使用指标
- 基础内存使用: < 50MB
- 峰值内存使用: < 100MB
- 大数据集处理: < 200MB
- 内存泄漏: 无

### 数据处理指标
- 支持最大文件大小: 1GB
- 支持最大记录数: 1,000,000
- 流式处理: 支持
- 增量处理: 支持

## 兼容性规范 (简化版)

### 命令兼容性
- 支持所有原版 ccusage 核心命令 (daily, monthly, session, blocks)
- 支持原版参数别名 (-b, -i, -p 等)
- 输出格式与原版兼容
- 错误处理行为一致

### 数据兼容性
- 支持原版 JSONL 数据格式
- 处理缺失字段情况 (使用默认值)
- 向后兼容数据版本
- 迁移工具支持 (未来)

### 配置兼容性
- 支持原版配置文件 (部分字段)
- 配置字段映射
- 默认值兼容
- 配置验证增强

## 测试接口规范 (简化版)

### 单元测试接口
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_data() -> Vec<UsageEntry> {
        // 创建测试数据
    }
    
    fn create_test_config() -> Config {
        // 创建测试配置
    }
    
    #[test]
    fn test_daily_aggregation() {
        // 测试日报聚合
    }
    
    #[tokio::test]
    async fn test_data_loading() {
        // 测试数据加载
    }
}
```

### 集成测试接口
```rust
#[tokio::test]
async fn test_full_workflow() {
    // 完整工作流测试
}

#[tokio::test]
async fn test_command_line_interface() {
    // 命令行接口测试
}
```

## 安全规范 (简化版)

### 输入安全
- 参数验证和清理
- 路径遍历防护 (防止 `../../../etc/passwd`)
- 命令注入防护
- 数据类型验证

### 数据安全
- 文件权限检查
- 敏感信息保护 (不记录 API 密钥等)
- 安全的数据处理
- 错误信息安全 (不泄露敏感信息)

### 网络安全 (可选功能)
- HTTPS 强制使用
- 证书验证
- 请求超时控制 (30 秒)
- 代理安全配置

## 扩展接口规范 (简化版)

### 插件接口 (未来)
```rust
pub trait Plugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn process_data(&self, data: &mut Vec<UsageEntry>) -> Result<()>;
}
```

### 自定义输出格式 (未来)
```rust
pub trait OutputFormatter {
    fn format(&self, report: &Report) -> Result<String>;
    fn file_extension(&self) -> &str;
}
```

## 文档规范 (简化版)

### API 文档
- 接口描述完整
- 参数说明详细
- 示例代码丰富
- 错误处理说明

### 用户文档
- 安装指南
- 使用说明
- 配置说明
- 故障排除

### 开发文档
- 架构设计
- 代码规范
- 测试指南
- 贡献指南

## 总结

ccusage-rs 的简化 API 规范确保了：

1. **简单性**: 只实现核心功能，减少复杂性
2. **一致性**: 统一的错误处理和数据格式
3. **可测试性**: 所有接口都设计为可单元测试
4. **性能**: 针对大数据集进行优化
5. **兼容性**: 与原版 ccusage 的数据格式和命令行接口兼容
6. **安全性**: 基本的安全措施和输入验证
7. **可扩展性**: 为未来功能扩展预留接口

该规范为系统的实现和维护提供了明确的指导，确保项目能够满足核心需求，同时保持高质量和可维护性。