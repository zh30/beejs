//! Enterprise Compliance Manager
//! Provides compliance checking against various frameworks and regulations

use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Compliance framework types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceFramework {
    GDPR,          // General Data Protection Regulation
    HIPAA,         // Health Insurance Portability and Accountability Act
    SOC2,          // System and Organization Controls 2
    ISO27001,      // Information Security Management
    PCI_DSS,       // Payment Card Industry Data Security Standard
    CCPA,          // California Consumer Privacy Act
    SOX,           // Sarbanes-Oxley Act
    FedRAMP,       // Federal Risk and Authorization Management Program
    Custom(String), // Custom compliance framework
}

/// Compliance requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequirement {
    pub id: String,
    pub framework: ComplianceFramework,
    pub title: String,
    pub description: String,
    pub control_type: ControlType,
    pub severity: ComplianceSeverity,
    pub check_function: String, // Reference to check implementation
}

/// Control type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlType {
    Preventive,  // Prevents compliance violations
    Detective,   // Detects compliance violations
    Corrective,  // Corrects compliance violations
    Directive,   // Provides guidance
}

/// Compliance severity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceSeverity {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

/// Compliance frameworks manager
#[derive(Debug)]
pub struct ComplianceFrameworks {
    frameworks: Arc<RwLock<HashMap<String, ComplianceFrameworkConfig>>,
}

/// Framework configuration
#[derive(Debug, Clone)]
struct ComplianceFrameworkConfig {
    framework: ComplianceFramework,
    requirements: Vec<ComplianceRequirement>,
    version: String,
    effective_date: std::time::SystemTime,
}

/// Policy engine for compliance rules
#[derive(Debug)]
pub struct PolicyEngine {
    policies: Arc<RwLock<HashMap<String, CompliancePolicy>>,
}

/// Compliance policy
#[derive(Debug, Clone)]
pub struct CompliancePolicy {
    pub name: String,
    pub framework: ComplianceFramework,
    pub rules: Vec<ComplianceRule>,
    pub enforcement_level: EnforcementLevel,
}

/// Compliance rule
#[derive(Debug, Clone)]
pub struct ComplianceRule {
    pub id: String,
    pub name: String,
    pub check: String, // Check implementation
    pub remediation: String,
    pub documentation_url: Option<String>,
}

/// Enforcement level
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnforcementLevel {
    Strict,    // Block on violation
    Warning,   // Warn on violation
    Advisory,  // Suggest on violation
    Disabled,  // No enforcement
}

/// Compliance report
#[derive(Debug)]
pub struct ComplianceReport {
    pub framework: ComplianceFramework,
    pub timestamp: std::time::SystemTime,
    pub status: ComplianceStatus,
    pub score: f64, // 0.0 to 1.0
    pub violations: Vec<ComplianceViolation>,
    pub recommendations: Vec<String>,
    pub summary: String,
}

/// Compliance status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComplianceStatus {
    Compliant,
    NonCompliant,
    PartiallyCompliant,
    Unknown,
}

/// Compliance violation
#[derive(Debug, Clone)]
pub struct ComplianceViolation {
    pub requirement_id: String,
    pub requirement_title: String,
    pub severity: ComplianceSeverity,
    pub description: String,
    pub location: String,
    pub remediation: String,
}

/// Compliance manager
#[derive(Debug)]
pub struct ComplianceManager {
    frameworks: Arc<ComplianceFrameworks>,
    policies: Arc<PolicyEngine>,
    checker: Arc<ComplianceChecker>,
}

/// Compliance checker
#[derive(Debug)]
pub struct ComplianceChecker {
    framework_checks: HashMap<String, Box<dyn FrameworkCheck>>,
}

/// Framework check trait
pub trait FrameworkCheck: Send + Sync {
    fn framework(&self) -> &ComplianceFramework;
    fn check(&self, script: &str) -> Vec<ComplianceViolation>;
}

impl ComplianceFrameworks {
    /// Create a new compliance frameworks manager
    pub fn new() -> Self {
        let frameworks: _ = Arc::new(std::sync::Mutex::new(RwLock::new(HashMap::new())));
        Self { frameworks }
    }

