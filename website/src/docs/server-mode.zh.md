---
title: "高性能 HTTP 服务与多 Worker 架构"
subtitle: "利用无锁跨线程任务分发与 V8 Isolate 线程池，承载每秒数万高并发连接"
group: "核心系统"
id: "server-mode"
---

## 1. 编写现代 HTTP 微服务

Beejs 原生实现了完整的 `node:http` 模块标准契约。无论是构建极速 RESTful API、处理大文件流式上传，还是提供长连接服务，都可以直接采用熟悉的 Node.js 编程模式：

```typescript
// server.ts - 生产级 HTTP 微服务示例
import http from 'node:http';

interface User {
  id: string;
  name: string;
  role: string;
}

const users: Map<string, User> = new Map([
  ['1', { id: '1', name: 'Alice', role: 'admin' }],
  ['2', { id: '2', name: 'Bob', role: 'engineer' }],
]);

const server = http.createServer(async (req, res) => {
  const { method, url } = req;
  const parsedUrl = new URL(url || '/', `http://${req.headers.host}`);

  // 全局响应头 (CORS 与 JSON)
  res.setHeader('Content-Type', 'application/json; charset=utf-8');
  res.setHeader('Access-Control-Allow-Origin', '*');

  try {
    // 路由分发
    if (method === 'GET' && parsedUrl.pathname === '/api/users') {
      res.writeHead(200);
      res.end(JSON.stringify({ code: 0, data: Array.from(users.values()) }));
      return;
    }

    if (method === 'POST' && parsedUrl.pathname === '/api/users') {
      // 流式读取 Request Body
      const chunks: Buffer[] = [];
      for await (const chunk of req) {
        chunks.push(typeof chunk === 'string' ? Buffer.from(chunk) : chunk);
      }
      const body = JSON.parse(Buffer.concat(chunks).toString('utf-8'));
      
      const newUser: User = {
        id: String(users.size + 1),
        name: body.name || 'Anonymous',
        role: body.role || 'user',
      };
      users.set(newUser.id, newUser);

      res.writeHead(201);
      res.end(JSON.stringify({ code: 0, data: newUser }));
      return;
    }

    // 404 兜底
    res.writeHead(404);
    res.end(JSON.stringify({ code: 404, message: 'Not Found' }));
  } catch (err: any) {
    res.writeHead(500);
    res.end(JSON.stringify({ code: 500, error: err.message }));
  }
});

const PORT = 3000;
server.listen(PORT, () => {
  console.log(`🚀 HTTP 服务就绪: http://localhost:${PORT}`);
});
```

---

## 2. 多 Worker 线程池并发架构 (`--workers`)

### 传统单线程事件循环的局限
在单线程事件循环运行时（如未做集群配置的 Node.js）中，一旦某个请求触发了密集的 JSON 序列化、密码学哈希计算或解压缩，整个主线程事件循环便会被阻塞，导致其他所有正在排队的请求延迟激增。

### Beejs 的无锁多 Isolate 线程池
Beejs 在底层设计了**多 Worker 线程池并发模型**：

```text
                        客户端并发请求 (TCP Traffic)
                                    │
                                    ▼
                 +─────────────────────────────────────+
                 │      主分发线程 (Rust Tokio I/O)    │
                 │   - TCP 连接监听与 SO_REUSEPORT      │
                 │   - 无锁 Channel 快速轮询分发       │
                 +─────────────────────────────────────+
                        │           │           │
           ┌────────────┘           │           └────────────┐
           ▼                        ▼                        ▼
+─────────────────────+  +─────────────────────+  +─────────────────────+
| Worker 1 (Isolate)  |  | Worker 2 (Isolate)  |  | Worker N (Isolate)  |
| - 专属 V8 执行堆    |  | - 专属 V8 执行堆    |  | - 专属 V8 执行堆    |
| - 独立 GC 垃圾回收  |  | - 独立 GC 垃圾回收  |  | - 独立 GC 垃圾回收  |
| - 处理请求 1, 4, 7  |  | - 处理请求 2, 5, 8  |  | - 处理请求 3, 6, 9  |
+─────────────────────+  +─────────────────────+  +─────────────────────+
```

### 启用多 Worker 模式
在运行脚本时，通过 `-W` 或 `--workers` 指定工作线程数量：

```bash
# 启用 8 个并行 Worker 线程
bee run --workers 8 server.ts
```

或者在生产环境中设置环境变量：
```bash
export BEE_WORKERS=8
bee run server.ts
```

**优势所在**：
- **真正的多核并行**：各个 Worker 运行在独立且互不干扰的 V8 Isolate 中，CPU 密集型任务完全并行，不会阻塞其他核心；
- **零请求创建成本**：所有 Worker 在进程启动时预热完成，请求到达时仅通过内存中的无锁队列唤醒，消除了每次请求重新创建线程的系统调用风暴。

---

## 3. `bee serve` 诊断与健康检查模式

在 CLI 子命令中，你可能会发现 `bee serve`：

```bash
bee serve --port 3000 --host 0.0.0.0
```

> [!IMPORTANT]
> **请注意**：`bee serve` 是一个由 Rust 直接驱动的超轻量健康检查 Stub，专门用于 **Kubernetes 集群的 LivenessProbe / ReadinessProbe** 以及负载均衡器的存活探测，返回固定的 `{"ok": true}`。
> 
> 如果你要运行自己的 JavaScript / TypeScript 业务服务，请始终使用 **`bee run server.ts`**（搭配 `http.createServer`）。

---

## 4. 性能压测与优化建议

在相同硬件环境下使用 `autocannon` 对上述 HTTP 示例进行压测：

```bash
# 启动 8 核心工作线程
bee run --workers 8 server.ts

# 发起压测 (100 并发连接，持续 10 秒)
npx autocannon -c 100 -d 10 http://localhost:3000/api/users
```

### 生产优化技巧
1. **复用连接 (Keep-Alive)**：默认保持开启，大幅减少每次请求重新建立 TCP 三次握手与 TLS 握手的网络开销；
2. **避免大字符串拼接**：处理二进制或大载荷时，优先使用 `Buffer.concat()` 或流式 Piping，配合 Beejs 的 Rust SIMD 向量化加速；
3. **设置合理的 Worker 数量**：在纯 I/O 服务中，Worker 数量建议设为 `CPU核心数` 至 `CPU核心数 * 2`；在重度密集计算时，建议严格等于物理核心数以避免频繁上下文切换。
