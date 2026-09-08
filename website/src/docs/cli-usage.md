---
title: "Complete CLI Command Reference & Sandbox Security"
subtitle: "Comprehensive subcommands, developer tooling suite, and deterministic Agent sandbox flags"
group: "Reference & Specs"
id: "cli-usage"
---

Beejs provides an all-in-one developer CLI, integrating script execution, serving, testing, code quality, packaging, and Agent sandbox security.

---

## 1. Subcommands Cheat Sheet

| Subcommand | Description | Example |
| :--- | :--- | :--- |
| **`bee run <file>`** | Execute JS / TS / TSX scripts | `bee run app.ts --watch` |
| **`bee serve [file]`** | Run modern Web application server | `bee serve --port 8080` |
| **`bee eval <code>`** | Evaluate inline JavaScript expression | `bee eval "1 + 1"` |
| **`bee repl`** | Interactive REPL with multiline detection & history | `bee repl` |
| **`bee task [name]`** | Fast zero-npm task runner for package.json scripts | `bee task build`, `bee run dev` |
| **`bee fmt [files]`** | Sub-millisecond code formatter powered by OXC | `bee fmt src/ --check` |
| **`bee lint [files]`** | Ultra-fast AST linter and diagnostics via OXC | `bee lint src/` |
| **`bee bundle <entry>`**| Production Bundler 2.0 (Minify & Sourcemaps) | `bee bundle src/index.ts -o dist/bundle.js --minify` |
| **`bee compile <file>`**| Single Executable Application (SEA) compiler | `bee compile app.ts -o myapp` |
| **`bee test [file]`** | Jest/Vitest compatible testing & coverage | `bee test --coverage` |
| **`bee bench [file]`** | High-precision microbenchmark test suite | `bee bench benchmarks/` |
| **`bee profile <file>`**| Generate Chrome DevTools `.cpuprofile` flamecharts | `bee profile app.ts -o app.cpuprofile` |
| **`bee debug [file]`** | Launch script paused for Chrome DevTools CDP | `bee debug app.ts` |
| **`bee lsp`** | Launch Language Server Protocol (LSP 3.17) server | `bee lsp` |
| **`bee types`** | Export TypeScript definitions (AI & Web APIs) | `bee types -o beejs.d.ts` |
| **`bee session <file>`**| Stdio JSON-RPC interface for Agent hosts | `bee session agent.ts` |
| **`bee mcp <file>`** | Model Context Protocol (MCP) server | `bee mcp tools.ts` |
| **`bee init [name]`** | Scaffold a new project and package.json | `bee init my-project` |
| **`bee install`** | Install project dependencies | `bee install` |

---

## 2. Command Details & Flags

### 2.1 `bee run` Script Execution

```bash
bee run [OPTIONS] <FILE> [-- ARGS...]
```

- **`-w, --watch`**: Watch files and hot-reload upon changes;
- **`--debounce <MS>`**: Debounce delay for reload events (default: `100`ms);
- **`-r, --preload <MODULE>`**: Preload and evaluate a module before main entry;
- **`-W, --workers <NUM>`**: Concurrency level of V8 Isolate workers (default: `1`);
- **`--timeout <MS>`**: Hard CPU timeout watchdog in milliseconds, forcibly terminating infinite loops;
- **`--max-memory <MB>`**: Physical V8 heap quota limit in megabytes;
- **`--seed <U64>`**: Seed for Mulberry32 PRNG for 100% deterministic `Math.random()`;
- **`--freeze-time <TIMESTAMP>`**: Freeze system time to a fixed timestamp or ISO8601 string;
- **`--import-map <PATH>`**: WICG Import Maps JSON mapping file;
- **`--inspect [ADDR]`**: Enable Chrome DevTools Protocol endpoint (default: `127.0.0.1:9229`);
- **`--inspect-brk [ADDR]`**: Enable CDP and break before user script starts.

### 2.2 `bee serve` Modern Web Server

```bash
bee serve [OPTIONS] [FILE]
```

- **`FILE`**: Application entry (defaults to auto-detecting `app.ts`, `app.js`, `server.ts`, `server.js`, `index.ts`, `index.js`);
- **`-p, --port <PORT>`**: Port to bind (default: `3000`);
- **`-H, --host <HOST>`**: Host address to bind (default: `localhost`);
- **`--max-memory <MB>`**: Restrict memory consumption for the serving process.

### 2.3 `bee bundle` Bundler 2.0

```bash
bee bundle [OPTIONS] <ENTRY>
```

- **`-o, --outfile <FILE>`**: Output bundle path (default: `dist/bundle.js`);
- **`-m, --minify`**: Minify code and eliminate dead code using OXC;
- **`-s, --sourcemap`**: Generate v3 SourceMap files;
- **`--target <ES>`**: Target ECMAScript version (`es2022`, `esnext`);
- **`--import-map <PATH>`**: Apply bare module resolution maps.

### 2.4 `bee compile` Standalone Binary Compiler

```bash
bee compile [OPTIONS] <FILE>
```

- **`-o, --outfile <PATH>`**: Executable binary output path;
- **`--minify`**: Minify bundled code before binary packaging;
- **`--include-assets <DIR>`**: Bundle a directory of static assets.

---

## 3. Agent Deterministic Sandbox & Granular Permissions

When executing untrusted code or autonomous Agent outputs, defense-in-depth isolation is critical:

```bash
# Run with closed sandbox and strict quotas
$ bee run --sandbox \
    --timeout 5000 \
    --max-memory 256 \
    --seed 42 \
    --allow-read ./data \
    agent_workflow.ts
```

### 3.1 Deterministic Resource Quotas
- **`--timeout <ms>`**: Independent watchdog thread triggers `v8::IsolateHandle::terminate_execution()` upon expiration, reliably escaping `while(true)` loops;
- **`--max-memory <MB>`**: Configures V8 `ResourceConstraints.max_old_generation_size_in_bytes` to prevent OOM memory exhaustion;
- **`--seed <u64>`**: Mulberry32 PRNG ensures reproducible randomness across replays;
- **`--freeze-time <time>`**: Fixes timestamps so evaluation does not drift with wall-clock time.

### 3.2 Granular Whitelisting Options
- **`--sandbox`**: Deny all unapproved file access, network calls, env reads, and child processes;
- **`--allow-read <PATHS>`**: Whitelist paths or files for read access;
- **`--allow-write <PATHS>`**: Whitelist paths or files for write access;
- **`--allow-net <HOSTS>`**: Whitelist network hosts and domains;
- **`--allow-listen <ADDRS>`**: Whitelist network listening addresses;
- **`--allow-env <VARS>`**: Whitelist allowed environment variable names;
- **`--allow-run <BINS>`**: Whitelist executable paths permitted for spawning;
- **`--audit-log <PATH>`**: Stream all runtime permission checks into a JSONL audit log.
