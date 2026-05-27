# Beejs Quick Start

Beejs v0.1 is a Rust + V8 JavaScript/TypeScript runtime. This quick start only
documents behavior verified by the current CLI in `src/main.rs`.

## Install Or Build

Install the latest release:

```bash
curl -fsSL https://raw.githubusercontent.com/zh30/beejs/main/install.sh | sh
bee --version
```

Build from source:

```bash
git clone https://github.com/zh30/beejs.git
cd beejs
cargo build --release
./target/release/bee --version
```

## Run Code

Evaluate a snippet:

```bash
bee eval "1 + 1"
```

Run JavaScript:

```bash
bee run examples/basics/hello_world.js
```

Run TypeScript through the built-in transpilation path:

```bash
bee run examples/basics/typescript_demo.ts
```

Start the REPL:

```bash
bee repl
```

Run a test file:

```bash
bee test examples/testing/math.test.js
```

When using a source checkout before installation, replace `bee` with
`./target/release/bee`.

## Current Scope

Beejs v0.1 is a core runtime release for experimentation, examples, and
compatibility work. It is not documented as a complete replacement for Node.js,
Bun, or Deno, and performance claims should be based on fresh benchmark runs in
the current repository.

Useful verification commands:

```bash
cargo build
cargo test --lib
cargo test --test timers_enhanced_tests
cargo run -- run examples/basics/hello_world.js
```
