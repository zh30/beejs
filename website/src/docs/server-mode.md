---
title: "HTTP Serving & Multi-Worker Concurrency"
subtitle: "Harnessing lockless cross-thread dispatching and V8 Isolate thread pools for high-throughput I/O"
group: "Core Systems"
id: "server-mode"
---

## 1. Writing Modern HTTP Microservices

Beejs natively implements the full specification contract of `node:http`. Whether you are constructing high-throughput REST APIs, streaming large files, or serving long-lived connections, you can build on standard Node.js patterns:

```typescript
// server.ts - Production HTTP API service example
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

  // Global headers (CORS and JSON content type)
  res.setHeader('Content-Type', 'application/json; charset=utf-8');
  res.setHeader('Access-Control-Allow-Origin', '*');

  try {
    // Route dispatch
    if (method === 'GET' && parsedUrl.pathname === '/api/users') {
      res.writeHead(200);
      res.end(JSON.stringify({ code: 0, data: Array.from(users.values()) }));
      return;
    }

    if (method === 'POST' && parsedUrl.pathname === '/api/users') {
      // Stream request body chunks
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

    // 404 fallback
    res.writeHead(404);
    res.end(JSON.stringify({ code: 404, message: 'Not Found' }));
  } catch (err: any) {
    res.writeHead(500);
    res.end(JSON.stringify({ code: 500, error: err.message }));
  }
});

const PORT = 3000;
server.listen(PORT, () => {
  console.log(`🚀 HTTP server ready at http://localhost:${PORT}`);
});
```

---

## 2. Multi-Worker Thread Pool Architecture (`--workers`)

### The Single-Thread Bottleneck
In single-threaded event loop models, heavy operations such as JSON serialization, cryptography, or compression monopolize the event loop, causing queuing delays for all concurrent requests.

### Beejs Lockless Multi-Isolate Architecture
Beejs implements a **pre-warmed multi-worker thread pool model**:

```text
                        Incoming TCP Connections
                                    │
                                    ▼
                 +─────────────────────────────────────+
                 │      Main Dispatcher (Tokio I/O)    │
                 │   - TCP Listener / SO_REUSEPORT      │
                 │   - Lockless Channel Distribution   │
                 +─────────────────────────────────────+
                        │           │           │
           ┌────────────┘           │           └────────────┐
           ▼                        ▼                        ▼
+─────────────────────+  +─────────────────────+  +─────────────────────+
| Worker 1 (Isolate)  |  | Worker 2 (Isolate)  |  | Worker N (Isolate)  |
| - Dedicated Heap    |  | - Dedicated Heap    |  | - Dedicated Heap    |
| - Independent GC    |  | - Independent GC    |  | - Independent GC    |
| - Serves Req 1, 4, 7|  | - Serves Req 2, 5, 8|  | - Serves Req 3, 6, 9|
+─────────────────────+  +─────────────────────+  +─────────────────────+
```

### Enabling Multi-Worker Execution
Pass `-W` or `--workers` when running your script:

```bash
# Launch with 8 worker threads in parallel
bee run --workers 8 server.ts
```

Or configure via environment variables:
```bash
export BEE_WORKERS=8
bee run server.ts
```

**Key Advantages**:
- **True Multi-Core Parallelism**: Each worker runs on an isolated V8 Isolate. CPU-heavy handlers execute completely in parallel without blocking sibling cores.
- **Zero Per-Request Spawning Costs**: Workers are pre-warmed during boot. Incoming requests are scheduled via lockless memory queues, eliminating thread creation storm overhead.

---

## 3. Understanding `bee serve`

You may encounter the `bee serve` subcommand in the CLI:

```bash
bee serve --port 3000 --host 0.0.0.0
```

> [!IMPORTANT]
> **Please Note**: `bee serve` is an ultra-lightweight Rust health-check stub intended for **Kubernetes cluster LivenessProbe and ReadinessProbe** checks, returning a fixed `{"ok": true}` payload.
> 
> To serve your own JavaScript or TypeScript application code, always use **`bee run server.ts`** with `http.createServer`.

---

## 4. Benchmarking & Tuning

Benchmark the HTTP service using `autocannon`:

```bash
# Run with 8 worker threads
bee run --workers 8 server.ts

# Benchmark with 100 concurrent connections for 10 seconds
npx autocannon -c 100 -d 10 http://localhost:3000/api/users
```

### Production Tuning Tips
1. **Connection Reuse (Keep-Alive)**: Keep-Alive is enabled by default to minimize TCP/TLS handshake round-trips.
2. **Buffer Streaming**: Prefer `Buffer.concat()` and stream piping over string concatenation for binary payloads to take advantage of Beejs SIMD optimizations.
3. **Worker Count Tuning**: For I/O-heavy workloads, set workers to `1x` to `2x` the physical CPU core count. For compute-heavy services, align strictly with physical cores to avoid context switching.
