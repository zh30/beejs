use beejs::package_manager::{PackageJson, PackageManager, PackageManagerConfig};
use flate2::write::GzEncoder;
use flate2::Compression;
use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::path::Path;
use tar::{Builder, EntryType, Header};
use tempfile::TempDir;

fn package_manager(temp_dir: &TempDir) -> PackageManager {
    PackageManager::new(PackageManagerConfig {
        cache_dir: temp_dir.path().join("cache"),
        node_modules_dir: temp_dir.path().join("node_modules"),
        ..Default::default()
    })
    .unwrap()
}

fn finish_archive(builder: Builder<GzEncoder<fs::File>>) {
    let encoder = builder.into_inner().unwrap();
    encoder.finish().unwrap();
}

fn append_file(builder: &mut Builder<GzEncoder<fs::File>>, path: &str, content: &[u8]) {
    let mut header = Header::new_gnu();
    header.set_entry_type(EntryType::Regular);
    header.set_size(content.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder
        .append_data(&mut header, path, Cursor::new(content))
        .unwrap();
}

fn append_raw_path_file(builder: &mut Builder<GzEncoder<fs::File>>, path: &str, content: &[u8]) {
    let mut header = Header::new_gnu();
    header.set_entry_type(EntryType::Regular);
    header.set_size(content.len() as u64);
    header.set_mode(0o644);
    let header_bytes = header.as_mut_bytes();
    header_bytes[..path.len()].copy_from_slice(path.as_bytes());
    header.set_cksum();
    builder.append(&header, Cursor::new(content)).unwrap();
}

fn append_link(
    builder: &mut Builder<GzEncoder<fs::File>>,
    entry_type: EntryType,
    path: &str,
    target: &Path,
) {
    let mut header = Header::new_gnu();
    header.set_entry_type(entry_type);
    header.set_size(0);
    header.set_mode(0o777);
    header.set_cksum();
    builder.append_link(&mut header, path, target).unwrap();
}

fn append_fifo(builder: &mut Builder<GzEncoder<fs::File>>, path: &str) {
    let mut header = Header::new_gnu();
    header.set_entry_type(EntryType::Fifo);
    header.set_size(0);
    header.set_mode(0o644);
    header.set_cksum();
    builder
        .append_data(&mut header, path, std::io::empty())
        .unwrap();
}

fn create_tgz<F>(path: &Path, write_entries: F)
where
    F: FnOnce(&mut Builder<GzEncoder<fs::File>>),
{
    let file = fs::File::create(path).unwrap();
    let encoder = GzEncoder::new(file, Compression::default());
    let mut builder = Builder::new(encoder);
    write_entries(&mut builder);
    builder.finish().unwrap();
    finish_archive(builder);
}

fn file_url(path: &Path) -> String {
    format!("file://{}", path.display())
}

#[test]
fn parses_npm_camel_case_dependency_sections() {
    let temp_dir = TempDir::new().unwrap();
    let pm = package_manager(&temp_dir);
    let package_json_path = temp_dir.path().join("package.json");

    fs::write(
        &package_json_path,
        r#"{
            "name": "camel-case-package",
            "version": "1.0.0",
            "devDependencies": {
                "typescript": "^5.0.0"
            },
            "peerDependencies": {
                "react": "^18.0.0"
            },
            "optionalDependencies": {
                "fsevents": "^2.3.0"
            }
        }"#,
    )
    .unwrap();

    let package = pm.parse_package_json(&package_json_path).unwrap();

    assert_eq!(
        package.dev_dependencies.unwrap().get("typescript"),
        Some(&"^5.0.0".to_string())
    );
    assert_eq!(
        package.peer_dependencies.unwrap().get("react"),
        Some(&"^18.0.0".to_string())
    );
    assert_eq!(
        package.optional_dependencies.unwrap().get("fsevents"),
        Some(&"^2.3.0".to_string())
    );
}

