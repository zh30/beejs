# Bun-Aligned Idle Memory Benchmark Suite

> **Reference Tweet**: [https://x.com/bunjavascript/status/2095696147813945347](https://x.com/bunjavascript/status/2095696147813945347)  
> **Benchmark Focus**: *"Resident memory 3 minutes after 60 seconds of sustained load, lower is better."*

---

## 📖 背景与原理

Bun 官方在 X (Twitter) 上发布了关于 **Bun v1.4.1 空闲内存回收（Idle Memory Usage）** 的性能数据对比。

### 为什么该指标至关重要？
许多服务器在经历高并发洪峰时，由于临时创建的大量请求上下文、I/O 缓冲区以及堆内存扩张，会导致进程常驻内存（RSS）急剧上升。在流量回落至零（静默空闲）后，某些运行时由于内存分配器缓存未释放、垃圾回收不彻底或未向操作系统归还物理页（`madvise(MADV_DONTNEED)`），仍然长期霸占大量物理内存，从而造成多租户边缘计算与容器编排中的“内存虚高”与 OOM。

Bun v1.4.1 在该项指标上进行了专项优化，测试了 **Next.js SSR、Vite dev、Express、Fastify、Elysia、Hono** 在 **60秒高负载（64连接）** 后、进入 **3分钟空闲冷却** 时的常驻内存（RSS），越低越好。

---

## 🔬 测试框架与运行时矩阵

本项目完全复刻了该测试方法论，支持对比三个主流/自研运行时：
- **Bun v1.4.1**
- **Node.js v22**
- **Beejs v0.4.0**

支持的工作负载：
1. **Hono** (`server_hono.js`): 高性能现代 HTTP 路由框架
2. **Raw HTTP** (`server_http.js`): 零依赖标准 Node `http` 原生基准
3. **Express** (`server_express.js`): 传统 Node.js Web 框架
4. **Fastify** (`server_fastify.js`): 现代化高性能 Node.js Web 框架

---

## 🚀 运行方式

### 1. 快速验证模式（Quick Mode: 10s 负载 + 15s 冷却）
适合本地快速验证与 CI 流水线测试：
```bash
python3 benchmarks/idle_memory/runner.py --mode quick
```

### 2. 完整复刻模式（Full Mode: 60s 持续负载 + 180s 静默冷却）
完全复刻 Bun 官方推文中的 4 分钟严谨生命周期测试：
```bash
python3 benchmarks/idle_memory/runner.py --mode full
```

### 3. 自定义过滤
```bash
# 仅对比 Hono 在 Bun、Node 与 Beejs 上的表现
python3 benchmarks/idle_memory/runner.py --workloads hono --runtimes bun,node,bee

# 自定义并发与时长
python3 benchmarks/idle_memory/runner.py --connections 64 --duration 30 --cooldown 60
```

---

## 📊 输出产物
- 实时终端表格
- [`REPORT.md`](file:///Users/henry/code/beejs/benchmarks/idle_memory/REPORT.md): Markdown 对比报告（含基线、峰值、最终空闲 RSS 与回收率）
- [`results.json`](file:///Users/henry/code/beejs/benchmarks/idle_memory/results.json): 包含每 100ms 采样的完整时间序列数据
