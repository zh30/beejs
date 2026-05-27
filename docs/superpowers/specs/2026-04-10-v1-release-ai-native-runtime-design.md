# Beejs v1.0 Release: AI-Native Runtime Design

## Problem Statement

Beejs has 241K lines of Rust code, but only 11 of 82 declared modules compile. 86% of the codebase is dead weight. The one working module (`runtime_minimal.rs`) is a 16,085-line monolith. The event loop is fake (polling instead of microtask-driven), async I/O does not exist, and there is no real module system. The project cannot run real-world JavaScript applications.

Meanwhile, no existing JS runtime (Node, Deno, Bun) is designed for AI workloads. They are general-purpose runtimes where AI features are bolted on as libraries. Beejs has an opportunity to be the first runtime where AI is a first-class primitive.

## Design Principles

1. **Ruthless deletion**: Any code that doesn't compile gets deleted. Any feature that isn't essential for v1.0 gets removed.
2. **Real over aspirational**: Every shipped feature must have real, working, tested code. No stubs. No `println!` placeholders.
3. **AI-native from the core**: Streaming, tool calling, and structured output are not libraries — they are runtime primitives, as fundamental as `setTimeout`.
4. **Node.js compatible**: Must run existing npm packages with zero migration cost. This means real ESM/CJS module resolution, real async APIs, real process/fs/path/crypto compatibility.
5. **Architecture over features**: A small runtime with clean architecture beats a large runtime with broken architecture.

## Phase 1: The Great Cleanup (Days 1-2)

### Delete dead code

- **46+ zombie module directories** in `src/`: quantum_computing, holographic, metaverse, neural_network, immersive_interaction, distributed_metaverse, ai_inference, aiops, cloud, enterprise, distributed, cloudnative, cloud_native, multilang, platform, simd, holographic, etc.
- **591 `.bak` files** in `src/`
- **58 `.disabled` test files** in `tests/`
- **100+ root-level scripts**: fix_*.py, bench_*.rs, test_*.rs, demo_*.rs, stage_*.rs scattered files
- **90+ STAGE_*.md progress docs** in `docs/`
- **Compiled binaries** in root: check_test, demo_error_handling, performance_test_bin, simple_test_stage95_bin, startup_benchmark, test_stage_28, etc.

### Clean lib.rs

Remove all `// pub mod X; // Temporarily disabled` comments. Only declare modules that actually exist and compile:

```rust
pub mod runtime;
pub mod nodejs_api;
pub mod web_api;
pub mod ai;
pub mod typescript;
pub mod testing;
pub mod package_manager;
pub mod watcher;
pub mod watcher_websocket;
pub mod cli;
```

### Result

Codebase shrinks from ~241K lines to ~60-70K lines. Every line compiles. Every module has a clear purpose.

## Phase 2: Core Runtime Restructure (Days 3-10)

### Break up runtime_minimal.rs

The 16,085-line monolith splits into:

```
src/runtime/
├── mod.rs              (200 lines)  Runtime trait + unified entry point
├── isolate.rs          (300 lines)  V8 isolate creation, configuration, lifecycle
├── context.rs          (500 lines)  V8 context setup, global object injection
├── event_loop.rs       (400 lines)  Real microtask-driven event loop
├── module_loader.rs    (600 lines)  ESM import + CJS require resolution
├── async_io.rs         (400 lines)  tokio-based async fs/net/timer dispatch
├── timers.rs           (500 lines)  setTimeout/setInterval/setImmediate/clear*
├── console.rs          (200 lines)  console.log/warn/error/table/timeEnd
├── process_api.rs      (400 lines)  process.env/argv/cwd/nextTick/exit
├── buffer.rs           (300 lines)  Buffer + ArrayBuffer + TypedArrays
└── snapshot.rs         (200 lines)  V8 snapshot warmup (from v8_snapshot/)
```

### Event Loop: The Critical Fix

**Current state (broken)**: `setTimeout` uses `std::thread::sleep` polling. `process.nextTick` executes immediately. No microtask queue. No I/O polling.

**New design**:

