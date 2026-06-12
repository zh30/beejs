use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::RwLock;
use thiserror::Error;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum PermissionKind {
    FileSystem,
    Environment,
    Network,
    Process,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum PermissionAction {
    Read,
    Write,
    Execute,
    Connect,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ResourceId {
    Any,
    Path(PathBuf),
    Name(String),
    Url(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PermissionDecision {
    Allow,
    Deny,
}

impl PermissionDecision {
    pub fn is_allowed(self) -> bool {
        matches!(self, Self::Allow)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct PermissionRule {
    kind: PermissionKind,
    action: PermissionAction,
    resource: ResourceId,
}

#[derive(Debug, Error)]
pub enum PermissionError {
    #[error("permission denied: {kind:?} {action:?} {resource:?}")]
    Denied {
        kind: PermissionKind,
        action: PermissionAction,
        resource: ResourceId,
    },
}

#[derive(Clone, Debug, Default)]
pub struct ResourceBroker {
    allow_rules: HashSet<PermissionRule>,
    deny_rules: HashSet<PermissionRule>,
}

impl ResourceBroker {
    pub fn allow(&mut self, kind: PermissionKind, action: PermissionAction, resource: ResourceId) {
        let rule = PermissionRule {
            kind,
            action,
            resource: normalize_resource(resource),
        };
        self.deny_rules.remove(&rule);
        self.allow_rules.insert(rule);
    }

    pub fn deny(&mut self, kind: PermissionKind, action: PermissionAction, resource: ResourceId) {
        let rule = PermissionRule {
            kind,
            action,
            resource: normalize_resource(resource),
        };
        self.allow_rules.remove(&rule);
        self.deny_rules.insert(rule);
    }

    pub fn check(
        &self,
        kind: PermissionKind,
        action: PermissionAction,
        resource: ResourceId,
    ) -> PermissionDecision {
        let exact = PermissionRule {
            kind: kind.clone(),
            action: action.clone(),
            resource: normalize_resource(resource),
        };
        let wildcard = PermissionRule {
            kind,
            action,
            resource: ResourceId::Any,
        };

        if self.deny_rules.contains(&exact) {
            return PermissionDecision::Deny;
        }
        if self.allow_rules.contains(&exact) {
            return PermissionDecision::Allow;
        }
        if self.deny_rules.contains(&wildcard) {
            return PermissionDecision::Deny;
        }
        if self.allow_rules.contains(&wildcard) {
            return PermissionDecision::Allow;
        }

        PermissionDecision::Allow
    }

    pub fn check_result(
        &self,
        kind: PermissionKind,
        action: PermissionAction,
        resource: ResourceId,
    ) -> Result<(), PermissionError> {
        if self
            .check(kind.clone(), action.clone(), resource.clone())
            .is_allowed()
        {
            Ok(())
        } else {
            Err(PermissionError::Denied {
                kind,
                action,
                resource,
            })
        }
    }
}

fn normalize_resource(resource: ResourceId) -> ResourceId {
    match resource {
        ResourceId::Path(path) => ResourceId::Path(normalize_path(path)),
        other => other,
    }
}

fn normalize_path(path: PathBuf) -> PathBuf {
    if let Ok(canonical) = path.canonicalize() {
        return canonical;
    }

    let Some(parent) = path.parent() else {
        return path;
    };
    let Some(file_name) = path.file_name().map(|name| name.to_os_string()) else {
        return parent.canonicalize().unwrap_or(path);
    };

    match parent.canonicalize() {
        Ok(parent) => parent.join(file_name),
        Err(_) => path,
    }
}

pub static GLOBAL_RESOURCE_BROKER: Lazy<RwLock<ResourceBroker>> =
    Lazy::new(|| RwLock::new(ResourceBroker::default()));

pub fn global_resource_broker() -> &'static RwLock<ResourceBroker> {
    &GLOBAL_RESOURCE_BROKER
}

pub fn check_global_permission(
    kind: PermissionKind,
    action: PermissionAction,
    resource: ResourceId,
) -> Result<(), PermissionError> {
    GLOBAL_RESOURCE_BROKER
        .read()
        .expect("resource broker lock poisoned")
        .check_result(kind, action, resource)
}