#[test]
fn extract_package_rejects_parent_directory_entries_without_writing_outside_package() {
    let temp_dir = TempDir::new().unwrap();
    let pm = package_manager(&temp_dir);
    let archive_path = temp_dir.path().join("parent-escape.tgz");
    let outside_path = temp_dir.path().join("outside.txt");

    create_tgz(&archive_path, |builder| {
        append_raw_path_file(builder, "package/../../outside.txt", b"escaped");
    });

    let result = pm.extract_package(&archive_path, "evil");

    assert!(result.is_err(), "parent-directory entry must be rejected");
    assert!(
        !outside_path.exists(),
        "malicious archive wrote outside package dir at {}",
        outside_path.display()
    );
}

#[test]
fn extract_package_cleans_partial_package_when_late_entry_is_unsafe() {
    let temp_dir = TempDir::new().unwrap();
    let pm = package_manager(&temp_dir);
    let archive_path = temp_dir.path().join("late-escape.tgz");
    let package_dir = temp_dir.path().join("node_modules").join("evil");
    let outside_path = temp_dir.path().join("outside.txt");

    create_tgz(&archive_path, |builder| {
        append_file(builder, "package/index.js", b"module.exports = 1;\n");
        append_raw_path_file(builder, "package/../../outside.txt", b"escaped");
    });

    let result = pm.extract_package(&archive_path, "evil");

    assert!(result.is_err(), "late unsafe entry must reject archive");
    assert!(
        !outside_path.exists(),
        "malicious archive wrote outside package dir at {}",
        outside_path.display()
    );
    assert!(
        !package_dir.exists(),
        "failed extraction must not leave a partial package at {}",
        package_dir.display()
    );
}

#[test]
fn extract_package_rejects_absolute_path_entries() {
    let temp_dir = TempDir::new().unwrap();
    let pm = package_manager(&temp_dir);
    let archive_path = temp_dir.path().join("absolute.tgz");

    create_tgz(&archive_path, |builder| {
        append_raw_path_file(builder, "/package/absolute.txt", b"absolute");
    });

    let result = pm.extract_package(&archive_path, "evil");

    assert!(result.is_err(), "absolute tar entry must be rejected");
}

#[test]
fn extract_package_rejects_symlink_escape_entries() {
    let temp_dir = TempDir::new().unwrap();
    let pm = package_manager(&temp_dir);
    let archive_path = temp_dir.path().join("symlink-escape.tgz");
    let link_path = temp_dir
        .path()
        .join("node_modules")
        .join("evil")
        .join("link");

    create_tgz(&archive_path, |builder| {
        append_link(
            builder,
            EntryType::Symlink,
            "package/link",
            Path::new("../../outside.txt"),
        );
    });

    let result = pm.extract_package(&archive_path, "evil");

    assert!(result.is_err(), "escaping symlink must be rejected");
    assert!(
        fs::symlink_metadata(&link_path).is_err(),
        "escaping symlink should not be materialized"
    );
}

#[test]
fn extract_package_rejects_hardlink_escape_entries() {
    let temp_dir = TempDir::new().unwrap();
    let pm = package_manager(&temp_dir);
    let archive_path = temp_dir.path().join("hardlink-escape.tgz");
    let outside_source = temp_dir.path().join("outside-source.txt");
    let hardlink_path = temp_dir
        .path()
        .join("node_modules")
        .join("evil")
        .join("hardlink");
    fs::write(&outside_source, "outside").unwrap();

    create_tgz(&archive_path, |builder| {
        append_link(
            builder,
            EntryType::Link,
            "package/hardlink",
            outside_source.as_path(),
        );
    });

    let result = pm.extract_package(&archive_path, "evil");

    assert!(result.is_err(), "escaping hardlink must be rejected");
    assert!(
        !hardlink_path.exists(),
        "escaping hardlink should not be materialized"
    );
}

#[test]
fn extract_package_rejects_special_file_entries() {
    let temp_dir = TempDir::new().unwrap();
    let pm = package_manager(&temp_dir);
    let archive_path = temp_dir.path().join("special-file.tgz");
    let fifo_path = temp_dir
        .path()
        .join("node_modules")
        .join("evil")
        .join("fifo");

    create_tgz(&archive_path, |builder| {
        append_fifo(builder, "package/fifo");
    });

    let result = pm.extract_package(&archive_path, "evil");

    assert!(result.is_err(), "special file entry must be rejected");
    assert!(
        !fifo_path.exists(),
        "special file should not be materialized"
    );
}

