# 基准测试配置文件

## 基准测试说明

本目录包含两个基准测试套件：

1. **performance_benchmarks.rs** - 完整的性能基准测试
   - 测试所有核心功能的性能表现
   - 包含大数据集测试（用于评估扩展性）
   - 测试内存使用和并发处理能力
   - 运行时间较长，用于深度性能分析

2. **quick_benchmarks.rs** - 快速基准测试
   - 关键功能的快速性能检查
   - 小数据集测试（用于CI/CD集成）
   - 内存泄漏检测
   - 运行时间短，适合频繁运行

## 运行基准测试

### 运行完整基准测试
```bash
cargo bench --bench performance_benchmarks
```

### 运行快速基准测试
```bash
cargo bench --bench quick_benchmarks
```

### 运行特定基准测试组
```bash
cargo bench --bench performance_benchmarks -- data_loading
cargo bench --bench performance_benchmarks -- data_processing
cargo bench --bench performance_benchmarks -- memory_usage
```

### 生成HTML报告
```bash
cargo bench --bench performance_benchmarks -- --output-format html
```

## 基准测试组说明

### data_loading
- **load_from_memory**: 从内存加载数据的性能
- **load_from_file**: 从文件加载数据的性能
- 测试不同数据集大小：100, 1000, 5000, 10000条记录

### data_processing
- **filter_by_date**: 按日期过滤数据的性能
- **aggregate_by_model**: 按模型聚合数据的性能
- **sort_by_cost**: 按成本排序数据的性能
- 测试不同数据集大小：1000, 5000, 10000条记录

### memory_usage
- **large_dataset_memory**: 大数据集内存占用测试
- **processing_memory**: 数据处理过程中的内存使用测试

### concurrent_processing
- **concurrent_filter**: 并发过滤处理的性能
- 测试不同线程数：1, 2, 4, 8线程

### cli_processing
- **argument_parsing**: CLI参数解析性能
- **config_loading**: 配置文件加载性能

### serialization
- **json_serialize**: JSON序列化性能
- **json_deserialize**: JSON反序列化性能
- 测试不同数据集大小：100, 1000, 5000条记录

### error_handling
- **error_creation**: 错误创建和传播性能
- **error_conversion**: 错误转换性能

## 性能目标

### 数据加载
- 100条记录： < 1ms
- 1000条记录： < 10ms
- 10000条记录： < 100ms

### 数据处理
- 过滤操作： < 5ms/1000条记录
- 聚合操作： < 10ms/1000条记录
- 排序操作： < 20ms/1000条记录

### 内存使用
- 10000条记录： < 10MB内存占用
- 处理过程中内存增长： < 20%

### CLI操作
- 参数解析： < 1ms
- 配置加载： < 5ms

## 基准测试结果分析

基准测试结果会保存在 `target/criterion/` 目录中，包含：

- 详细的性能统计数据
- 图表可视化
- 与历史结果的比较
- 内存使用分析

## CI/CD集成

在CI/CD管道中使用快速基准测试来检测性能回归：

```yaml
- name: Run benchmarks
  run: cargo bench --bench quick_benchmarks
  
- name: Check performance regression
  run: |
    # 检查基准测试结果是否在可接受范围内
    python scripts/check_benchmarks.py
```

## 性能优化建议

1. **数据加载优化**：
   - 使用内存映射文件
   - 实现增量加载
   - 添加数据缓存

2. **数据处理优化**：
   - 使用并行处理
   - 优化算法复杂度
   - 减少内存分配

3. **内存优化**：
   - 使用对象池
   - 实现流式处理
   - 优化数据结构

4. **并发优化**：
   - 调整线程池大小
   - 使用异步I/O
   - 减少锁竞争

## 注意事项

1. 基准测试应在安静的环境中运行，避免其他进程干扰
2. 多次运行取平均值以获得稳定结果
3. 定期更新基准测试以反映实际使用场景
4. 监控系统资源使用情况，避免测试结果受到系统限制