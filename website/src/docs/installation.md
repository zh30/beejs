---
title: "Installation & Setup"
subtitle: "Quickly install Beejs on macOS and Linux, or compile from source"
group: "Getting Started"
id: "installation"
---

## Quick Install (Recommended)

On macOS or Linux, run the official one-line install script in your terminal:

```bash
curl -fsSL https://bee.zhanghe.dev/install.sh | sh
```

### What the Install Script Does
1. **Detects Environment**: Automatically identifies your OS (macOS / Linux) and CPU architecture (Apple Silicon `arm64` or Intel `x86_64`).
2. **Fetches Prebuilt Archive**: Downloads the release archive optimized with `-O3` and verifies binary integrity.
3. **Deploys Binary**: Unpacks the `bee` executable into `~/.bee/bin/bee`.
4. **Configures PATH**: Updates your active shell profile (`~/.zshrc`, `~/.bashrc`, etc.) with `export PATH="$HOME/.bee/bin:$PATH"`.

After installation finishes, either restart your terminal or run `source ~/.zshrc` (or `source ~/.bashrc`) to start using `bee`.

---

## Verifying Installation

Verify that `bee` is properly installed and accessible:

```bash
# Print version info
bee --version

# Evaluate a quick JavaScript snippet
bee eval "console.log('🐝 Beejs is ready! Node compat: ' + process.version);"
```

You should see output similar to:
```text
bee 1.0.0
🐝 Beejs is ready! Node compat: v24.0.0
```

---

## Supported Platforms & Requirements

| Operating System | Architecture | Minimum Requirements | Prebuilt Release |
| :--- | :---: | :---: | :---: |
| **macOS (Apple Silicon)** | `arm64` (M1–M4) | macOS 12.0+ (Monterey or later) | ✅ Yes |
| **macOS (Intel)** | `x86_64` | macOS 12.0+ | ✅ Yes |
| **Linux** | `x86_64` | Kernel 4.18+, glibc 2.28+ | ✅ Yes |
| **Linux (ARM64)** | `aarch64` | Kernel 4.18+ | Build from source |
| **Windows** | `x86_64` | WSL 2 (Ubuntu / Debian) | ✅ Recommended via WSL 2 |

> [!NOTE]
> For Windows users, we strongly recommend **WSL 2** to take advantage of native Linux kernel performance and zero-copy asynchronous I/O.

---

## Building from Source

To customize Beejs, debug internal subsystems, or build for non-standard architectures, compile directly using the Rust toolchain.

### 1. Prerequisites
Ensure you have the following installed:
- **Rust** 1.80.0+ ([rustup.rs](https://rustup.rs))
- **Clang / LLVM** (required for V8 C++ bindings)
- **Python 3** (build helper scripts)

On Ubuntu / Debian:
```bash
sudo apt update && sudo apt install -y build-essential clang llvm git curl cmake python3
```

On macOS:
```bash
xcode-select --install
```

### 2. Clone and Build
```bash
git clone https://github.com/zh30/beejs.git
cd beejs

# Release build with full optimizations
cargo build --release

# The compiled binary is output to:
./target/release/bee --version
```

### 3. Install to Global PATH
```bash
sudo cp ./target/release/bee /usr/local/bin/
bee --version
```

---

## Environment Variables

| Variable | Default | Purpose |
| :--- | :---: | :--- |
| `BEE_WORKERS` | `1` | Default number of worker threads for parallel HTTP execution |
| `BEE_HOME` | `~/.bee` | Base directory for package caches and downloads |
| `BEE_AUDIT_LOG` | None | File path for writing JSONL security sandbox audit records |
| `BEE_LOG` | `info` | Log verbosity (`error`, `warn`, `info`, `debug`, `trace`) |

Example for `~/.zshrc` or Dockerfile:
```bash
export BEE_WORKERS=8
export BEE_LOG=warn
```

---

## Uninstallation

To uninstall Beejs, delete the binary directory and remove the PATH entry from your shell configuration:

```bash
# 1. Remove binary and caches
rm -rf ~/.bee

# 2. Remove the PATH export line from ~/.zshrc or ~/.bashrc
```
