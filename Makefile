# ccusage-rs Makefile
# 用于简化测试和构建流程

.PHONY: help test test-unit test-integration test-e2e test-performance test-error test-all
.PHONY: bench bench-quick bench-full coverage format clippy clean build
.PHONY: docs security ci-check install-tools

# 默认目标
help:
	@echo "可用的命令："
	@echo ""
	@echo "测试命令："
	@echo "  test-unit          - 运行单元测试"
	@echo "  test-integration   - 运行集成测试"
	@echo "  test-e2e          - 运行端到端测试"
	@echo "  test-performance   - 运行性能测试"
	@echo "  test-error        - 运行错误处理测试"
	@echo "  test-all          - 运行所有测试"
	@echo ""
	@echo "基准测试："
	@echo "  bench-quick       - 运行快速基准测试"
	@echo "  bench-full        - 运行完整基准测试"
	@echo ""
	@echo "代码质量："
	@echo "  format            - 格式化代码"
	@echo "  clippy            - 运行clippy检查"
	@echo "  coverage          - 生成测试覆盖率报告"
	@echo "  security          - 运行安全检查"
	@echo ""
	@echo "构建："
	@echo "  build             - 构建项目"
	@echo "  build-release     - 构建发布版本"
	@echo "  clean             - 清理构建文件"
	@echo ""
	@echo "文档："
	@echo "  docs              - 生成文档"
	@echo "  docs-serve        - 启动文档服务器"
	@echo ""
	@echo "CI/CD："
	@echo "  ci-check          - 运行CI检查"
	@echo "  install-tools     - 安装开发工具"

# 测试命令
test-unit:
	cargo test unit_tests

test-integration:
	cargo test integration_tests

test-e2e:
	cargo test e2e_tests

test-performance:
	cargo test performance_tests

test-error:
	cargo test error_tests

test-all:
	cargo test

# 基准测试
bench-quick:
	cargo bench --bench quick_benchmarks

bench-full:
	cargo bench --bench performance_benchmarks

# 代码质量
format:
	cargo fmt --all

clippy:
	cargo clippy -- -D warnings

coverage:
	cargo install cargo-tarpaulin || true
	cargo tarpaulin --out Html --workspace --exclude-files benches/*
	@echo "覆盖率报告已生成: tarpaulin-report.html"

security:
	cargo install cargo-audit || true
	cargo audit

# 构建
build:
	cargo build

build-release:
	cargo build --release

clean:
	cargo clean
	rm -rf target/criterion/
	rm -rf tarpaulin-report.html
	rm -rf cobertura.xml

# 文档
docs:
	cargo doc --no-deps --all-features

docs-serve:
	cargo doc --no-deps --all-features --open

# CI/CD检查
ci-check: format clippy security test-unit test-integration test-performance

# 安装开发工具
install-tools:
	cargo install cargo-tarpaulin cargo-audit cargo-release cargo-watch
	cargo install --git https://github.com/kbknapp/cargo-outdated

# 开发辅助
watch-test:
	cargo watch -x "test --lib"

watch-coverage:
	cargo watch -x "tarpaulin --out Html"

# 性能分析
perf-profile:
	cargo install cargo-flamegraph || true
	cargo flamegraph --bench performance_benchmarks

# 内存检查
memory-check:
	cargo install cargo-valgrind || true
	cargo valgrind test

# 交叉编译检查
cross-build:
	cargo install cross || true
	cross build --target x86_64-unknown-linux-gnu
	cross build --target aarch64-unknown-linux-gnu
	cross build --target x86_64-pc-windows-gnu

# 发布准备
release-check:
	cargo install cargo-release || true
	cargo release --dry-run --no-tag

# 依赖更新
update-deps:
	cargo update
	cargo outdated

# 代码分析
analyze:
	cargo install cargo-tree || true
	cargo tree
	cargo deny check

# 测试特定功能
test-data-loading:
	cargo test data_loading --bench performance_benchmarks

test-data-processing:
	cargo test data_processing --bench performance_benchmarks

test-cli:
	cargo test cli_processing --bench performance_benchmarks

# 快速开发循环
dev-test: test-unit test-integration
	@echo "快速开发测试完成"

dev-check: format clippy dev-test
	@echo "开发检查完成"

# 生产检查
prod-check: ci-check bench-quick coverage
	@echo "生产检查完成"

# 清理和重新构建
rebuild: clean build
	@echo "重新构建完成"

# 完整的CI流程
full-ci: ci-check bench-quick coverage cross-build
	@echo "完整CI流程完成"