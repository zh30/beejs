//! Enterprise Security Manager
//! Provides security policy enforcement and audit logging

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};

/// Security policy type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityPolicy {
    /// Allow all operations
    Permissive,
    /// Block operations based on rules
    Restrictive,
    /// Custom policy with specific rules
    Custom(HashMap<String, SecurityRule>),
}

/// Security rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRule {
    pub name: String,
    pub description: String,
    pub allowed: bool,
    pub conditions: Vec<String>,
}

/// Security manager
#[derive(Debug)]
pub struct SecurityManager {
    policies: HashMap<String, SecurityPolicy>,
    audit_log: Vec<SecurityEvent>,
}

/// Security audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub timestamp: std::time::SystemTime,
    pub event_type: String,
    pub source: String,
    pub action: String,
    pub result: SecurityResult,
    pub details: HashMap<String, String>,
}

/// Security result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityResult {
    Allowed,
    Denied(String),
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new() -> Self {
        let mut manager = Self {
            policies: HashMap::new(),
            audit_log: Vec::new(),
        };

        // Add default permissive policy
        manager.policies.insert(
            "default".to_string(),
            SecurityPolicy::Permissive,
        );

        manager
    }

    /// Add a security policy
    pub fn add_policy(&mut self, name: String, policy: SecurityPolicy) {
        self.policies.insert(name, policy);
    }

    /// Check if an operation is allowed
    pub fn check_permission(
        &self,
        policy_name: &str,
        operation: &str,
        context: &HashMap<String, String>,
    ) -> Result<SecurityResult> {
        let policy = self.policies.get(policy_name)
            .ok_or_else(|| anyhow!("Policy not found: {}", policy_name))?;

        let result = match policy {
            SecurityPolicy::Permissive => SecurityResult::Allowed,
            SecurityPolicy::Restrictive => {
                // Default to deny for restrictive policies
                SecurityResult::Denied("Restrictive policy blocks operation".to_string())
            }
            SecurityPolicy::Custom(rules) => {
                if let Some(rule) = rules.get(operation) {
                    if rule.allowed {
                        SecurityResult::Allowed
                    } else {
                        SecurityResult::Denied(format!("Operation '{}' denied by rule: {}", operation, rule.name))
                    }
                } else {
                    SecurityResult::Denied(format!("No rule found for operation: {}", operation))
                }
            }
        };

        // Log the security event
        self.log_event(
            "permission_check".to_string(),
            "security_manager".to_string(),
            operation.to_string(),
            result.clone(),
        );

        Ok(result)
    }

    /// Log a security event
    fn log_event(&self, event_type: String, source: String, action: String, result: SecurityResult) {
        let event = SecurityEvent {
            timestamp: std::time::SystemTime::now(),
            event_type,
            source,
            action,
            result,
            details: HashMap::new(),
        };

        // Note: In real implementation, this would append to audit_log
        // For now, we just print it
        println!("Security Event: {:?}", event);
    }

    /// Get audit log
    pub fn get_audit_log(&self) -> &[SecurityEvent] {
        &self.audit_log
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_manager_creation() {
        let manager = SecurityManager::new();
        assert!(!manager.policies.is_empty());
    }

    #[test]
    fn test_permissive_policy() {
        let manager = SecurityManager::new();
        let result = manager.check_permission("default", "execute_script", &HashMap::new());
        assert!(matches!(result, Ok(SecurityResult::Allowed)));
    }
}
