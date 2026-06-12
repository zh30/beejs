use std::fmt;
use std::fs;
use std::path::Component;
use std::path::{Path, PathBuf};

const JS_EXTENSIONS: &[&str] = &["js", "json"];
const BUILTIN_MODULES: &[&str] = &[
    "buffer",
    "child_process",
    "crypto",
    "dns",
    "events",
    "fs",
    "http",
    "net",
    "os",
    "path",
    "performance",
    "process",
    "querystring",
    "readline",
    "stream",
    "tcp_async",
    "timers",
    "url",
    "util",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResolvedModule {
    Builtin(String),
    File(PathBuf),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PackageSpecifier {
    name: String,
    subpath: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommonJsResolveError {
    specifier: String,
    from: PathBuf,
    reason: Option<String>,
}

impl CommonJsResolveError {
    pub fn new(specifier: impl Into<String>, from: impl Into<PathBuf>) -> Self {
        Self {
            specifier: specifier.into(),
            from: from.into(),
            reason: None,
        }
    }

    fn with_reason(
        specifier: impl Into<String>,
        from: impl Into<PathBuf>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            specifier: specifier.into(),
            from: from.into(),
            reason: Some(reason.into()),
        }
    }
}

impl fmt::Display for CommonJsResolveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(reason) = &self.reason {
            return write!(formatter, "{reason}");
        }

        write!(
            formatter,
            "Cannot find module '{}' from '{}'",
            self.specifier,
            self.from.display()
        )
    }
}

impl std::error::Error for CommonJsResolveError {}

pub fn is_builtin_module(specifier: &str) -> bool {
    BUILTIN_MODULES.contains(&specifier)
}

pub fn resolve_commonjs_module(
    specifier: &str,
    parent_dir: &Path,
) -> Result<ResolvedModule, CommonJsResolveError> {
    if is_builtin_module(specifier) {
        return Ok(ResolvedModule::Builtin(specifier.to_string()));
    }

    if specifier == "."
        || specifier == ".."
        || specifier.starts_with("./")
        || specifier.starts_with("../")
        || Path::new(specifier).is_absolute()
    {
        let candidate = if Path::new(specifier).is_absolute() {
            PathBuf::from(specifier)
        } else {
            parent_dir.join(specifier)
        };
        return resolve_path_candidate(&candidate)?
            .map(ResolvedModule::File)
            .ok_or_else(|| CommonJsResolveError::new(specifier, parent_dir));
    }

    resolve_node_modules(specifier, parent_dir)?
        .map(ResolvedModule::File)
        .ok_or_else(|| CommonJsResolveError::new(specifier, parent_dir))
}

fn resolve_node_modules(
    specifier: &str,
    parent_dir: &Path,
) -> Result<Option<PathBuf>, CommonJsResolveError> {
    let package = parse_package_specifier(specifier);
    let mut current = Some(parent_dir);
    while let Some(dir) = current {
        let package_root = dir.join("node_modules").join(&package.name);
        let resolved = if let Some(subpath) = &package.subpath {
            resolve_package_subpath(&package_root, subpath)?
        } else {
            resolve_path_candidate(&package_root)?
        };

        if let Some(resolved) = resolved {
            return Ok(Some(resolved));
        }
        current = dir.parent();
    }
    Ok(None)
}

fn parse_package_specifier(specifier: &str) -> PackageSpecifier {
    if specifier.starts_with('@') {
        let mut parts = specifier.splitn(3, '/');
        let scope = parts.next().unwrap_or_default();
        let name = parts.next().unwrap_or_default();
        let subpath = parts.next().map(|path| path.to_string());

        if !scope.is_empty() && !name.is_empty() {
            return PackageSpecifier {
                name: format!("{scope}/{name}"),
                subpath,
            };
        }
    }

    let mut parts = specifier.splitn(2, '/');
    let name = parts.next().unwrap_or_default().to_string();
    let subpath = parts.next().map(|path| path.to_string());
    PackageSpecifier { name, subpath }
}

fn resolve_path_candidate(candidate: &Path) -> Result<Option<PathBuf>, CommonJsResolveError> {
    if let Some(file) = resolve_as_file(candidate) {
        return Ok(Some(file));
    }

    resolve_as_directory(candidate)
}

fn resolve_as_file(candidate: &Path) -> Option<PathBuf> {
    if candidate.is_file() {
        return canonical_file(candidate);
    }

    if candidate.extension().is_none() {
        for extension in JS_EXTENSIONS {
            let with_extension = candidate.with_extension(extension);
            if with_extension.is_file() {
                return canonical_file(&with_extension);
            }
        }
    }

    None
}

