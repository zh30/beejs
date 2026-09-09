# GitHub Actions 是否达到「语言运行时配套设施」水准

## Verdict

**尚未达到**完整的「语言运行时配套设施」水准。当前流水线覆盖了语言运行时的核心 PR 门禁和 `v*` 正式 GitHub Release，但还缺 Windows、失败即红的 feature 矩阵、镜像仓库发布，以及默认的 crates.io / 包注册表发布。

这是有资格的判断：基础设施骨架在，不能当成 Node / Deno / Bun 同级的发布与兼容性网。

## Live workflows

| Workflow file | Display name | Trigger | What it actually does |
|---|---|---|---|
| `.github/workflows/ci.yml` | **CI** | `push` `main`, `pull_request` → `main`, `workflow_dispatch`, `tags: v*` (CI jobs only) | Format, clippy (`-D warnings`), `cargo test`, feature compile-only matrix, Node conformance scorecard, `bee test examples/testing`, `cargo package --list`, ubuntu + macOS release-binary smoke tests |
| `.github/workflows/release-assets.yml` | **Release Assets** | `push` tags `v*`, plus `workflow_dispatch` | Cross-compile `bee` for linux x86_64, macOS arm64, macOS x86_64; SHA-256 checksums; `scripts/generate_release_notes.py`; **non-draft** GitHub Release attaching `bee-*.tar.gz` |
| `.github/workflows/docker.yml` | **Docker** | `workflow_dispatch` only | Build the `runtime` image locally on the runner and `docker run --rm beejs:ci --version`. Does **not** push to Docker Hub / GHCR |
| `.github/workflows/performance-tests.yml` | **Performance** | Monday 02:00 UTC cron, `workflow_dispatch` | `cargo test --release --lib benchmark` on ubuntu-latest; upload logs. Not a PR gate |

## Compared with a typical language-runtime CI

A language runtime usually gates PRs, ships multi-OS binaries on version tags, runs a conformance suite, and publishes installable artifacts.

| Expectation | Beejs today |
|---|---|
| PR gates (fmt / clippy / tests) | Yes, on **CI** (`ubuntu-latest` only for the main `checks` job) |
| Multi-OS release archives | Yes, on **Release Assets**: linux x86_64, macOS arm64, macOS x86_64 |
| Conformance | Partial: Node scorecard in **CI**; WinterTC coverage is `cargo test --test wintertc_compliance_tests`, not a separate WPT job |
| Publish | Optional crates.io on **Release Assets** only if `CARGO_REGISTRY_TOKEN` is set; no npm / jsr; Docker image is not published |

## Named gaps

1. **No Windows** in either PR checks or release assets (`x86_64-pc-windows-msvc` / `bee-*.zip` is absent).
2. **Feature matrix is not fail-closed**: `ci.yml` runs `cargo check --features ai|benchmarks|observability` and turns compile failure into `::warning::`.
3. **Docker** never publishes: dispatch-only, `load: true`, tag `beejs:ci`, no registry login.
4. **Performance** is weekly, not a PR / release gate, and does not fail the default merge path.
5. **Default publish is missing**: crates.io runs on `v*` only when `CARGO_REGISTRY_TOKEN` is present; there is no npm / jsr job.
6. **PR OS coverage is thin**: default `checks` is Ubuntu-only; macOS is smoke-tested in `release-build`, not the full clippy + conformance suite.

Until Windows archives, a fail-closed feature matrix, and at least one published install channel (crates.io or a container registry) land on `v*` tags, GitHub Actions here is a solid project CI — not yet a full language-runtime supporting facility.

可执行待办见 [INFRASTRUCTURE_CAPABILITY_BACKLOG.md](./INFRASTRUCTURE_CAPABILITY_BACKLOG.md)。
