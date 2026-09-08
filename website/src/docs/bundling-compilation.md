---
title: "Module Bundling & Standalone Compilation (Bundler 2.0 & SEA)"
subtitle: "Production multi-module static bundling and zero-dependency standalone executable generation"
group: "Developer Tooling"
id: "bundling-compilation"
---

Beejs provides two distribution and packaging tools:
1. **Bundler 2.0 (`bee bundle`)**: Resolves module dependency graphs into a single production JS bundle with minification, source maps, and import maps;
2. **SEA Compiler (`bee compile`)**: Packages your code into a **single, standalone, zero-dependency native executable binary** (Single Executable Application).

---

## 1. Module Bundler 2.0 (`bee bundle`)

Bundler 2.0 recursively parses static ASTs to construct a complete module graph, honoring relative imports and `node_modules` resolution rules.

### 1.1 Command Examples

```bash
# Basic bundling
$ bee bundle src/index.ts -o dist/bundle.js

# Production bundle (minify & generate sourcemaps)
$ bee bundle src/index.ts -o dist/bundle.min.js --minify --sourcemap

# Bundle with WICG Import Maps
$ bee bundle src/index.ts -o dist/bundle.js --import-map import_map.json
```

### 1.2 Key Architectural Highlights
- **Function-Level Scope Isolation**: Wraps each module in an isolated registry function with `__beejs_require__`, preventing scope collisions;
- **Built-in OXC Minifier**: Performs dead-code elimination, whitespace stripping, and identifier mangling in milliseconds;
- **SourceMap v3 Generation**: Produces accurate source mapping back to original TypeScript lines.

---

## 2. Standalone Application Compiler (`bee compile`)

Need to distribute a CLI tool or microservice directly to end users without requiring them to install Beejs, Node.js, or npm? `bee compile` packages your application into a self-contained executable.

### 2.1 Compiling a Binary

```bash
# Compile app.ts into native standalone executable 'myapp'
$ bee compile app.ts -o myapp

# Run directly on target machines or containers (zero prerequisites)
$ ./myapp
```

Output:
```text
📦 Compiling app.ts into standalone executable: myapp
✅ Executable created successfully (78.2 MB, chmod +x applied).
```

### 2.2 Magic Trailer Architecture

Beejs employs an **appended payload architecture with a magic trailer**:

```text
+───────────────────────────────────────────────────────────+
|               Beejs Core Runtime Binary (ELF / Mach-O)    |
+───────────────────────────────────────────────────────────+
|                   Bundled User Script Payload             |
+───────────────────────────────────────────────────────────+
| 8-byte Payload Size (u64)  |  Magic BEE_STANDALONE\0\0 (16B)|
+───────────────────────────────────────────────────────────+
```

1. **Sub-15ms Compilation**: Directly clones the host executable and appends the serialized user payload;
2. **Nanosecond Fast Boot**: During boot, Beejs inspects its own binary trailer. If `BEE_STANDALONE` is present, it directly runs the embedded payload in-memory, bypassing CLI parser overhead;
3. **Automatic Permissions**: Automatically applies `0o755` executable permissions on macOS and Linux.

---

## 3. Comparison with Alternatives

| Feature | Beejs (`bee compile`) | esbuild / webpack | pkg / ncc |
| :--- | :--- | :--- | :--- |
| **Standalone Native Binary** | **Native (`bee compile`)** | No (output is JS only) | Requires re-bundling |
| **Compilation Time** | **< 15ms** | ~100ms | 10s ~ 30s |
| **External Dependencies** | **None** | Requires Node.js | Requires Node.js |
| **Direct Execution** | **Yes (`./myapp`)** | No (`node bundle.js`) | Bloated binaries (200MB+) |
