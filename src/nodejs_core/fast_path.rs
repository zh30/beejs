//! Rust-typed hot paths for `path`, `Buffer`, `crypto.createHash`, and
//! `fs.readFileSync`.
//!
//! V8 Fast API (`v8::fast_api`) requires rusty_v8 >= 0.32. Until
//! `upgrade/rusty-v8-0.32` merges, these helpers stay in Rust so the JS
//! bindings do not grow extra allocations. Do not add FastCall stubs on 0.22.

use std::path::{Component, Path};

/// Normalize a filesystem path without going through JS string splitting.
pub fn normalize_path(input: &str) -> String {
    let path = Path::new(input);
    let mut parts = Vec::new();
    let absolute = path.is_absolute();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                parts.pop();
            }
            Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
            Component::RootDir => parts.clear(),
            Component::Prefix(prefix) => {
                parts.push(prefix.as_os_str().to_string_lossy().into_owned());
            }
        }
    }
    if absolute {
        format!("/{}", parts.join("/"))
    } else {
        parts.join("/")
    }
}

/// Digest bytes with the same algorithm names as `crypto.createHash`.
pub fn hash_bytes(algorithm: &str, data: &[u8]) -> Option<Vec<u8>> {
    let normalized = algorithm.to_ascii_lowercase().replace('-', "");
    match normalized.as_str() {
        "sha256" => Some(
            ring::digest::digest(&ring::digest::SHA256, data)
                .as_ref()
                .to_vec(),
        ),
        "sha384" => Some(
            ring::digest::digest(&ring::digest::SHA384, data)
                .as_ref()
                .to_vec(),
        ),
        "sha512" => Some(
            ring::digest::digest(&ring::digest::SHA512, data)
                .as_ref()
                .to_vec(),
        ),
        "sha1" => {
            use sha1::{Digest, Sha1};
            let mut hasher = Sha1::new();
            hasher.update(data);
            Some(hasher.finalize().to_vec())
        }
        "blake3" => Some(blake3::hash(data).as_bytes().to_vec()),
        _ => None,
    }
}

/// Sync file read used by `fs.readFileSync` after permission checks.
pub fn read_file_sync_bytes(path: &str) -> std::io::Result<Vec<u8>> {
    std::fs::read(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_path_collapses_dots() {
        assert_eq!(normalize_path("/tmp/./a/../b"), "/tmp/b");
    }

    #[test]
    fn hash_bytes_sha256_is_stable() {
        let digest = hash_bytes("sha256", b"abc").expect("sha256");
        assert_eq!(digest.len(), 32);
        assert_eq!(digest, hash_bytes("SHA-256", b"abc").unwrap());
    }
}
