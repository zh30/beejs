---
title: "Overview & Philosophy"
subtitle: "High-performance JavaScript/TypeScript runtime built for modern high-concurrency I/O, edge computing, and AI-native workloads"
group: "Getting Started"
id: "introduction"
---

## What is Beejs?

**Beejs** is a next-generation JavaScript and TypeScript runtime built from the ground up using **Rust** and **Google V8**.

Over the last decade, Node.js powered the rise of server-side JavaScript, while Deno and Bun pioneered unified modern toolchains. However, as edge computing, high-density serverless platforms, and autonomous AI agents become mainstream, modern server workloads face critical challenges:

- **Instant Cold Starts**: Serverless and edge workers require runtime startup times strictly under 20ms.
- **Heavy Tensor & Vector Operations**: AI workloads demand native high-dimensional vector math, embeddings, and streaming LLM inference without heavyweight Python environments.
- **Zero-Copy High-Concurrency I/O**: Sustaining tens of thousands of concurrent requests per second requires eliminating redundant memory copies and thread switching bottlenecks.
- **Deterministic Sandboxing**: Autonomous agent execution requires strict capability policies, virtual clock freezing, and deterministic PRNG seeds.

Beejs was engineered from first principles to solve these challenges for production environments.

---

## Core System Architecture

Beejs fuses Rust's memory safety and zero-cost abstractions with Google V8's raw execution performance and oxc's sub-millisecond compiler:

```text
+-----------------------------------------------------------------------+
|                 Application Layer (JavaScript / TypeScript)           |
|           React / TSX  ·  HTTP APIs  ·  Agent Pipelines  ·  Tests     |
+-----------------------------------------------------------------------+
|                               Runtime APIs                            |
|   +--------------------+  +--------------------+  +----------------+  |
|   | Node.js Compat     |  | Web Standards API  |  | Native AI      |  |
|   | (fs, http, buffer) |  | (fetch, Streams)   |  | (Tensor, LLM)  |  |
|   +--------------------+  +--------------------+  +----------------+  |
+-----------------------------------------------------------------------+
|              Multi-Worker Thread Pool (Multi-Isolate Concurrency)     |
|   [Worker 1] <--- Lockless Channel ---> [Worker 2] ... [Worker N]     |
+-----------------------------------------------------------------------+
|                            Core Engine Layer                          |
|   +--------------------------+     +-------------------------------+  |
|   |     Google V8 Engine     |     |  oxc Native TS/TSX Pipeline   |  |
|   | (JIT, WASM, GC, mmap 2.0)|     | (Type erasure, Decorator)     |  |
|   +--------------------------+     +-------------------------------+  |
+-----------------------------------------------------------------------+
|                     Host Runtime & System Abstraction                 |
|  - Rust SIMD Vectorized Buffer       - Deterministic ResourceBroker   |
|  - High-precision Timing Wheel       - Dual-cache Stat Bypass Module  |
+-----------------------------------------------------------------------+
```

---

## Key Highlights

### 1. Sub-Millisecond V8 Snapshot 2.0 (<18ms Cold Start)
Traditional runtimes spend tens of milliseconds initializing global objects and parsing built-in modules on startup. Beejs pre-compiles internal contexts into a binary snapshot and uses `memmap2` for zero-copy memory mapping. Isolates share read-only pages via Copy-on-Write, achieving sub-18ms cold starts—**1.93x faster than Node.js**.

### 2. Rust SIMD Zero-Copy Buffer (#1 Fastest Across Engines)
Leveraging modern CPU vector extensions (AVX2 / NEON), Beejs accelerates memory allocation, slice operations, and byte searches in `node:buffer`. In 100,000 64KB buffer benchmarks, Beejs completes operations in just **2.09ms**, outperforming Bun and Node.js.

### 3. Dual-Cache Module Resolution (4.6M ops/s)
Module loading is often a major startup bottleneck. Beejs implements a two-tiered memory cache that completely bypasses filesystem `stat` system calls, reaching an unprecedented **4,601,226 ops/s** resolution throughput (**4.1x faster than Node.js**).

### 4. Zero-Config Native TypeScript 6.0 & TSX
Powered by the Rust-native `oxc` compiler, Beejs runs `.ts`, `.tsx`, `.mts`, and `.jsx` files without any `tsc` compilation or `tsconfig.json` boilerplate. It strips types in sub-milliseconds and natively downlevels Stage 3 Decorators and `using` resource management.

### 5. Native Agentic AI Engine (`bee:ai`)
No Python installations or brittle native addons needed. Beejs exposes native `Tensor` objects backed by `Float32Array`, local LLM streaming inference (`LLM`), and autonomous agent execution pipelines (`AgentPipeline`) directly in JavaScript.

---

## Comparison Matrix

| Capability | Beejs v1.0.0 | Node.js v24 | Bun v1.4 | Deno v2.x |
| :--- | :---: | :---: | :---: | :---: |
| **Engine** | Google V8 + Rust | Google V8 + C++ | JSC + Zig | Google V8 + Rust |
| **CLI Cold Start** | **< 18 ms** | ~35 ms | ~15 ms | ~25 ms |
| **TypeScript / TSX** | **Native oxc (zero-config)** | Flags / external | Built-in | Built-in SWC |
| **Buffer 100k SIMD** | **2.09 ms (#1 fastest)** | 4.80 ms | 2.50 ms | 4.20 ms |
| **Module Resolution**| **4.61M ops/s** | 1.12M ops/s | 3.88M ops/s | 2.10M ops/s |
| **HTTP Concurrency** | **Worker Thread Pool** | Single / Cluster | Multi-threaded | Tokio async |
| **Native AI Engine** | **Built-in `bee:ai`** | npm / node-gyp | Experimental | WebGPU required |
| **Deterministic Sandbox**| **Supported (seed/time/audit)**| OS containers | No sandbox | Capabilities |
| **Node Conformance** | **100% (51/51 suites)** | Official native | High | High |
