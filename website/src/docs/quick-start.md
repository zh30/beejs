---
title: "Quick Start Guide"
subtitle: "Write and execute your first modern TypeScript application in 3 minutes"
group: "Getting Started"
id: "quick-start"
---

## 1. Writing Your First TypeScript Script

Beejs natively executes TypeScript (`.ts` / `.tsx`) files out of the box without requiring `tsc`, `ts-node`, or `tsconfig.json`.

Create a file named `hello.ts`:

```typescript
// hello.ts - Demonstrates type annotations, interfaces, and top-level await
interface SystemInfo {
  runtime: string;
  version: string;
  arch: string;
  uptime: number;
}

const getInfo = async (): Promise<SystemInfo> => {
  // Simulate an asynchronous operation
  await new Promise((resolve) => setTimeout(resolve, 50));
  
  return {
    runtime: 'Beejs',
    version: '1.0.0',
    arch: process.arch,
    uptime: Math.round(process.uptime() * 1000) / 1000,
  };
};

// Use top-level await directly without wrapping in async IIFE
const info = await getInfo();
console.log(`🚀 Welcome to ${info.runtime} v${info.version} (${info.arch})! Started in: ${info.uptime}s`);
```

Run it using `bee run`:

```bash
$ bee run hello.ts
🚀 Welcome to Beejs v1.0.0 (arm64)! Started in: 0.016s
```

In under **18ms**, your TypeScript code was parsed, type-erased by `oxc`, and executed by Google V8!

---

## 2. Core CLI Workflows

### File Watching & Hot Reload (`--watch`)
During local development, use `--watch` to automatically restart whenever source files change:

```bash
bee run --watch hello.ts
```

You can customize the debounce window (in milliseconds) and the notification port:
```bash
bee run --watch --debounce 200 -p 9999 server.ts
```

### Fast One-Liner Evaluation (`bee eval`)
Evaluate one-off expressions or inspect module behavior directly from the terminal:

```bash
# Print system platform, architecture, and memory usage
bee eval "console.log({ arch: process.arch, platform: process.platform, memory: process.memoryUsage() });"

# Generate a cryptographically secure UUID
bee eval "console.log(crypto.randomUUID());"
```

### Interactive Console (`bee repl`)
Launch the interactive REPL with full multi-line input, top-level await, and tab completion:

```bash
$ bee repl
Beejs v1.0.0 REPL
Type ".exit" to quit, ".help" for help.

> const buf = Buffer.from("Hello Beejs");
> buf.toString('hex')
'48656c6c6f204265656a73'
> await fetch('https://httpbin.org/get').then(r => r.json())
{ origin: '...', url: 'https://httpbin.org/get', ... }
> .exit
```

---

## 3. Minimal HTTP Microservice

Using the standard `node:http` module, writing server applications is straightforward:

```typescript
// server.ts
import http from 'node:http';

const server = http.createServer((req, res) => {
  const url = new URL(req.url || '/', `http://${req.headers.host}`);
  
  if (url.pathname === '/health') {
    res.writeHead(200, { 'Content-Type': 'application/json' });
    res.end(JSON.stringify({ status: 'ok', time: new Date().toISOString() }));
    return;
  }
  
  res.writeHead(200, { 'Content-Type': 'text/plain; charset=utf-8' });
  res.end('🐝 Hello from Beejs HTTP Server!\n');
});

const PORT = 3000;
server.listen(PORT, () => {
  console.log(`⚡ HTTP server listening at http://localhost:${PORT}`);
});
```

Run the server:
```bash
bee run server.ts
```

Test requests with curl:
```bash
$ curl http://localhost:3000/
🐝 Hello from Beejs HTTP Server!

$ curl http://localhost:3000/health
{"status":"ok","time":"2026-09-07T06:00:00.000Z"}
```

---

## 4. Passing Command-Line Arguments

Arguments passed after the filename are exposed via `process.argv`:

```typescript
// cli.ts
const args = process.argv.slice(2);
console.log('Arguments passed:', args);
```

Run with flags:
```bash
$ bee run cli.ts --name myapp --port 8080
Arguments passed: [ '--name', 'myapp', '--port', '8080' ]
```

---

## 5. Recommended Project Structure

Here is a standard project layout for production-ready TypeScript applications on Beejs:

```text
my-bee-app/
├── package.json         # Package metadata and dependencies
├── tsconfig.json        # Optional: For IDE type-checking and autocompletion
├── src/
│   ├── index.ts         # Main entry point
│   ├── routes/          # API route handlers
│   ├── services/        # Business logic services
│   └── models/          # Types and data schemas
├── tests/
│   └── api.test.ts      # Test suites (run with `bee test`)
└── .env                 # Environment variables
```

Add standard scripts to `package.json`:

```json
{
  "name": "my-bee-app",
  "version": "1.0.0",
  "scripts": {
    "dev": "bee run --watch src/index.ts",
    "start": "bee run --workers 4 src/index.ts",
    "test": "bee test tests/",
    "check": "tsc --noEmit"
  }
}
```

> [!TIP]
> Run code during development and production directly with `bee run` for instant cold starts. Run `tsc --noEmit` in CI to ensure static type soundness.
