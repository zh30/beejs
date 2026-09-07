---
title: "CLI Command Reference & Sandbox Security"
subtitle: "Comprehensive subcommands, flags reference, and fine-grained capability sandboxing"
group: "Developer Guide"
id: "cli-usage"
---

## 1. Subcommands Quick Reference

Beejs provides an all-in-one developer toolchain:

| Command | Description | Example Flags |
| :--- | :--- | :--- |
| **`bee run <file>`** | Execute a JS, TS, or TSX script | `--watch`, `--workers 4`, `--sandbox` |
| **`bee eval <code>`** | Evaluate a one-off JavaScript snippet | `bee eval "console.log(process.arch)"` |
| **`bee repl`** | Start the interactive REPL shell | Multiline editing, top-level await |
| **`bee test [path]`** | Run Jest / Vitest compatible test runner | `-t "auth"`, `--bail`, `--parallel`, `-w` |
| **`bee bundle <entry>`**| Bundle local modules (experimental) | `-o dist/bundle.js`, `--minify` |
| **`bee serve`** | Start lightweight health-check HTTP stub | `--port 3000`, `--host 0.0.0.0` |
| **`bee session <file>`**| Stdio JSON-RPC session for Agent hosts | `--isolate-per-call` |
| **`bee mcp <file>`** | Run Model Context Protocol (MCP) server | `--isolate-per-call` |
| **`bee init [name]`** | Initialize a new project with package.json | `bee init my-app` |
| **`bee add <pkg>`** | Add and install an npm package | `bee add lodash@4.17.21`, `--dev` |
| **`bee install`** | Install dependencies from package.json | `--frozen-lockfile` (for CI) |
| **`bee remove <pkg>`** | Remove an installed dependency | `bee remove lodash` |

---

## 2. `bee run` Options Reference

`bee run` is the primary command for execution and service serving:

```bash
bee run [OPTIONS] <FILE> [-- SCRIPT_ARGS...]
```

### Execution Control Options
- **`-w, --watch`**: Enable live file watching and hot reload on file changes.
- **`--debounce <MS>`**: Debounce window for hot reload events in milliseconds (default: `100`).
- **`-p, --websocket-port <PORT>`**: Port used by the hot reload notification channel (default: `9999`).
- **`-r, --preload, --require <MODULE>`**: Preload and evaluate a module before the entrypoint runs (repeatable).
- **`-W, --workers <NUM>`**: Spawn $N$ parallel worker isolates for concurrent HTTP handling (default: `1`, or `BEE_WORKERS`).
- **`--export-tools`**: Parse and print exported tool schemas as JSON and exit (used for Agent tool discovery).
- **`-v, --verbose`**: Print internal debug and module resolution logs.

---

## 3. Fine-Grained Security Sandboxing

When hosting multi-tenant code or executing autonomous AI agent actions, unrestricted disk and network access present severe security risks. Beejs embeds an enterprise-grade **ResourceBroker capability engine**:

### 1. Default-Deny Sandbox (`--sandbox`)
Passing `--sandbox` instructs the runtime to **block all filesystem reads/writes, network sockets, environment variables, and child processes by default**:

```bash
# Deny all unauthorized host I/O
bee run --sandbox untrusted_agent_code.ts
```

### 2. Explicit Allowlist Flags
Grant capabilities selectively using `--allow-*` flags:

```bash
bee run --sandbox \
  --allow-read ./data \
  --allow-read /etc/hosts \
  --allow-write ./output.json \
  --allow-net api.openai.com:443 \
  --allow-listen 0.0.0.0:3000 \
  --allow-env NODE_ENV,API_KEY \
  app.ts
```

| Permission Flag | Description | Example |
| :--- | :--- | :--- |
| **`--allow-read <PATH>`** | Allow read access to exact path or directory | `--allow-read ./public` |
| **`--allow-write <PATH>`**| Allow write access to exact path or directory| `--allow-write /tmp/logs` |
| **`--allow-net <HOST>`** | Allow outbound network access to host or URL | `--allow-net api.github.com` |
| **`--allow-listen <HOST>`**| Allow binding network listeners to port | `--allow-listen 0.0.0.0:3000`|
| **`--allow-env <NAME>`** | Allow reading exact environment variables | `--allow-env PORT,DATABASE_URL`|
| **`--allow-run <CMD>`** | Allow executing specific external binaries | `--allow-run git` |

### 3. JSON Policy Files
For automated deployments or container environments, save permission sets in a JSON policy file:

```json
// policy.json
{
  "permissions": {
    "deny_fs": false,
    "deny_net": false,
    "allow_read": ["./src", "./public"],
    "allow_write": ["/tmp"],
    "allow_net": ["127.0.0.1", "cdn.example.com"]
  }
}
```

Load policy directly via CLI:
```bash
bee run --policy policy.json app.ts
```

### 4. Security Audit Logging (`--audit-log`)
Log every security decision made by the ResourceBroker to a JSONL audit trail:

```bash
bee run --sandbox --allow-read ./data --audit-log /var/log/bee_audit.jsonl app.ts
```

Sample audit entry:
```json
{"timestamp":1788756000000,"kind":"fs","action":"read","resource":"/etc/passwd","decision":"deny"}
{"timestamp":1788756000050,"kind":"fs","action":"read","resource":"./data/config.json","decision":"allow"}
```

---

## 4. Deterministic Sandboxing (`--seed` & `--freeze-time`)

For compliance audits, cryptographic validation, and AI agent regression testing, Beejs allows pinning random numbers and system clocks:

```bash
# Pin PRNG random seed and virtual ISO time
bee run \
  --seed 12345678 \
  --freeze-time "2026-09-07T12:00:00.000Z" \
  audit_report.ts
```

- **`Math.random()` & `crypto.getRandomValues()`** output deterministic pseudo-random sequences.
- **`Date.now()`, `new Date()`, and `performance.now()`** remain locked to the frozen timestamp, ensuring 100% reproducible execution across machines.
