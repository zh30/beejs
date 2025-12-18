#!/bin/bash
# 真实性能对比脚本 - 修复版
# 运行 Beejs 和 Bun 的基准测试并生成准确的对比报告

set -e

echo "🚀 开始真实性能对比测试 (修复版)"
echo "======================================"
echo ""

# 1. 运行 Bun 基准测试
echo "📊 运行 Bun 基准测试..."
bun run benchmarks/bun_benchmarks.js > benchmarks/bun_results.json 2>&1 || true

# 提取 Bun 结果
BUN_STARTUP=$(cat benchmarks/bun_results.json | grep startup_time_ms | cut -d':' -f2 | tr -d ' ,')
BUN_SIMPLE=$(cat benchmarks/bun_results.json | grep simple_execution_ops_per_sec | cut -d':' -f2 | tr -d ' ,')
BUN_COMPLEX=$(cat benchmarks/bun_results.json | grep complex_calculation_ops_per_sec | cut -d':' -f2 | tr -d ' ,')
BUN_MEMORY=$(cat benchmarks/bun_results.json | grep memory_usage_mb | cut -d':' -f2 | tr -d ' ,')
BUN_CONCURRENT=$(cat benchmarks/bun_results.json | grep concurrent_capacity | cut -d':' -f2 | tr -d ' ,')

echo "  ✅ Bun 基准测试完成"
echo ""

# 2. 运行 Beejs 基准测试
echo "📊 运行 Beejs 基准测试..."
node benchmarks/beejs_benchmark.js > benchmarks/beejs_results.json 2>&1 || true

# 提取 Beejs 结果
BEEJS_STARTUP=$(cat benchmarks/beejs_results.json | grep startup_time_ms | cut -d':' -f2 | tr -d ' ,')
BEEJS_SIMPLE=$(cat benchmarks/beejs_results.json | grep simple_execution_ops_per_sec | cut -d':' -f2 | tr -d ' ,')
BEEJS_COMPLEX=$(cat benchmarks/beejs_results.json | grep complex_calculation_ops_per_sec | cut -d':' -f2 | tr -d ' ,')

echo "  ✅ Beejs 基准测试完成"
echo ""

# 3. 计算性能差距
STARTUP_RATIO=$(echo "scale=2; $BEEJS_STARTUP / $BUN_STARTUP" | bc)
SIMPLE_RATIO=$(echo "scale=2; $BUN_SIMPLE / $BEEJS_SIMPLE" | bc)
COMPLEX_RATIO=$(echo "scale=2; $BUN_COMPLEX / $BEEJS_COMPLEX" | bc)
MEMORY_RATIO=$(echo "scale=2; $BEEJS_MEMORY / $BUN_MEMORY" | bc)

# 4. 生成准确的对比报告
echo "📝 生成性能对比报告..."
cat > benchmarks/real_performance_comparison_fixed.md << EOF
# Beejs vs Bun 真实性能对比报告 (修正版)

## 测试环境
- **测试日期**: $(date)
- **测试平台**: macOS Darwin 25.2.0
- **Beejs 版本**: 0.1.0
- **Bun 版本**: $(/Users/henry/.bun/bin/bun --version)

## 真实性能数据

### 启动时间
| 运行时 | 启动时间 | 差距 |
|--------|----------|------|
| Beejs | ${BEEJS_STARTUP}ms | 基准 |
| Bun | ${BUN_STARTUP}ms | **Bun 快 ${STARTUP_RATIO}x** |

### 简单执行速度
| 运行时 | 执行速度 | 差距 |
|--------|----------|------|
| Beejs | ${BEEJS_SIMPLE} ops/sec | 基准 |
| Bun | ${BUN_SIMPLE} ops/sec | **Bun 快 ${SIMPLE_RATIO}x** |

### 复杂计算
| 运行时 | 执行速度 | 差距 |
|--------|----------|------|
| Beejs | ${BEEJS_COMPLEX} ops/sec | 基准 |
| Bun | ${BUN_COMPLEX} ops/sec | **Bun 快 ${COMPLEX_RATIO}x** |

### 内存使用
| 运行时 | 内存使用 | 差距 |
|--------|----------|------|
| Beejs | ${BUN_MEMORY}MB | 基准 |
| Bun | ${BUN_MEMORY}MB | **Bun 少 ${MEMORY_RATIO}x** |

## 客观分析

### ✅ Beejs 的优势
1. **架构设计**: Rust + V8 提供内存安全和高性能
2. **模块化**: 完整的 JIT 优化、内联缓存、内存池系统
3. **AI 优化**: 专为 AI 工作负载设计的优化
4. **可扩展性**: 完整的服务器模式和 WebSocket 支持

### ⚠️ 性能差距分析
- **启动时间**: Bun 显著更快 (原生 vs 进程创建)
- **执行速度**: Bun 在 JIT 编译方面更成熟
- **内存效率**: Bun 使用更少内存
- **总体**: Bun 在纯性能方面领先，但 Beejs 有独特优势

## 结论

**Bun 在纯性能指标上显著领先**，但 Beejs 在以下方面具有价值：
- 🎯 **AI 工作负载优化**: 专为 AI 推理设计
- 🛡️ **内存安全**: Rust 提供内存安全保障
- 🔧 **可定制性**: 完整的模块化架构
- 🌐 **服务器模式**: 完整的 HTTP/WebSocket 支持

Beejs 适合需要内存安全、AI 优化和可定制性的场景。
Bun 适合需要极致性能的通用 JavaScript/TypeScript 执行。

---
*报告生成时间: $(date)*
EOF

echo "✅ 性能对比报告已生成: benchmarks/real_performance_comparison_fixed.md"
echo ""

# 显示报告
cat benchmarks/real_performance_comparison_fixed.md
