# Beejs

[![Release](https://img.shields.io/badge/release-v0.1.0-blue)](#current-status)
[![Runtime](https://img.shields.io/badge/runtime-Rust%20%2B%20V8-orange)](#why-beejs)
[![License](https://img.shields.io/badge/license-MIT-yellow.svg)](LICENSE)

Beejs is a JavaScript/TypeScript runtime built with Rust and V8. The v0.1 release focuses on a predictable source-built CLI for running scripts, evaluating snippets, using a REPL, and exploring an evolving TypeScript, Node.js, and Web API compatibility layer.

This repository also contains historical stage work, experiments, benchmarks, reports, and feature-gated modules. For current behavior, use `Cargo.toml`, `src/lib.rs`, `src/main.rs`, executable tests, and the [current scope](docs/CURRENT_SCOPE.md) as the source of truth.

## Why Beejs

- Rust-based runtime architecture with explicit module boundaries.
- V8-backed JavaScript execution.
- A focused stable CLI core for scripts, eval, and REPL sessions.
- Preview TypeScript transpilation before execution for `.ts` and `.tsx` entry files.
- Actively developed Node.js and Web API compatibility surfaces, with behavior tracked API by API.
- Experimental tooling for tests, bundling, serving, project setup, and lightweight package workflows.
- Testable examples and transparent release scope for contributors who want to work on runtime internals.

## Current Status

Beejs v0.1.0 is a core runtime release. It is suitable for experimenting with the runtime, learning how the internals fit together, running repository examples, trying scripting workflows, and contributing to compatibility work.

It is not yet positioned as a complete replacement for established production JavaScript runtimes. Some modules in the repository are retained for staged development, are behind Cargo features, or are not part of the default public runtime surface.

See [Current Scope](docs/CURRENT_SCOPE.md) for the Stable, Preview, Experimental, and Historical capability map.

## Installation

The primary installation path for the current repository state is to build from source:

```bash
git clone https://github.com/zh30/beejs.git
cd beejs
cargo build --release
./target/release/bee --version
```

When public GitHub Release artifacts are available, the release automation is configured to produce prebuilt archives for:

- macOS x86_64
- macOS arm64
- Linux x86_64

After those artifacts are published and `install.sh` is available from the default branch, install the latest published release with:

```bash
curl -fsSL https://raw.githubusercontent.com/zh30/beejs/main/install.sh | sh
```

Install a specific release or choose a custom install directory:

```bash
curl -fsSL https://raw.githubusercontent.com/zh30/beejs/main/install.sh | BEEJS_VERSION=v0.1.0 sh
curl -fsSL https://raw.githubusercontent.com/zh30/beejs/main/install.sh | BEEJS_INSTALL_DIR="$HOME/.local/bin" sh
```

## Quick Start

From a source checkout, evaluate JavaScript:

```bash
./target/release/bee eval "1 + 1"
```

Run a JavaScript file:

```bash
./target/release/bee run examples/basics/hello_world.js
```

Run a TypeScript file through the preview transpilation path:

```bash
./target/release/bee run examples/basics/typescript_demo.ts
```

Start the REPL:

```bash
./target/release/bee repl
```

If `bee` is installed on your `PATH`, you can replace `./target/release/bee` with `bee`.

Run an experimental test-file smoke command:

```bash
./target/release/bee test examples/testing/math.test.js
```

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

Preview and experimental commands are present in the CLI, but their behavior should be checked against [Current Scope](docs/CURRENT_SCOPE.md) before being treated as product capability:

```bash
bee test [file]
bee bundle <entry> --outfile dist/bundle.js
bee debug <file>
bee serve --host localhost --port 3000
bee init [name]
bee create js my-app
bee create ts my-ts-app
bee add <package>
bee remove <package>
bee install
bee prune
bee bunx <package> [args...]
bee upgrade [package]
```

The global `--verbose` flag belongs before the subcommand:

```bash
bee --verbose run examples/basics/hello_world.js
```

See the [CLI usage guide](docs/CLI_USAGE_GUIDE.md) for command options and additional examples, and the [current scope](docs/CURRENT_SCOPE.md) for stability level.

## TypeScript Preview

Beejs reads `.ts` and `.tsx` files through the TypeScript module before passing JavaScript to the runtime:

```bash
bee run examples/basics/typescript_demo.ts
```

The TypeScript path is intended for direct runtime execution of supported syntax. Compatibility, diagnostics, source maps, and TSX behavior are evolving with the rest of the project.

## Testing Status

Beejs includes test framework code under `src/testing/` and exposes a `bee test` command, but the full Jest-style runner contract is still experimental. Treat CLI test commands as smoke checks unless a behavior is covered by current executable tests:

```bash
bee test
bee test examples/testing/math.test.js
bee test examples/testing/math.test.js --test-name-pattern "adds"
bee test examples/testing/math.test.js --bail
bee test examples/testing/math.test.js --timeout 10
```

Repository-level Rust tests live under `tests/` and are the stronger quality gate today:

```bash
cargo test --lib
cargo test --test beejs_core_tests
cargo test --test cli_release_tests
```

## What Works Today

The default build currently includes these major areas, grouped by current stability. See [Current Scope](docs/CURRENT_SCOPE.md) for definitions and graduation rules.

Stable:

- `src/main.rs`: the active Cargo binary entry for the `bee` CLI.
- `src/runtime_minimal.rs`: the V8 execution runtime used by the CLI.

Preview:

- `src/typescript/`: TypeScript transpilation used for `.ts` and `.tsx` entry-file execution.
- `src/nodejs_core/`: Node.js compatibility work, including modules such as `fs`, `crypto`, `events`, `buffer`, `path`, `os`, `url`, `dns`, `process`, timers, streams, HTTP, networking, readline, and CommonJS `require`.
- `src/web_api/`: Web API compatibility work, including fetch, WebSocket, Web Crypto, URL, events, FormData, Abort, Blob, timers, encoding, performance, streams, compression, structured clone, workers, service workers, broadcast channels, and message channels.
- `src/watcher.rs` and `src/watcher_websocket.rs`: watch and hot reload support.

Experimental:

- `src/testing/`: Jest-style framework work and V8 test execution support.
- `src/package_manager.rs`: lightweight package management commands.
- CLI tooling such as `bundle`, `debug`, `serve`, `init`, `create`, `bunx`, and dependency management commands.

Feature-gated and staged modules such as AI, observability, enterprise, cloud-native, and multi-language work are not default public commitments until their feature builds and behavior are verified.

## Performance And Feature Gates

Performance numbers in current user-facing docs must come from fresh, reproducible commands run against the current checkout or release. Historical stage benchmark reports are design context and should not be cited as current performance proof without rerunning the benchmark and recording the command, date, commit, build mode, hardware, and output validation.

The default Cargo feature set is empty. Optional features such as `ai`, `benchmarks`, `cloudnative`, `enterprise`, `multilang`, `observability`, and `tch` are Experimental unless the relevant feature check was run successfully in the current branch:

```bash
cargo check --features observability
cargo check --features enterprise
cargo check --features ai
```

## Roadmap

- Improve runtime stability and isolate lifecycle behavior.
- Broaden Node.js and Web API compatibility with executable tests.
- Refine the TypeScript execution path and diagnostics.
- Improve package management workflows and project setup commands.
- Expand documentation and examples around supported behavior.
- Graduate optional feature modules only after build and behavior verification.

## Development

Common build and quality commands:

```bash
cargo build
cargo build --release
cargo test --lib
cargo test --test beejs_core_tests
cargo test --test cli_release_tests
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

The Makefile also provides common shortcuts:

```bash
make build
make test
make fmt
make lint
```

When changing feature-gated modules, also run the relevant feature check, for example:

```bash
cargo check --features observability
cargo check --features enterprise
cargo check --features ai
```

If a historical feature is not currently buildable, call that out in the pull request rather than documenting it as a default capability.

## Documentation And Examples

- [Current scope](docs/CURRENT_SCOPE.md)
- [Documentation index](docs/README.md)
- [CLI usage guide](docs/CLI_USAGE_GUIDE.md)
- [Examples](examples/)
- [License](LICENSE)

Historical `docs/STAGE_*`, `docs/IMPLEMENTATION_PLAN_STAGE_*`, stage completion reports, and stage benchmark reports are historical materials. They may describe goals, past experiments, higher staged version numbers, or unrevalidated performance results rather than the current v0.1 default build.

## Contributing

Contributions should be scoped to the current runtime surface unless a change intentionally works on a feature-gated module. Before opening a pull request, include:

- The change scope and affected modules.
- Commands or tests that were run.
- Any behavior that was not verified.
- Notes for feature-gated work if the relevant feature does not currently compile.

For CLI behavior, runtime APIs, Node.js compatibility, Web API compatibility, TypeScript execution, or test framework changes, prefer executable tests over example-only changes.

## Security And Reporting

For security-sensitive issues, avoid publishing exploit details before maintainers have had time to triage. Use the repository's private vulnerability reporting channel if available, or open a minimal issue asking for a secure contact path.

## License

Beejs is released under the [MIT License](LICENSE).