```rust
pub struct EventLoop {
    next_tick_queue: VecDeque<Task>,        // process.nextTick callbacks (highest priority)
    microtask_queue: VecDeque<Task>,        // Promise.then/catch/finally
    macrotask_queue: BinaryHeap<TimerTask>, // setTimeout, setInterval (by deadline)
    immediate_queue: VecDeque<Task>,        // setImmediate
    io_runtime: tokio::runtime::Runtime,   // async I/O driver
}

impl EventLoop {
    pub fn run(&mut self, isolate: &mut v8::Isolate, context: &v8::Global<v8::Context>) -> Result<()> {
        loop {
            // Phase 1: Drain all nextTick callbacks
            self.drain_next_tick(isolate, context);

            // Phase 2: Drain all microtasks (Promise callbacks)
            self.drain_microtasks(isolate, context);

            // Phase 3: Poll I/O (one tick of tokio)
            self.poll_io()?;

            // Phase 4: Execute due macrotasks (timers)
            self.drain_due_timers(isolate, context);

            // Phase 5: Execute immediate callbacks
            self.drain_immediates(isolate, context);

            // Exit if nothing left
            if self.is_idle() { break; }
        }
        Ok(())
    }
}
```

Key behaviors matching Node.js semantics:
- `process.nextTick` callbacks always run before Promise microtasks
- Timers are sorted by deadline, not FIFO
- I/O polling happens between timer checks (like libuv)
- The loop exits naturally when no pending work remains

### Module Loader: Real ESM/CJS

**Current state**: `require()` is a stub that prints "require not implemented".

**New design**:

```rust
pub struct ModuleLoader {
    cache: HashMap<String, LoadedModule>,
    node_modules_paths: Vec<PathBuf>,
}

struct LoadedModule {
    exports: v8::Global<v8::Value>,
    path: PathBuf,
    loaded: bool,
}

impl ModuleLoader {
    /// CJS require() resolution following Node.js algorithm
    pub fn resolve_require(&mut self, specifier: &str, parent: &Path) -> Result<PathBuf> {
        // 1. If starts with ./ or ../ or / — resolve as file
        // 2. If starts with @ or no prefix — resolve as node_modules package
        //    a. Check parent/node_modules/specifier
        //    b. Walk up directories checking node_modules
        // 3. Check package.json "main" / "exports" fields
        // 4. Try extensions: .js, .json, .node
        // 5. Try directory: specifier/index.js
    }

    /// ESM import resolution
    pub fn resolve_import(&mut self, specifier: &str, parent: &Path) -> Result<ModuleUrl> {
        // 1. "file:" URLs → resolve as file path
        // 2. "data:" URLs → inline source
        // 3. Bare specifiers → node_modules resolution
        // 4. Check import map if configured
    }
}
```

### Async I/O: tokio Integration

All I/O operations dispatch through tokio, bridged to V8 via callbacks:

```rust
// async_io.rs
pub fn register_async_ops(scope: &mut v8::HandleScope, global: v8::Local<v8::Object>) {
    // fs.readFile(path, callback) → tokio::fs::read → callback(null, data)
    // fs.writeFile(path, data, callback) → tokio::fs::write → callback(null)
    // http.request(options, callback) → reqwest::request → callback(response)
    // net.connect(port, host, callback) → tokio::net::TcpStream → callback(socket)
}

// Bridge pattern:
// 1. JS calls async API → creates tokio task
// 2. Tokio completes task → sends result via mpsc channel
// 3. Event loop picks up result on next I/O poll → invokes JS callback
```

## Phase 3: AI-Native Layer (Days 11-18)

### Architecture

```
src/ai/
├── mod.rs                (100 lines)  AI module entry + Beejs.ai global
├── streaming.rs          (400 lines)  StreamingResponse for SSE/NDJSON
├── tool_calling.rs       (500 lines)  Tool/function calling primitives
├── structured_output.rs  (300 lines)  JSON Schema validation + constrained output
├── tokenizer.rs          (300 lines)  Token counting + text chunking
├── embedding.rs          (200 lines)  Cosine similarity + vector search
└── context_window.rs     (300 lines)  Token-aware context window management
```

### JS API Design

