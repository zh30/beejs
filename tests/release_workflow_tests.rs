use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn release_assets_yaml() -> String {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".github/workflows/release-assets.yml");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn ci_yaml() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".github/workflows/ci.yml");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn docker_yaml() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".github/workflows/docker.yml");
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

#[test]
fn tag_v_star_publishes_non_draft_release_with_five_bee_archives() {
    let yaml = release_assets_yaml();

    assert!(
        yaml.contains("tags: [\"v*\"]") || yaml.contains("tags: ['v*']"),
        "Release Assets must trigger on v* tags"
    );
    assert!(
        yaml.contains("startsWith(github.ref, 'refs/tags/v')"),
        "publish path must be gated on refs/tags/v"
    );
    assert!(
        yaml.contains("draft: false"),
        "GitHub Release must be non-draft"
    );
    assert!(
        yaml.contains("generate_release_notes.py") && yaml.contains("checksums.txt"),
        "release notes and SHA-256 checksums must be generated"
    );
    assert!(
        yaml.contains("cosign sign-blob") && yaml.contains("cdx.json"),
        "release must attach cosign signatures and a CycloneDX SBOM"
    );
    assert!(
        yaml.contains("x86_64-unknown-linux-gnu")
            && yaml.contains("aarch64-unknown-linux-gnu")
            && yaml.contains("aarch64-apple-darwin")
            && yaml.contains("x86_64-apple-darwin")
            && yaml.contains("x86_64-pc-windows-msvc"),
        "must ship linux gnu x64/arm64, macOS arm64/x64, and Windows x64"
    );
    assert!(
        yaml.contains("archive: zip") && yaml.contains("bee.exe"),
        "Windows asset must be a zip containing bee.exe"
    );
    assert!(
        yaml.contains("CARGO_REGISTRY_TOKEN is not set") || yaml.contains("skipping crates.io"),
        "missing crates.io token must be annotated, not silent success"
    );
}

#[test]
fn macos_x86_64_asset_job_uses_live_intel_runner() {
    let yaml = release_assets_yaml();

    assert!(
        !yaml.contains("macos-13"),
        "macos-13 was retired 2025-12-04; the Intel macOS asset job would never start"
    );

    let intel_block = yaml
        .split("include:")
        .nth(1)
        .expect("matrix include")
        .split("steps:")
        .next()
        .expect("steps");

    assert!(
        intel_block.contains("os: macos-15-intel")
            && intel_block.contains("target: x86_64-apple-darwin"),
        "x86_64-apple-darwin must run on macos-15-intel (GitHub-hosted Intel macOS 15)"
    );
}

#[test]
fn ci_gates_are_fail_closed_and_cover_oses() {
    let yaml = ci_yaml();
    assert!(
        !yaml.contains("::warning::feature"),
        "feature matrix must not swallow compile failures"
    );
    assert!(
        yaml.contains("wintertc_compliance_tests"),
        "CI must run WinterTC tests as a named step"
    );
    assert!(yaml.contains("windows-latest"), "CI must smoke Windows");
    assert!(
        yaml.contains("Library tests on macOS") || yaml.contains("cargo test --lib --tests"),
        "macOS must run tests, not only --version"
    );
    assert!(
        yaml.contains("audit-check") || yaml.contains("cargo audit") || yaml.contains("cargo deny"),
        "CI must run cargo-audit or cargo-deny"
    );
}

#[test]
fn docker_workflow_publishes_ghcr_on_v_tags() {
    let yaml = docker_yaml();
    assert!(
        yaml.contains("ghcr.io/zh30/beejs"),
        "image must target ghcr.io/zh30/beejs"
    );
    assert!(
        yaml.contains("tags: [\"v*\"]") || yaml.contains("tags: ['v*']"),
        "GHCR workflow must trigger on v* tags"
    );
    assert!(
        yaml.contains("push: true"),
        "GHCR workflow must push, not only load: true"
    );
    assert!(
        yaml.contains("--version"),
        "image job must smoke bee --version"
    );
}

#[test]
fn dependabot_covers_cargo_and_github_actions() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".github/dependabot.yml");
    let yaml = fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    assert!(yaml.contains("package-ecosystem: cargo"));
    assert!(yaml.contains("package-ecosystem: github-actions"));
}

#[test]
fn homebrew_formula_points_at_github_release_assets() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Formula/bee.rb");
    let formula =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    for asset in [
        "bee-v#{version}-aarch64-apple-darwin.tar.gz",
        "bee-v#{version}-x86_64-apple-darwin.tar.gz",
        "bee-v#{version}-aarch64-unknown-linux-gnu.tar.gz",
        "bee-v#{version}-x86_64-unknown-linux-gnu.tar.gz",
    ] {
        assert!(
            formula.contains(asset),
            "Homebrew formula missing asset {asset}"
        );
    }
    assert!(formula.contains("https://github.com/zh30/beejs/releases/download/"));
}

#[test]
fn generate_release_notes_detects_aarch64_linux_before_generic_linux() {
    let script =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/generate_release_notes.py");
    let output = Command::new("python3")
        .arg("-c")
        .arg(format!(
            r#"
import importlib.util
spec = importlib.util.spec_from_file_location("grn", r"{script}")
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)
assert mod.detect_target_platform("bee-v1.8.0-aarch64-unknown-linux-gnu.tar.gz") == "Linux (aarch64)", mod.detect_target_platform("bee-v1.8.0-aarch64-unknown-linux-gnu.tar.gz")
assert mod.detect_target_platform("bee-v1.8.0-x86_64-unknown-linux-gnu.tar.gz") == "Linux (x86_64)"
print("ok")
"#,
            script = script.display()
        ))
        .output()
        .expect("python detect_target_platform");
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "detect_target_platform: {stderr}{stdout}"
    );
}

#[test]
fn install_sh_maps_unix_platforms_to_release_targets() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("install.sh");
    let cases = [
        ("Darwin", "arm64", "aarch64-apple-darwin"),
        ("Darwin", "x86_64", "x86_64-apple-darwin"),
        ("Linux", "x86_64", "x86_64-unknown-linux-gnu"),
        ("Linux", "aarch64", "aarch64-unknown-linux-gnu"),
    ];
    for (os, arch, expected) in cases {
        let output = Command::new("sh")
            .arg(&script)
            .arg("--print-platform")
            .env("BEEJS_UNAME_S", os)
            .env("BEEJS_UNAME_M", arch)
            .output()
            .expect("run install.sh --print-platform");
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            output.status.success(),
            "install.sh --print-platform {os}/{arch} failed: {stderr}"
        );
        assert_eq!(stdout, expected, "platform mapping for {os}/{arch}");
        assert!(
            !stderr.contains("prebuilt Linux arm64 archive is not available yet"),
            "Linux aarch64 must be supported"
        );
    }

    let ps1 = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("install.ps1");
    let ps1_text = fs::read_to_string(&ps1).expect("install.ps1");
    assert!(ps1_text.contains("x86_64-pc-windows-msvc.zip"));
    assert!(ps1_text.contains("bee.exe"));
}
