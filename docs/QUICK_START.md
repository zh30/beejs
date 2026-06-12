# Beejs Quick Start

Beejs v0.1 is a Rust + V8 JavaScript/TypeScript runtime. This quick start only
documents behavior from the current CLI in `src/main.rs`. See
[Current Scope](CURRENT_SCOPE.md) for the full Stable / Preview / Experimental /
Historical capability map.

## Build From Source

The source checkout is the preferred path for the current repository state:

```bash
git clone https://github.com/zh30/beejs.git
cd beejs
cargo build --release
./target/release/bee --version
```

During development, you can also use Cargo directly:

```bash
cargo run -- --version
```

## Release Install

Use the install script only after the matching GitHub Release artifacts and
`install.sh` are published for the version you want:

```bash
curl -fsSL https://raw.githubusercontent.com/zh30/beejs/main/install.sh | sh
bee --version
```

## Run Stable Commands

Evaluate a JavaScript snippet:

```bash
./target/release/bee eval "1 + 1"
```

Run JavaScript:

```bash
./target/release/bee run examples/basics/hello_world.js
```

Start the REPL:

```bash
./target/release/bee repl
```

If you are iterating without a release build, the same commands can be run
through Cargo:

```bash
cargo run -- eval "1 + 1"
cargo run -- run examples/basics/hello_world.js
cargo run -- repl
```

## Try Preview And Experimental Paths

Run TypeScript through the preview built-in transpilation path:

```bash
./target/release/bee run examples/basics/typescript_demo.ts
```

Run an experimental test-file smoke command:

```bash
./target/release/bee test examples/testing/math.test.js
```

When using an installed release, replace `./target/release/bee` with `bee`.

## Current Scope

Beejs v0.1 is a core runtime release for experimentation, examples, and
compatibility work. It is not documented as a complete replacement for Node.js,
Bun, or Deno.

Historical Stage documents are historical materials, not current product
capability proof. Performance claims should be based on fresh benchmark runs in
the current repository, with the command, commit, build mode, environment, and
output validation recorded.

Useful verification commands:

```bash
cargo build
cargo test --lib
cargo test --test timers_enhanced_tests
cargo run -- run examples/basics/hello_world.js
```