```javascript
// Global: Beejs.ai
const ai = Beejs.ai;

// 1. Streaming — first-class async iterator
const stream = ai.stream("https://api.openai.com/v1/chat/completions", {
  method: "POST",
  headers: { Authorization: `Bearer ${process.env.OPENAI_API_KEY}` },
  body: JSON.stringify({
    model: "gpt-4",
    stream: true,
    messages: [{ role: "user", content: "Explain quantum computing" }],
  }),
});

for await (const chunk of stream) {
  process.stdout.write(chunk.content);
}

// 2. Tool calling — structured function dispatch
const result = await ai.useTools(prompt, {
  model: "gpt-4",
  tools: [{
    name: "get_weather",
    description: "Get current weather for a location",
    parameters: {
      type: "object",
      properties: {
        location: { type: "string" },
        unit: { type: "string", enum: ["celsius", "fahrenheit"] },
      },
      required: ["location"],
    },
  }],
  execute: async (toolCall) => {
    if (toolCall.name === "get_weather") {
      return await fetchWeather(toolCall.arguments.location);
    }
  },
});

// 3. Structured output — JSON Schema validation
const user = await ai.parse(response, {
  type: "object",
  properties: {
    name: { type: "string" },
    age: { type: "number" },
  },
  required: ["name", "age"],
});
// Returns validated object or throws ValidationError

// 4. Token counting + context window management
// v1.0: character-approximation (1 token ≈ 4 chars English, 1-2 chars CJK)
// v1.1: optional tiktoken integration for exact counts
const tokens = ai.countTokens("Hello world"); // → 3
const chunks = ai.chunkText(longDocument, { maxTokens: 4096, overlap: 200 });
```

### Rust Implementation Strategy

The AI layer is pure Rust logic exposed to V8 via callback bindings (same pattern as existing `setup_*_api` functions). No external AI runtime dependencies — it provides building blocks, not model execution:

- `streaming.rs`: SSE parser + NDJSON parser + async iterator bridge to V8
- `tool_calling.rs`: JSON Schema validation (using `serde_json` + `schemars`), function dispatch
- `structured_output.rs`: JSON parsing + schema validation + error reporting
- `tokenizer.rs`: Character-approximation token counter (1 token ≈ 4 chars for English, 1-2 chars for CJK), with optional tiktoken integration for v1.1
- `embedding.rs`: Cosine similarity, top-K search on Float32Array
- `context_window.rs`: Sliding window with token budget tracking

## Phase 4: Node.js API Completion (Days 19-25)

### Priority APIs for npm compatibility

Must-have (v1.0 blockers):
- `fs` — readFile/readFileSync, writeFile/writeFileSync, readdir/readdirSync, stat/statSync, mkdir/mkdirSync, existsSync, unlinkSync, createReadStream, createWriteStream
- `path` — join, resolve, dirname, basename, extname, normalize, relative, isAbsolute
- `process` — env, argv, cwd, nextTick, exit, on, stdout, stderr, stdin
- `http` — createServer, request, get (async, with proper chunked response)
- `https` — same as http with TLS
- `crypto` — createHash, createHmac, randomBytes, pbkdf2Sync, webcrypto.subtle
- `url` — URL, URLSearchParams
- `events` — EventEmitter
- `stream` — Readable, Writable, Transform, Duplex
- `util` — promisify, callbackify, inspect, types

Important but not blockers:
- `os` — hostname, type, platform, arch, cpus, totalmem, freemem
- `dns` — resolve, lookup
- `net` — createServer, createConnection
- `child_process` — exec, spawn
- `buffer` — Buffer.from, Buffer.alloc, Buffer.concat

### Source of truth

All existing real implementations in `nodejs_core/` (15K lines) and `web_api/` (15K lines) are preserved and migrated to the new module structure. No rewriting of working code — only reorganization.

## Phase 5: Polish and Release (Days 26-30)

### Code quality

- Replace all `println!`/`eprintln!` debug output with `tracing` crate
- Remove all `#![allow(clippy::all)]` suppression — fix warnings properly
- Consistent error handling with `anyhow` + custom error types
- Remove Chinese debug messages from production code paths

### Testing