    /// Register a compliance framework
    pub async fn register_framework(&self, config: ComplianceFrameworkConfig) -> Result<()> {
        let mut frameworks = self.frameworks.write().await;
        let framework_name: _ = match &config.framework {
            ComplianceFramework::GDPR => "GDPR".to_string(),
            ComplianceFramework::HIPAA => "HIPAA".to_string(),
            ComplianceFramework::SOC2 => "SOC2".to_string(),
            ComplianceFramework::ISO27001 => "ISO27001".to_string(),
            ComplianceFramework::PCI_DSS => "PCI_DSS".to_string(),
            ComplianceFramework::CCPA => "CCPA".to_string(),
            ComplianceFramework::SOX => "SOX".to_string(),
            ComplianceFramework::FedRAMP => "FedRAMP".to_string(),
            ComplianceFramework::Custom(name) => name.clone(),
        };

        frameworks.insert(framework_name, config);
        Ok(())
    }

    /// Get framework configuration
    pub async fn get_framework(&self, name: &str) -> Result<ComplianceFrameworkConfig> {
        let frameworks: _ = self.frameworks.read().await;
        frameworks.get(name)
            .cloned()
            .ok_or_else(|| anyhow!("Framework '{}' not found", name))
    }

    /// List all registered frameworks
    pub async fn list_frameworks(&self) -> Result<Vec<String>> {
        let frameworks: _ = self.frameworks.read().await;
        Ok(frameworks.keys().cloned().collect())
    }
}

impl Default for ComplianceFrameworks {
    fn default() -> Self {
        Self::new()
    }
}

impl PolicyEngine {
    /// Create a new policy engine
    pub fn new() -> Self {
        PolicyEngine {
            policies: Arc::new(std::sync::Mutex::new(RwLock::new(HashMap::new()))),
        }
    }

    /// Add a compliance policy
    pub async fn add_policy(&self, policy: CompliancePolicy) -> Result<()> {
        let mut policies = self.policies.write().await;
        policies.insert(policy.name.clone(), policy);
        Ok(())
    }

    /// Get policy by name
    pub async fn get_policy(&self, name: &str) -> Result<CompliancePolicy> {
        let policies: _ = self.policies.read().await;
        policies.get(name)
            .cloned()
            .ok_or_else(|| anyhow!("Policy '{}' not found", name))
    }

