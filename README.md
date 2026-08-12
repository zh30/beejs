# Beejs

[![Website](https://img.shields.io/badge/website-bee.zhanghe.dev-amber)](https://bee.zhanghe.dev)
[![Release](https://img.shields.io/badge/release-v0.1.0--repair--sprint-blue)](#current-status)
[![Runtime](https://img.shields.io/badge/runtime-Rust%20%2B%20V8-orange)](#why-beejs)
[![License](https://img.shields.io/badge/license-MIT-yellow.svg)](LICENSE)

Beejs is a JavaScript and TypeScript runtime built with Rust and V8. The active product path is the `bee` CLI over `MinimalRuntime`, with growing Node.js and Web API compatibility. Performance and compatibility numbers are only valid when backed by current, reproducible benchmarks and conformance scores.

Official Website & Documentation: [https://bee.zhanghe.dev](https://bee.zhanghe.dev)

---

## Why Beejs

- **Rust + V8 execution**: Default runtime path is `src/runtime_minimal.rs` with Tokio-backed timers and I/O work in progress.
- **TypeScript support**: `.ts` files are transpiled before execution (self-hosted compiler today; cache and TSX coverage are active work items).
- **Fail-closed permissions**: Granular broker flags such as `--deny-fs` / `--deny-net`.
- **Node.js & Web API surface (incremental)**: Partial `fs` / `http` / `crypto` / `fetch` / Streams / WebCrypto. See [Current Scope](docs/CURRENT_SCOPE.md) and `tests/conformance/` for what is actually verified.

---

## Current Status

Package version is **`0.1.0`**. Treat [Current Scope](docs/CURRENT_SCOPE.md) as the only user-facing capability boundary. Historical `docs/STAGE_*` reports and old “357/357” / “1000-5000x” claims are not current facts.

Compatibility progress is tracked by the Node conformance scorecard under `tests/conformance/`. Performance claims require the scripts in `benchmarks/` against the binary you just built.

---

## Quick Install

Install the latest release with the official one-line script:

```bash
curl -fsSL https://bee.zhanghe.dev/install.sh | sh
```

Or specify a custom version tag or installation directory:

```bash
curl -fsSL https://bee.zhanghe.dev/install.sh | BEEJS_VERSION=v0.1.0-repair-sprint sh
curl -fsSL https://bee.zhanghe.dev/install.sh | BEEJS_INSTALL_DIR="$HOME/.local/bin" sh
```

### Build from Source

You can also build the optimized release binary directly using Cargo:

```bash
git clone https://github.com/zh30/beejs.git
cd beejs
cargo build --release
./target/release/bee --version
```

---

## Quick Start

Evaluate JavaScript inline:

```bash
bee eval "1 + 1"
```

Run a JavaScript or TypeScript file natively:

```bash
bee run examples/basics/hello_world.js
bee run examples/basics/typescript_demo.ts
```

Start the interactive REPL:

```bash
bee repl
```

Run test suite smoke checks:

```bash
bee test examples/testing/math.test.js
```

---

## CLI Overview

Stable commands:

```bash
bee --version
bee --help
bee run <file> [args...]
bee eval <code>
bee repl
bee version
```

Preview and experimental commands:

```bash
bee test [file]
bee bundle <entry> --outfile dist/bundle.js
bee debug <file>
bee serve --host localhost --port 3000
bee init [name]
bee create my-app js
bee create my-ts-app ts
bee add <package>
bee remove <package>
bee install
bee prune
bee bunx <package> [args...]
bee upgrade [package]
```

See [CLI usage guide](docs/CLI_USAGE_GUIDE.md) for full options and examples.

---

## Native TypeScript & WebAssembly

Beejs handles `.ts`, `.tsx`, and `.wasm` modules natively:

```bash
bee run examples/basics/typescript_demo.ts
```

For WebAssembly, Beejs exposes full `WebAssembly.compile`, `WebAssembly.Instance`, and shared `WebAssembly.Memory` buffer APIs through V8 JIT compilation.

---

## Core Quality & Verification

Beejs maintains strict quality gates across Rust integration suites:

```bash
cargo build --release
cargo test --lib
cargo test --test wasm_v8_execution_tests
cargo test --test http_streaming_response_tests
cargo clippy --all-targets -- -D warnings
```

---

## Documentation & Links

- **Official Website**: [https://bee.zhanghe.dev](https://bee.zhanghe.dev)
- [Current Scope](docs/CURRENT_SCOPE.md)
- [Documentation Index](docs/README.md)
- [CLI Usage Guide](docs/CLI_USAGE_GUIDE.md)
- [Project Skill Guide](.gemini/skills/deploy-cloudflare-website/SKILL.md)
- [Examples](examples/)
- [License](LICENSE)

---

## License

Beejs is released under the [MIT License](LICENSE).