fn resolve_as_directory(candidate: &Path) -> Result<Option<PathBuf>, CommonJsResolveError> {
    if !candidate.is_dir() {
        return Ok(None);
    }

    if let Some(main_path) = resolve_package_main(candidate)? {
        return Ok(Some(main_path));
    }

    for extension in JS_EXTENSIONS {
        let index_path = candidate.join(format!("index.{extension}"));
        if index_path.is_file() {
            return Ok(canonical_file(&index_path));
        }
    }

    Ok(None)
}

fn resolve_package_main(package_root: &Path) -> Result<Option<PathBuf>, CommonJsResolveError> {
    let Some(package_json) = read_package_json(package_root)? else {
        return Ok(None);
    };

    if let Some(exports) = package_json.get("exports") {
        if let Some(resolved) = resolve_package_root_exports(package_root, exports)? {
            return Ok(Some(resolved));
        }
    }

    let Some(main) = package_json.get("main").and_then(|value| value.as_str()) else {
        return Ok(None);
    };
    let main_candidate = package_root.join(main);
    resolve_path_candidate(&main_candidate)
}

fn resolve_package_subpath(
    package_root: &Path,
    subpath: &str,
) -> Result<Option<PathBuf>, CommonJsResolveError> {
    let Some(package_json) = read_package_json(package_root)? else {
        return resolve_path_candidate(&package_root.join(subpath));
    };

    let Some(exports) = package_json.get("exports") else {
        return resolve_path_candidate(&package_root.join(subpath));
    };
    let Some(exports_obj) = exports.as_object() else {
        return Ok(None);
    };

    let export_key = format!("./{}", subpath.trim_start_matches('/'));
    let Some(target) = exports_obj.get(&export_key) else {
        return Ok(None);
    };

    resolve_package_export_value(package_root, target)
}

fn read_package_json(
    package_root: &Path,
) -> Result<Option<serde_json::Value>, CommonJsResolveError> {
    let package_json_path = package_root.join("package.json");
    if !package_json_path.is_file() {
        return Ok(None);
    }

    crate::permissions::check_global_permission(
        crate::permissions::PermissionKind::FileSystem,
        crate::permissions::PermissionAction::Read,
        crate::permissions::ResourceId::Path(package_json_path.clone()),
    )
    .map_err(|error| {
        CommonJsResolveError::with_reason(
            package_json_path.to_string_lossy(),
            package_root,
            error.to_string(),
        )
    })?;

    let content = match fs::read_to_string(&package_json_path) {
        Ok(content) => content,
        Err(_) => return Ok(None),
    };
    let package_json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(package_json) => package_json,
        Err(_) => return Ok(None),
    };

    Ok(Some(package_json))
}

fn resolve_package_root_exports(
    package_root: &Path,
    exports: &serde_json::Value,
) -> Result<Option<PathBuf>, CommonJsResolveError> {
    if exports.is_string() {
        return resolve_package_export_value(package_root, exports);
    }

    let Some(exports_obj) = exports.as_object() else {
        return Ok(None);
    };
    let Some(root_target) = exports_obj.get(".") else {
        return Ok(None);
    };

    resolve_package_export_value(package_root, root_target)
}

fn resolve_package_export_value(
    package_root: &Path,
    target: &serde_json::Value,
) -> Result<Option<PathBuf>, CommonJsResolveError> {
    let Some(target) = target.as_str() else {
        return Ok(None);
    };

    resolve_package_string_target(package_root, target)
}

fn resolve_package_string_target(
    package_root: &Path,
    target: &str,
) -> Result<Option<PathBuf>, CommonJsResolveError> {
    if target.is_empty() {
        return Ok(None);
    }

    let target_path = Path::new(target);
    if target_path.is_absolute()
        || target_path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Ok(None);
    }

    let Some(resolved) = resolve_path_candidate(&package_root.join(target_path))? else {
        return Ok(None);
    };
    let canonical_root =
        fs::canonicalize(package_root).unwrap_or_else(|_| package_root.to_path_buf());
    if resolved.starts_with(&canonical_root) {
        Ok(Some(resolved))
    } else {
        Ok(None)
    }
}

fn canonical_file(path: &Path) -> Option<PathBuf> {
    fs::canonicalize(path)
        .ok()
        .or_else(|| Some(path.to_path_buf()))
}