    /// List all policies
    pub async fn list_policies(&self) -> Result<Vec<String>> {
        let policies: _ = self.policies.read().await;
        Ok(policies.keys().cloned().collect())
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ComplianceChecker {
    /// Create a new compliance checker
    pub fn new() -> Self {
        let mut checker = ComplianceChecker {
            framework_checks: HashMap::new(),
        };

        // Register default framework checks
        checker.register_default_checks();

        checker
    }

    /// Register default framework checks
    fn register_default_checks(&mut self) {
        self.framework_checks.insert(
            "GDPR".to_string(),
            Box::new(GDPRCheck) as Box<dyn FrameworkCheck>,
        );

        self.framework_checks.insert(
            "HIPAA".to_string(),
            Box::new(HIPAACheck) as Box<dyn FrameworkCheck>,
        );

        self.framework_checks.insert(
            "SOC2".to_string(),
            Box::new(SOC2Check) as Box<dyn FrameworkCheck>,
        );
    }

    /// Check compliance against a framework
    pub fn check(&self, framework: &ComplianceFramework, script: &str) -> Vec<ComplianceViolation> {
        let framework_name: _ = match framework {
            ComplianceFramework::GDPR => "GDPR",
            ComplianceFramework::HIPAA => "HIPAA",
            ComplianceFramework::SOC2 => "SOC2",
            ComplianceFramework::ISO27001 => "ISO27001",
            ComplianceFramework::PCI_DSS => "PCI_DSS",
            ComplianceFramework::CCPA => "CCPA",
            ComplianceFramework::SOX => "SOX",
            ComplianceFramework::FedRAMP => "FedRAMP",
            ComplianceFramework::Custom(name) => name.as_str(),
        };

        if let Some(check) = self.framework_checks.get(framework_name) {
            check.check(script)
        } else {
            Vec::new()
        }
    }
}

impl Default for ComplianceChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// GDPR compliance check
#[derive(Debug)]
pub struct GDPRCheck;

impl FrameworkCheck for GDPRCheck {
    fn framework(&self) -> &ComplianceFramework {
        &ComplianceFramework::GDPR
    }

    fn check(&self, script: &str) -> Vec<ComplianceViolation> {
        let mut violations = Vec::new();

        // Check for personal data processing
        if script.contains("localStorage") || script.contains("sessionStorage") {
            violations.push(ComplianceViolation {
                requirement_id: "GDPR-001".to_string(),
                requirement_title: "Data Storage Consent".to_string(),
                severity: ComplianceSeverity::High,
                description: "Code may store data in browser storage without explicit consent".to_string(),
                location: "storage_access".to_string(),
                remediation: "Implement explicit user consent for data storage".to_string(),
            });
        }

        // Check for data transmission
        if script.contains("fetch(") || script.contains("XMLHttpRequest") {
            violations.push(ComplianceViolation {
                requirement_id: "GDPR-002".to_string(),
                requirement_title: "Data Transfer".to_string(),
                severity: ComplianceSeverity::Medium,
                description: "Code may transmit data to external servers".to_string(),
                location: "network_request".to_string(),
                remediation: "Ensure data transfer is compliant with GDPR".to_string(),
            });
        }

        violations
    }
}

/// HIPAA compliance check
#[derive(Debug)]
pub struct HIPAACheck;

impl FrameworkCheck for HIPAACheck {
    fn framework(&self) -> &ComplianceFramework {
        &ComplianceFramework::HIPAA
    }

    fn check(&self, script: &str) -> Vec<ComplianceViolation> {
        let mut violations = Vec::new();

        // Check for encryption
        if !script.contains("encrypt") && !script.contains("crypto") {
            violations.push(ComplianceViolation {
                requirement_id: "HIPAA-001".to_string(),
                requirement_title: "Data Encryption".to_string(),
                severity: ComplianceSeverity::Critical,
                description: "No encryption detected for PHI (Protected Health Information)".to_string(),
                location: "encryption_check".to_string(),
                remediation: "Implement encryption for all PHI".to_string(),
            });
        }

        violations
    }
}

/// SOC2 compliance check
#[derive(Debug)]
pub struct SOC2Check;

impl FrameworkCheck for SOC2Check {
    fn framework(&self) -> &ComplianceFramework {
        &ComplianceFramework::SOC2
    }

    fn check(&self, script: &str) -> Vec<ComplianceViolation> {
        let mut violations = Vec::new();

        // Check for logging
        if !script.contains("console.log") && !script.contains("log") {
            violations.push(ComplianceViolation {
                requirement_id: "SOC2-001".to_string(),
                requirement_title: "Audit Logging".to_string(),
                severity: ComplianceSeverity::Medium,
                description: "No logging detected for audit trail".to_string(),
                location: "logging_check".to_string(),
                remediation: "Implement comprehensive logging".to_string(),
            });
        }

        violations
    }
}

impl ComplianceManager {
    /// Create a new compliance manager
    pub fn new() -> Self {
        ComplianceManager {
            frameworks: Arc::new(std::sync::Mutex::new(ComplianceFrameworks::new())),
            policies: Arc::new(std::sync::Mutex::new(PolicyEngine::new())),
            checker: Arc::new(std::sync::Mutex::new(ComplianceChecker::new())),
        }
    }

    /// Check compliance for a script
    pub async fn check_compliance(&self, script: &str, framework: ComplianceFramework) -> Result<ComplianceReport> {
        let violations: _ = self.checker.check(&framework, script);

        let score: _ = if violations.is_empty() {
            1.0
        } else {
            let critical_count = violations.iter().filter(|v| v.severity == ComplianceSeverity::Critical).count();
            let high_count: _ = violations.iter().filter(|v| v.severity == ComplianceSeverity::High).count();
            let medium_count: _ = violations.iter().filter(|v| v.severity == ComplianceSeverity::Medium).count();

            let penalty: _ = (critical_count as f64 * 0.3) + (high_count as f64 * 0.2) + (medium_count as f64 * 0.1);
            (1.0 - penalty).max(0.0)
        };

        let status: _ = if score >= 0.95 {
            ComplianceStatus::Compliant
        } else if score >= 0.7 {
            ComplianceStatus::PartiallyCompliant
        } else {
            ComplianceStatus::NonCompliant
        };

        let recommendations: _ = violations.iter()
            .map(|v| format!("{}: {}", v.requirement_title, v.remediation))
            .collect();

        let framework_name: _ = match &framework {
            ComplianceFramework::GDPR => "GDPR",
            ComplianceFramework::HIPAA => "HIPAA",
            ComplianceFramework::SOC2 => "SOC2",
            ComplianceFramework::ISO27001 => "ISO27001",
            ComplianceFramework::PCI_DSS => "PCI DSS",
            ComplianceFramework::CCPA => "CCPA",
            ComplianceFramework::SOX => "SOX",
            ComplianceFramework::FedRAMP => "FedRAMP",
            ComplianceFramework::Custom(name) => name.as_str(),
        };

        Ok(ComplianceReport {
            framework,
            timestamp: std::time::SystemTime::now(),
            status,
            score,
            violations,
            recommendations,
            summary: format!("{} compliance check completed with score: {:.2}%", framework_name, score * 100.0),
        })
    }

    /// Register a compliance framework
    pub async fn register_framework(&self, config: ComplianceFrameworkConfig) -> Result<()> {
        self.frameworks.register_framework(config).await
    }

    /// Add a compliance policy
    pub async fn add_policy(&self, policy: CompliancePolicy) -> Result<()> {
        self.policies.add_policy(policy).await
    }
}

impl Default for ComplianceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    #[tokio::test]
    async fn test_compliance_frameworks() {
        let manager: _ = ComplianceFrameworks::new();

        let config: _ = ComplianceFrameworkConfig {
            framework: ComplianceFramework::GDPR,
            requirements: Vec::new(),
            version: "1.0".to_string(),
            effective_date: std::time::SystemTime::now(),
        };

        manager.register_framework(config).await.unwrap();

        let frameworks: _ = manager.list_frameworks().await.unwrap();
        assert!(frameworks.contains(&"GDPR".to_string()));
    }

    #[tokio::test]
    async fn test_gdpr_compliance_check() {
        let checker: _ = ComplianceChecker::new();
        let framework: _ = ComplianceFramework::GDPR;

        let script: _ = r#"
localStorage.setItem('user_data', 'sensitive');
fetch('/api/data');
"#;

        let violations: _ = checker.check(&framework, script);
        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| v.requirement_id == "GDPR-001"));
    }

    #[tokio::test]
    async fn test_hipaa_compliance_check() {
        let checker: _ = ComplianceChecker::new();
        let framework: _ = ComplianceFramework::HIPAA;

        let script: _ = "const data = 'patient_info';";
        let violations: _ = checker.check(&framework, script);
        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| v.requirement_id == "HIPAA-001"));
    }

    #[tokio::test]
    async fn test_soc2_compliance_check() {
        let checker: _ = ComplianceChecker::new();
        let framework: _ = ComplianceFramework::SOC2;

        let script: _ = "const x = 42;";
        let violations: _ = checker.check(&framework, script);
        assert!(!violations.is_empty());
        assert!(violations.iter().any(|v| v.requirement_id == "SOC2-001"));
    }

    #[tokio::test]
    async fn test_compliance_manager() {
        let manager: _ = ComplianceManager::new();

        let script: _ = "console.log('test');";
        let framework: _ = ComplianceFramework::GDPR;

        let report: _ = manager.check_compliance(script, framework).await.unwrap();
        assert!(report.score >= 0.0 && report.score <= 1.0);
        assert!(report.violations.len() >= 0);
    }

    #[tokio::test]
    async fn test_compliance_score_calculation() {
        let manager: _ = ComplianceManager::new();

        // Compliant script
        let compliant_script: _ = "";
        let report: _ = manager.check_compliance(compliant_script, ComplianceFramework::GDPR).await.unwrap();
        assert_eq!(report.status, ComplianceStatus::Compliant);

        // Non-compliant script
        let non_compliant_script: _ = "localStorage.setItem('data', 'value');";
        let report: _ = manager.check_compliance(non_compliant_script, ComplianceFramework::GDPR).await.unwrap();
        assert!(report.status != ComplianceStatus::Compliant);
    }
}
