#!/bin/sh
set -e

BEEJS_REPO_DEFAULT="zh30/beejs"
BEEJS_INSTALL_DIR_DEFAULT="${HOME}/.beejs/bin"

BEEJS_REPO="${BEEJS_REPO:-$BEEJS_REPO_DEFAULT}"
BEEJS_INSTALL_DIR="${BEEJS_INSTALL_DIR:-$BEEJS_INSTALL_DIR_DEFAULT}"

usage() {
  cat <<'USAGE'
Beejs installer

Usage:
  curl -fsSL https://raw.githubusercontent.com/zh30/beejs/main/install.sh | sh

Environment variables:
  BEEJS_VERSION     Version tag to install (example: v0.1.0 or 0.1.0)
  BEEJS_INSTALL_DIR Install directory (default: ~/.beejs/bin)
  BEEJS_REPO        GitHub repo (default: zh30/beejs)

Examples:
  BEEJS_VERSION=v0.1.0 sh install.sh
  BEEJS_INSTALL_DIR=~/.local/bin sh install.sh
USAGE
}

if [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; then
  usage
  exit 0
fi

fail() {
  echo "beejs install: $1" >&2
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1
}

if need_cmd curl; then
  http_get() { curl -fsSL "$1"; }
  http_download() { curl -fsSL "$1" -o "$2"; }
elif need_cmd wget; then
  http_get() { wget -qO- "$1"; }
  http_download() { wget -qO "$2" "$1"; }
else
  fail "curl or wget is required"
fi

resolve_platform() {
  os=$(uname -s)
  arch=$(uname -m)

  case "$os" in
    Darwin) os="apple-darwin" ;;
    Linux) os="unknown-linux-gnu" ;;
    *) fail "unsupported OS: $os" ;;
  esac

  case "$arch" in
    x86_64|amd64) arch="x86_64" ;;
    arm64|aarch64) arch="aarch64" ;;
    *) fail "unsupported architecture: $arch" ;;
  esac

  echo "${arch}-${os}"
}

resolve_version() {
  if [ -n "${BEEJS_VERSION:-}" ]; then
    version="$BEEJS_VERSION"
  else
    api_url="https://api.github.com/repos/${BEEJS_REPO}/releases/latest"
    json=$(http_get "$api_url") || fail "unable to fetch latest release"
    version=$(printf "%s" "$json" | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n 1)
    [ -n "$version" ] || fail "unable to resolve latest version"
  fi

  case "$version" in
    v*) echo "$version" ;;
    *) echo "v$version" ;;
  esac
}

install_binary() {
  target="$1"
  version_tag="$2"

  tmpdir=$(mktemp -d 2>/dev/null || mktemp -d -t beejs)
  archive="$tmpdir/beejs.tar.gz"
  url="https://github.com/${BEEJS_REPO}/releases/download/${version_tag}/beejs-${version_tag}-${target}.tar.gz"

  trap 'rm -rf "$tmpdir"' EXIT INT TERM

  echo "Downloading ${url}"
  http_download "$url" "$archive" || fail "download failed"

  tar -xzf "$archive" -C "$tmpdir" || fail "failed to extract archive"

  if [ -f "$tmpdir/beejs" ]; then
    src="$tmpdir/beejs"
  else
    src=$(find "$tmpdir" -type f -name beejs | head -n 1)
  fi

  [ -n "${src:-}" ] || fail "beejs binary not found in archive"

  mkdir -p "$BEEJS_INSTALL_DIR"
  cp "$src" "$BEEJS_INSTALL_DIR/beejs"
  chmod +x "$BEEJS_INSTALL_DIR/beejs"
}

ensure_path() {
  install_dir="$1"

  case ":$PATH:" in
    *":$install_dir:"*) return 0 ;;
  esac

  shell_name=$(basename "${SHELL:-sh}")
  profile=""

  case "$shell_name" in
    zsh) profile="$HOME/.zshrc" ;;
    bash)
      if [ -f "$HOME/.bashrc" ]; then
        profile="$HOME/.bashrc"
      elif [ -f "$HOME/.bash_profile" ]; then
        profile="$HOME/.bash_profile"
      else
        profile="$HOME/.bashrc"
      fi
      ;;
    fish) profile="$HOME/.config/fish/config.fish" ;;
    *) profile="$HOME/.profile" ;;
  esac

  if [ -f "$profile" ] && grep -qs "$install_dir" "$profile"; then
    return 0
  fi

  mkdir -p "$(dirname "$profile")" 2>/dev/null || true

  if [ "$shell_name" = "fish" ]; then
    line="set -gx PATH \"$install_dir\" \$PATH"
  else
    line="export PATH=\"$install_dir:\$PATH\""
  fi

  printf "\n# Beejs\n%s\n" "$line" >> "$profile"
}

main() {
  target=$(resolve_platform)
  version_tag=$(resolve_version)

  install_binary "$target" "$version_tag"
  ensure_path "$BEEJS_INSTALL_DIR"

  echo "Beejs ${version_tag} installed to ${BEEJS_INSTALL_DIR}/beejs"
  echo "Restart your shell or run:"
  echo "  export PATH=\"${BEEJS_INSTALL_DIR}:\$PATH\""
  echo "Verify with: beejs --version"
}

main
