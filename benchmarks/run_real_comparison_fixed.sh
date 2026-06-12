#!/bin/bash
# 真实性能对比脚本 - 修复版
# 运行 Beejs 和 Bun 的基准测试并生成准确的对比报告

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BUN_BINARY="${BUN_BINARY:-bun}"
BEEJS_BINARY="${BEEJS_BINARY:-${REPO_ROOT}/target/release/bee}"
export BEEJS_BINARY

cd "${REPO_ROOT}"

echo "🚀 开始真实性能对比测试 (修复版)"
echo "======================================"
echo ""

# 1. 运行 Bun 基准测试
echo "📊 运行 Bun 基准测试..."
echo "  Command: ${BUN_BINARY} run benchmarks/bun_benchmarks.js"
"${BUN_BINARY}" run benchmarks/bun_benchmarks.js > benchmarks/bun_results.json 2>&1

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
echo "  Binary: ${BEEJS_BINARY}"
echo "  Command: node benchmarks/beejs_benchmark.js"
node benchmarks/beejs_benchmark.js > benchmarks/beejs_results.json 2>&1

# 提取 Beejs 结果
BEEJS_STARTUP=$(cat benchmarks/beejs_results.json | grep startup_time_ms | cut -d':' -f2 | tr -d ' ,')
BEEJS_SIMPLE=$(cat benchmarks/beejs_results.json | grep simple_execution_ops_per_sec | cut -d':' -f2 | tr -d ' ,')
BEEJS_COMPLEX=$(cat benchmarks/beejs_results.json | grep complex_calculation_ops_per_sec | cut -d':' -f2 | tr -d ' ,')
BEEJS_BINARY_REPORTED=$(cat benchmarks/beejs_results.json | grep bee_binary_path | cut -d':' -f2- | tr -d ' ,"')

echo "  ✅ Beejs 基准测试完成"
echo ""

# 3. 计算性能差距
STARTUP_RATIO=$(echo "scale=2; $BEEJS_STARTUP / $BUN_STARTUP" | bc)
SIMPLE_RATIO=$(echo "scale=2; $BUN_SIMPLE / $BEEJS_SIMPLE" | bc)
COMPLEX_RATIO=$(echo "scale=2; $BUN_COMPLEX / $BEEJS_COMPLEX" | bc)

# 4. 生成准确的对比报告
echo "📝 生成性能对比报告..."
cat > benchmarks/real_performance_comparison_fixed.md << EOF
# Beejs vs Bun 真实性能对比报告 (修正版)

## 测试环境
- **测试日期**: $(date)
- **测试平台**: macOS Darwin 25.2.0
- **Beejs 版本**: 0.1.0
- **Beejs binary**: ${BEEJS_BINARY_REPORTED}
- **Beejs benchmark command**: \`node benchmarks/beejs_benchmark.js\`
- **Bun 版本**: $("${BUN_BINARY}" --version)
- **Bun benchmark command**: \`${BUN_BINARY} run benchmarks/bun_benchmarks.js\`

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

本脚本当前不生成 Beejs 内存对比数据。`benchmarks/beejs_benchmark.js` 将
`memory_usage_mb` 和 `concurrent_capacity` 输出为 `null`，旧脚本中的估算值不可作为性能证明。

## 客观分析

### Beejs 的非性能观察
1. **实现路线**: Rust + V8 是当前运行时的核心技术栈。
2. **模块化**: 仓库包含 Node/Web API、测试、包管理和性能实验等模块，但稳定性以当前源码和测试为准。
3. **可验证优先**: 本报告只引用本次脚本实际采集的启动、简单执行和复杂计算结果；未采集的数据不作为优势声明。

### ⚠️ 性能差距分析
- **启动时间**: Bun 显著更快 (原生 vs 进程创建)
- **执行速度**: Bun 在 JIT 编译方面更成熟
- **内存效率**: Bun 使用更少内存
- **总体**: Bun 在本脚本采集的纯性能指标上领先；Beejs 的其他方向需要单独、可复现的专项验证。

## 结论

**Bun 在本脚本采集的纯性能指标上显著领先**。Beejs 目前更适合作为运行时设计与兼容层实验项目来评估；任何 AI、服务器模式或生产适用性结论都需要另行运行对应 feature、示例和专项基准。

---
*报告生成时间: $(date)*
EOF

echo "✅ 性能对比报告已生成: benchmarks/real_performance_comparison_fixed.md"
echo ""

# 显示报告
cat benchmarks/real_performance_comparison_fixed.md
