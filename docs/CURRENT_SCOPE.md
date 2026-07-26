# Current Scope

Last reviewed: 2026-06-12

This page is the user-facing capability boundary for the current Beejs checkout. It is intentionally narrower than many historical stage reports in this repository.

## Source Of Truth

Use these files and checks as the current fact sources:

- `Cargo.toml`: package version, enabled binary targets, Cargo features, and dependencies.
- `src/lib.rs`: the default library module surface and feature-gated modules.
- `src/main.rs`: the active `bee` CLI entrypoint.
- Executable tests and smoke commands run in the current checkout.

Current facts from those sources:

- Package version is `0.1.0`.
- The active Cargo binary is `bee`, built from `src/main.rs`.
- Default Cargo features are empty: `default = []`.
- The default runtime path used by the CLI is `src/runtime_minimal.rs`.
- Modules present in the repository are not automatically public product capabilities. Many are staged, feature-gated, partially wired, or retained for historical context.

## Stability Levels

### Stable

Stable means the capability is part of the current v0.1 default scope, is reachable from the active `bee` binary or default library surface, and should be kept working by focused smoke tests or Rust tests.

Current stable scope:

- Build Beejs from source with Cargo.
- Inspect the CLI with `bee --help`, `bee --version`, or `bee version`.
- Evaluate simple JavaScript snippets with `bee eval <code>`.
- Run JavaScript files with `bee run <file>`.
- Use the basic REPL with `bee repl`.
- Use V8-backed execution through `src/runtime_minimal.rs` for repository examples and small scripts.

Stable does not mean Node.js, Bun, or Deno compatibility. It also does not imply a production support commitment.

### Preview

Preview means the capability is present in the default build and is useful for experiments, but its compatibility contract, diagnostics, edge cases, or test coverage are still being tightened.

Current preview scope:

- TypeScript and TSX entry files are accepted by the CLI and pass through the built-in TypeScript transpilation path before execution. The supported syntax subset and diagnostics are still evolving.
- Node.js compatibility modules under `src/nodejs_core/` are installed into the runtime, including areas such as `fs`, `crypto`, `events`, `buffer`, `path`, `os`, `url`, `dns`, `process`, timers, streams, HTTP, networking, readline, and CommonJS `require`. Treat these as compatibility work in progress unless a behavior is covered by current executable tests.
- Web API modules under `src/web_api/` are installed into the runtime, including areas such as fetch, WebSocket, Web Crypto, URL, events, FormData, Abort, Blob, timers, encoding, performance, streams, compression, structured clone, workers, service workers, broadcast channels, and message channels. Treat these as API-specific preview work, not blanket Web platform compatibility.
- Watch and hot reload code paths exist through `bee run --watch`, `src/watcher.rs`, and `src/watcher_websocket.rs`.

### Experimental

Experimental means the capability exists as code, command surface, module surface, design work, or historical implementation, but should not be presented as current product capability without fresh verification.

Current experimental scope:

- `bee test` and the Jest-style framework under `src/testing/`. The CLI command can execute test files and built-in smoke cases, but the full test-runner contract is still under repair.
- `bee bundle`, `bee debug`, `bee serve`, `bee init`, `bee create`, `bee add`, `bee remove`, `bee install`, `bee prune`, `bee bunx`, and `bee upgrade`.
- Lightweight package-management and project setup behavior, including resolver, lifecycle, supply-chain, and package execution paths.
- V8 snapshot, benchmarking helpers, performance reporting, memory/fallback/error support modules, and ecosystem-lite helpers beyond the behaviors covered by current tests.
- Optional Cargo features: `ai`, `benchmarks`, `cloudnative`, `enterprise`, `multilang`, `observability`, `tch`, and `verbose_logging`.

Experimental capabilities may be useful for contributors. They are not stable user promises.

### Historical

Historical means the document or code exists to preserve stage context, design intent, prior experiments, benchmark attempts, or migration notes.

Historical sources include:

- `docs/STAGE_*`
- `docs/IMPLEMENTATION_PLAN_STAGE_*`
- Stage completion reports, progress reports, and stage benchmark reports.
- Older performance comparison documents unless they include a current reproducible command, environment, commit, and validation status.
- Archived progress logs under `docs/archive/`.

Historical material may contain higher version numbers, production-readiness claims, performance multipliers, or broad compatibility statements. Those statements are not current Beejs product facts unless revalidated against the current checkout and reflected in this scope page or another current-status document that links back here.

## Performance Claims

Beejs does not currently publish a stable performance claim from this scope page.

Any public performance number must include:

- The date and commit or release tag.
- The exact command used to build and run the benchmark.
- Whether the binary was debug or release.
- Hardware, operating system, and relevant runtime versions.
- The benchmark harness and input files.
- Exit-code and output correctness checks, not timing alone.

Historical stage benchmark numbers are design context only. Do not cite them as current performance facts without rerunning and documenting the current command.

## Cargo Features

The default build uses no Cargo features. A feature-gated module is current only for the feature build that was actually checked.

Use focused checks such as:

```bash
cargo check --features observability
cargo check --features enterprise
cargo check --features ai
cargo check --features cloudnative
cargo check --features multilang
cargo check --features benchmarks
```

If a feature build fails or has not been checked in the current branch, document the related capability as Experimental, not Stable.

## Graduation Rule

Move a capability upward only when all of these are true:

- It is reachable through `src/main.rs` or a documented library API in `src/lib.rs`.
- Its command or API behavior is described without relying on historical stage reports.
- Current tests or smoke commands cover the documented behavior.
- Known limitations are documented next to the capability.
- Feature-gated work has a passing feature check for the relevant feature.
