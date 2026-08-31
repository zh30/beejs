//! Disk + memory cache for TypeScript transpile output (content-hash keyed).

use crate::typescript::compiler::{CompilationOutput, TypeScriptError};
use crate::typescript::oxc_backend::BACKEND_ID;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::Mutex;

static MEMORY_CACHE: Lazy<Mutex<HashMap<u64, CompilationOutput>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn hash_source(source: &str, file_name: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    BACKEND_ID.hash(&mut hasher);
    source.hash(&mut hasher);
    file_name.hash(&mut hasher);
    hasher.finish()
}

fn cache_dir() -> PathBuf {
    std::env::temp_dir().join("beejs-ts-cache")
}

pub fn get_cached(source: &str, file_name: &str) -> Option<CompilationOutput> {
    let key = hash_source(source, file_name);
    if let Ok(cache) = MEMORY_CACHE.lock() {
        if let Some(hit) = cache.get(&key) {
            return Some(hit.clone());
        }
    }
    let path = cache_dir().join(format!("{:016x}.js", key));
    if let Ok(js) = fs::read_to_string(&path) {
        let output = CompilationOutput {
            js_code: js,
            source_map: None,
            diagnostics: Vec::<TypeScriptError>::new(),
        };
        if let Ok(mut cache) = MEMORY_CACHE.lock() {
            cache.insert(key, output.clone());
        }
        return Some(output);
    }
    None
}

pub fn put_cached(source: &str, file_name: &str, output: &CompilationOutput) {
    let key = hash_source(source, file_name);
    if let Ok(mut cache) = MEMORY_CACHE.lock() {
        cache.insert(key, output.clone());
    }
    let dir = cache_dir();
    let _ = fs::create_dir_all(&dir);
    let path = dir.join(format!("{:016x}.js", key));
    let _ = fs::write(path, &output.js_code);
}

/// Clear both memory and disk transpile caches.
pub fn clear_cache() {
    if let Ok(mut cache) = MEMORY_CACHE.lock() {
        cache.clear();
    }
    let dir = cache_dir();
    if dir.exists() {
        let _ = fs::remove_dir_all(dir);
    }
}