#[test]
fn download_package_rejects_integrity_mismatch_and_does_not_keep_cache_file() {
    let temp_dir = TempDir::new().unwrap();
    let registry_dir = temp_dir.path().join("registry");
    fs::create_dir_all(&registry_dir).unwrap();
    let archive_path = temp_dir.path().join("pkg.tgz");
    create_tgz(&archive_path, |builder| {
        append_file(builder, "package/index.js", b"module.exports = 1;\n");
    });

    fs::write(
        registry_dir.join("pkg"),
        format!(
            r#"{{
                "versions": {{
                    "1.0.0": {{
                        "dist": {{
                            "tarball": "{}",
                            "integrity": "sha512-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="
                        }}
                    }}
                }}
            }}"#,
            file_url(&archive_path)
        ),
    )
    .unwrap();

    let pm = PackageManager::new(PackageManagerConfig {
        registry_url: file_url(&registry_dir),
        cache_dir: temp_dir.path().join("cache"),
        node_modules_dir: temp_dir.path().join("node_modules"),
        ..Default::default()
    })
    .unwrap();

    let result = pm.download_package("pkg", "1.0.0");

    assert!(result.is_err(), "integrity mismatch must reject download");
    assert!(
        !temp_dir
            .path()
            .join("cache")
            .join("pkg")
            .join("1.0.0.tgz")
            .exists(),
        "mismatched tarball must not remain in cache"
    );
}

#[test]
fn download_package_rejects_registry_entry_without_integrity_or_shasum() {
    let temp_dir = TempDir::new().unwrap();
    let registry_dir = temp_dir.path().join("registry");
    fs::create_dir_all(&registry_dir).unwrap();
    let archive_path = temp_dir.path().join("pkg.tgz");
    create_tgz(&archive_path, |builder| {
        append_file(builder, "package/index.js", b"module.exports = 1;\n");
    });

    fs::write(
        registry_dir.join("pkg"),
        format!(
            r#"{{
                "versions": {{
                    "1.0.0": {{
                        "dist": {{
                            "tarball": "{}"
                        }}
                    }}
                }}
            }}"#,
            file_url(&archive_path)
        ),
    )
    .unwrap();

    let pm = PackageManager::new(PackageManagerConfig {
        registry_url: file_url(&registry_dir),
        cache_dir: temp_dir.path().join("cache"),
        node_modules_dir: temp_dir.path().join("node_modules"),
        ..Default::default()
    })
    .unwrap();

    let result = pm.download_package("pkg", "1.0.0");

    assert!(
        result.is_err(),
        "download without integrity or shasum must not be treated as trusted"
    );
}

#[test]
fn generate_lock_for_package_refuses_missing_integrity_material() {
    let temp_dir = TempDir::new().unwrap();
    let pm = package_manager(&temp_dir);

    let result = pm.generate_lock_for_package("pkg", "1.0.0");

    assert!(
        result.is_err(),
        "lock generation must not mark a package trusted without integrity"
    );
}

#[test]
fn install_dependencies_propagates_required_dependency_failure() {
    let temp_dir = TempDir::new().unwrap();
    let pm = PackageManager::new(PackageManagerConfig {
        registry_url: file_url(&temp_dir.path().join("empty-registry")),
        cache_dir: temp_dir.path().join("cache"),
        node_modules_dir: temp_dir.path().join("node_modules"),
        ..Default::default()
    })
    .unwrap();
    let mut dependencies = HashMap::new();
    dependencies.insert("missing".to_string(), "1.0.0".to_string());
    let package_json = PackageJson {
        name: "root".to_string(),
        version: "1.0.0".to_string(),
        description: None,
        main: None,
        scripts: None,
        dependencies: Some(dependencies),
        dev_dependencies: None,
        peer_dependencies: None,
        optional_dependencies: None,
        author: None,
        license: None,
        repository: None,
    };

    let result = pm.install_dependencies(&package_json);

    assert!(
        result.is_err(),
        "required dependency installation failures must propagate"
    );
}
