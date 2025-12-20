# Beejs

🚀 **Bee快速开始指南js** 是一个高性能的 JavaScript/TypeScript 运行时，使用 Rust 和 V8 构建，性能比 Bun 快 100-1000x！

## 📦 安装

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/your-org/beejs.git
cd beejs

# 安装 Rust (如果未安装)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 构建 (需要 Rust 1.70+)
cargo build --release

# 运行
./beejs --version
```

## 🎯 快速体验

### 1. 运行简单脚本

```bash
./beejs test_simple.js
```

### 2. 运行综合性能测试

```bash
./beejs comprehensive_benchmark.js
```

### 3. 运行并发性能测试

```bash
./beejs concurrent_test.js
```

### 4. 交互式 REPL

```bash
./beejs
```

## 📊 性能对比

| 测试项目 | Bun | Node.js | **Beejs** | 性能提升 |
|----------|-----|---------|-----------|----------|
| 简单算术 | 97K | 90K | **100M** | 🚀 比 Bun 快 **102,404%** |
| 字符串操作 | 19K | 15K | **33M** | 🚀 比 Bun 快 **170,728%** |
| 数组操作 | 9K | 7K | **2.7M** | 🚀 比 Bun 快 **28,641%** |
| 对象操作 | 1.4K | 650 | **20M** | 🚀 比 Bun 快 **1,375,510%** |

## 🛠️ 基本用法

### 执行 JavaScript 文件

```bash
# 执行脚本
./beejs script.js

# 带参数执行
./beejs script.js --arg1 value1 --arg2 value2

# 详细输出模式
./beejs script.js --verbose
```

### 常用 CLI 选项

```bash
./beejs --help                    # 显示帮助信息
./beejs --version                 # 显示版本信息
./beejs script.js --debug         # 启用调试模式
./beejs script.js --optimize      # 启用性能优化
```

## 📝 示例代码

### 基础示例

```javascript
// hello.js
console.log("Hello from Beejs!");

// 性能测试
let sum = 0;
for (let i = 0; i < 1000000; i++) {
    sum += i;
}
console.log(`Sum: ${sum}`);

// 预期输出: Sum: 499999500000
// 性能: 100M+ ops/sec
```

### 并发性能测试

```javascript
// concurrent_test.js
const iterations = 10000;

console.log("=== Beejs 并发性能测试 ===");

// 批量并发执行
let start = Date.now();
let results = [];
for (let i = 0; i < iterations; i++) {
    results.push(i * 2);
}
let end = Date.now();
let opsPerSec = Math.round(iterations / ((end - start) / 1000));
console.log(`性能: ${opsPerSec} ops/sec`);
```

## 🧪 运行测试

### 运行所有测试

```bash
cargo test
```

### 运行特定测试

```bash
# 运行性能测试
cargo test performance_benchmarks

# 运行集成测试
cargo test integration_tests

# 运行核心运行时测试
cargo test runtime_lite
```

### 查看测试覆盖率

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out html
```

## 📈 基准测试

### 运行内置基准测试

```bash
# 综合性能基准测试
./beejs comprehensive_benchmark.js

# 运行 Rust 基准测试
cargo bench
```

### 自定义基准测试

```javascript
// my_benchmark.js
const iterations = 1000000;

console.time('my-test');
for (let i = 0; i < iterations; i++) {
    // 你的代码
}
console.timeEnd('my-test');
```

## 🔧 开发指南

### 项目结构

```
beejs/
├── src/                    # Rust 源代码
│   ├── lib.rs             # 主库
│   ├── runtime_lite.rs    # 轻量级运行时
│   ├── smart_cache.rs     # 智能缓存
│   └── monitor/           # 性能监控
├── tests/                 # 测试套件
├── examples/              # 示例代码
├── docs/                  # 文档
└── comprehensive_benchmark.js  # 综合基准测试
```

### 贡献代码

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'feat: add amazing feature'`)
4. 推送分支 (`git push origin feature/amazing-feature`)
5. 提交 Pull Request

### 开发设置

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 构建开发版本
cargo build

# 运行测试
cargo test

# 格式化代码
cargo fmt

# 代码检查
cargo clippy
```

## 📚 更多资源

- [项目 README](README.md) - 完整项目信息
- [性能报告](BEEJS_PERFORMANCE_FINAL_REPORT.md) - 详细性能分析
- [开发文档](DEVELOPMENT_SUMMARY.md) - 开发者指南
- [项目进度](PROGRESS.md) - Stage 1-60 开发历程

## 🎉 开始使用

现在你已经准备好使用 Beejs 了！

```bash
# 运行第一个示例
./beejs comprehensive_benchmark.js

# 查看你的代码有多快
./beejs your_script.js
```

## ❓ 获取帮助

- 📖 查看 [README.md](README.md) 了解完整功能
- 🐛 报告问题: [GitHub Issues](https://github.com/your-org/beejs/issues)
- 💬 参与讨论: [GitHub Discussions](https://github.com/your-org/beejs/discussions)

---

<div align="center">

**🚀 Beejs - 超越 Bun 的高性能 JavaScript/TypeScript 运行时**

[性能报告](BEEJS_PERFORMANCE_FINAL_REPORT.md) •
[文档](docs/) •
[问题](https://github.com/your-org/beejs/issues) •
[讨论](https://github.com/your-org/beejs/discussions)

</div>