- Keep only tests that test real functionality
- Write 20 end-to-end integration tests covering:
  - Basic JS execution (arithmetic, strings, objects, arrays)
  - Async patterns (setTimeout, Promise, async/await, fetch)
  - File I/O (read, write, exists, mkdir)
  - HTTP (server + client)
  - Crypto (hash, encrypt, sign)
  - Module system (require, import)
  - AI primitives (streaming, tool calling, structured output)
  - TypeScript execution
- All tests must pass on CI

### CI/CD

- GitHub Actions: build (macOS, Linux, Windows) + test + release
- Binary release via GitHub Releases
- Install script: `curl -fsSL https://beejs.dev/install.sh | sh`
- npm package: `npm install -g beejs` (wrapper that downloads the binary)

### Documentation

- Rewrite README.md with real benchmarks and honest capability description
- API reference for core modules
- Getting started guide
- AI API reference

## Target Architecture (Post-v1.0)

```
src/
├── main.rs                     CLI entry (200 lines)
├── lib.rs                      Module registration (50 lines)
├── runtime/                     Core runtime (3,500 lines)
│   ├── mod.rs                   Runtime entry
│   ├── isolate.rs               V8 isolate management
│   ├── context.rs               V8 context + globals
│   ├── event_loop.rs            Microtask-driven event loop
│   ├── module_loader.rs         ESM/CJS module resolution
│   ├── async_io.rs              tokio async I/O bridge
│   ├── timers.rs                Timer APIs
│   ├── console.rs               Console API
│   ├── process_api.rs           Process global
│   ├── buffer.rs                Buffer + ArrayBuffer
│   └── snapshot.rs              V8 snapshot warmup
├── nodejs_api/                  Node.js compatibility (8,000 lines)
│   ├── mod.rs
│   ├── fs.rs                    File system (async + sync)
│   ├── path.rs                  Path utilities
│   ├── http.rs                  HTTP server/client
│   ├── crypto.rs                Crypto (Node + WebCrypto)
│   ├── stream.rs                Streams
│   ├── events.rs                EventEmitter
│   └── util.rs                  Utilities
├── web_api/                     Web standard APIs (6,000 lines)
│   ├── mod.rs
│   ├── fetch.rs                 fetch() + Request/Response
│   ├── streams.rs               Web Streams API
│   ├── url.rs                   URL + URLSearchParams
│   ├── blob.rs                  Blob + File
│   ├── websocket.rs             WebSocket client
│   └── crypto.rs               Web Crypto API (crypto.subtle)
├── ai/                          AI-native primitives (2,100 lines)
│   ├── mod.rs
│   ├── streaming.rs             SSE/NDJSON streaming
│   ├── tool_calling.rs          Function calling
│   ├── structured_output.rs     JSON Schema validation
│   ├── tokenizer.rs             Token counting
│   ├── embedding.rs             Vector similarity
│   └── context_window.rs        Context management
├── typescript/                   TypeScript compiler (preserved, 14K lines)
├── testing/                      Test framework (preserved, 5K lines)
├── package_manager.rs            Package manager (preserved, 1K lines)
├── watcher.rs                   File watcher (preserved)
└── watcher_websocket.rs          WebSocket hot reload (preserved)

Total target: ~40K lines of real, compiling, tested code
```

## Success Criteria for v1.0

1. **Can execute real JS**: `beejs run app.js` works for apps that use fs, http, crypto, path, process
2. **Async works**: `setTimeout`, `Promise`, `async/await`, `fetch` all work correctly
3. **Module system works**: `require()` and `import` resolve real npm packages
4. **AI primitives work**: Streaming, tool calling, structured output are functional APIs
5. **Test suite passes**: All integration tests pass on macOS and Linux
6. **CI/CD ships**: Binary releases for macOS (arm64/x64) and Linux (x64)
7. **Installable**: `curl | sh` install works; `npm install -g beejs` works
8. **Honest documentation**: README describes real capabilities, no inflated benchmarks

## Non-Goals for v1.0

- WASM runtime (can be added in v1.1)
- Bundler (can be added in v1.1)
- Debugger (can be added in v1.2)
- TypeScript type checking (v1.0 does transpilation only)
- Full Node.js API surface (cover top 15 modules, expand in v1.1+)
- Cloud/enterprise/distributed features (not relevant to v1.0)
- Quantum computing, holographic rendering, metaverse (never relevant)
