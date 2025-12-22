//! Container security scanner
//! Scans container images for vulnerabilities and compliance issues
use std::collections::HashMap;
use std::path::Path;
use tracing::info;
/// Security scanner for container images
pub struct SecurityScanner {
    /// Vulnerability database
    vulnerability_db: VulnerabilityDatabase,
    /// Scan configuration
    config: ScanConfig,
}
impl SecurityScanner {
    /// Create a new security scanner
    pub fn new(config: ScanConfig) -> Self {
        Self {
            vulnerability_db: VulnerabilityDatabase::new(),
            config,
        }
    }
    /// Scan a container image
    pub async fn scan_image(&self, image: &ContainerImage) -> Result<ScanReport, Error> {
        info!("Scanning image: {}", image.name);
        // Scan for vulnerabilities
        let vulnerabilities: _ = self.scan_vulnerabilities(image).await?;
        // Scan for compliance issues
        let compliance_issues: _ = self.scan_compliance(image).await?;
        // Scan for secrets
        let secrets: _ = self.scan_secrets(image).await?;
        // Calculate overall risk score
        let risk_score: _ = self.calculate_risk_score(&vulnerabilities, &compliance_issues, &secrets);
        let report: _ = ScanReport {
            image_name: image.name.clone(),
            image_digest: image.digest.clone(),
            scan_timestamp: std::time::SystemTime::now(),
            vulnerabilities: vulnerabilities.clone(),
            compliance_issues: compliance_issues.clone(),
            secrets: secrets.clone(),
            risk_score,
            recommendations: self.generate_recommendations(&vulnerabilities, &compliance_issues),
        };
        info!("Completed scan for image: {}. Risk score: {}", image.name, risk_score);
        Ok(report)
    }
    /// Scan for vulnerabilities
    async fn scan_vulnerabilities(&self, image: &ContainerImage) -> Result<Vec<Vulnerability>, Error> {
        let mut vulnerabilities = Vec::new();
        // Scan each layer for vulnerabilities
        for layer in &image.layers {
            let layer_vulns: _ = self.scan_layer_vulnerabilities(layer).await?;
            vulnerabilities.extend(layer_vulns);
        }
        // Remove duplicates
        vulnerabilities.sort_by(|a, b| a.severity.cmp(&b.severity));
        vulnerabilities.dedup_by(|a, b| a.id == b.id);
        Ok(vulnerabilities)
    }
    /// Scan a single layer for vulnerabilities
    async fn scan_layer_vulnerabilities(&self, layer: &ImageLayer) -> Result<Vec<Vulnerability>, Error> {
        let mut vulnerabilities = Vec::new();
        // Check against vulnerability database
        for vuln in self.vulnerability_db.get_vulnerabilities() {
            // Check if vulnerability affects this layer
            if self.is_vulnerability_relevant(vuln, layer) {
                vulnerabilities.push(vuln.clone());
            }
        }
        Ok(vulnerabilities)
    }
    /// Check if vulnerability is relevant to layer
    fn is_vulnerability_relevant(&self, vuln: &Vulnerability, layer: &ImageLayer) -> bool {
        // Check if vulnerability affects the base image
        if layer.is_base_layer() {
            return vuln.affected_packages.iter().any(|pkg| {
                layer.packages.contains(pkg)
            });
        }
        // Check if vulnerability affects installed packages
        vuln.affected_packages.iter().any(|pkg| {
            layer.packages.contains(pkg)
        })
    }
    /// Scan for compliance issues
    async fn scan_compliance(&self, image: &ContainerImage) -> Result<Vec<ComplianceIssue>, Error> {
        let mut issues = Vec::new();
        // Check for non-root user
        if !self.has_non_root_user(image) {
            issues.push(ComplianceIssue {
                rule_id: "CIS-4.1".to_string(),
                severity: ComplianceSeverity::High,
                description: "Container is running as root user".to_string(),
                remediation: "Create a non-root user and switch to it".to_string(),
            });
        }
        // Check for read-only root filesystem
        if !self.has_read_only_root(image) {
            issues.push(ComplianceIssue {
                rule_id: "CIS-4.6".to_string(),
                severity: ComplianceSeverity::Medium,
                description: "Root filesystem is not read-only".to_string(),
                remediation: "Mount root filesystem as read-only".to_string(),
            });
        }
        // Check for capability dropping
        if !self.drops_capabilities(image) {
            issues.push(ComplianceIssue {
                rule_id: "CIS-4.4".to_string(),
                severity: ComplianceSeverity::Medium,
                description: "Linux capabilities are not dropped".to_string(),
                remediation: "Drop unnecessary Linux capabilities".to_string(),
            });
        }
        // Check for health check
        if !self.has_health_check(image) {
            issues.push(ComplianceIssue {
                rule_id: "CIS-4.8".to_string(),
                severity: ComplianceSeverity::Low,
                description: "No health check defined".to_string(),
                remediation: "Add a HEALTHCHECK instruction".to_string(),
            });
        }
        Ok(issues)
    }
    /// Scan for secrets in the image
    async fn scan_secrets(&self, image: &ContainerImage) -> Result<Vec<Secret>, Error> {
        let mut secrets = Vec::new();
        // Scan each layer for secrets
        for layer in &image.layers {
            let layer_secrets: _ = self.scan_layer_secrets(layer).await?;
            secrets.extend(layer_secrets);
        }
        Ok(secrets)
    }
    /// Scan a layer for secrets
    async fn scan_layer_secrets(&self, layer: &ImageLayer) -> Result<Vec<Secret>, Error> {
        let mut secrets = Vec::new();
        // Define secret patterns
        let secret_patterns: _ = vec![
            ("AWS_ACCESS_KEY_ID", r#"(?i)aws_access_key_id\s*[=:]\s*['\"]?[A-Z0-9]{16,}['\"]?"#),
            ("AWS_SECRET_ACCESS_KEY", r#"(?i)aws_secret_access_key\s*[=:]\s*['\"]?[A-Z0-9/+=]{40,}['\"]?"#),
            ("PRIVATE_KEY", r"-----BEGIN [A-Z ]*PRIVATE KEY-----"),
            ("API_KEY", r#"(?i)(api[_-]?key|apikey)\s*[=:]\s*['\"]?[a-zA-Z0-9]{32,}['\"]?"#),
            ("PASSWORD", r#"(?i)password\s*[=:]\s*['\"]?[a-zA-Z0-9]{8,}['\"]?"#),
        ];
        // Scan files for secrets
        for file_path in &layer.files {
            if let Ok(content) = self.read_file(file_path) {
                for (secret_type, pattern) in &secret_patterns {
                    if let Ok(regex) = regex::Regex::new(pattern) {
                        if regex.is_match(&content) {
                            secrets.push(Secret {
                                secret_type: secret_type.to_string(),
                                file_path: file_path.clone(),
                                line_number: self.find_line_number(&content, &regex),
                                remediation: format!("Remove or encrypt {}", secret_type),
                            });
                        }
                    }
                }
            }
        }
        Ok(secrets)
    }
    /// Read file content
    fn read_file(&self, path: &str) -> Result<String, std::io::Error> {
        // In a real implementation, this would read from the layer filesystem
        // For now, return an empty string
        Ok(String::new())
    }
    /// Find line number where pattern matches
    fn find_line_number(&self, content: &str, regex: &regex::Regex) -> Option<u32> {
        for (line_num, line) in content.lines().enumerate() {
            if regex.is_match(line) {
                return Some(line_num as u32 + 1);
            }
        }
        None
    }
    /// Calculate overall risk score
    fn calculate_risk_score(
        &self,
        vulnerabilities: &[Vulnerability],
        compliance_issues: &[ComplianceIssue],
        secrets: &[Secret],
    ) -> f64 {
        let mut score = 0.0;
        // Weight vulnerabilities
        for vuln in vulnerabilities {
            score += match vuln.severity {
                VulnerabilitySeverity::Critical => 10.0,
                VulnerabilitySeverity::High => 7.5,
                VulnerabilitySeverity::Medium => 5.0,
                VulnerabilitySeverity::Low => 2.5,
                VulnerabilitySeverity::Info => 1.0,
            };
        }
        // Weight compliance issues
        for issue in compliance_issues {
            score += match issue.severity {
                ComplianceSeverity::High => 5.0,
                ComplianceSeverity::Medium => 3.0,
                ComplianceSeverity::Low => 1.0,
            };
        }
        // Weight secrets
        for secret in secrets {
            score += 8.0; // Secrets are always high risk
        }
        // Normalize to 0-100 scale
        (score / 100.0_f64).min(100.0_f64)
    }
    /// Generate recommendations
    fn generate_recommendations(
        &self,
        vulnerabilities: &[Vulnerability],
        compliance_issues: &[ComplianceIssue],
    ) -> Vec<String> {
        let mut recommendations = Vec::new();
        // Critical vulnerability recommendations
        if vulnerabilities.iter().any(|v| v.severity == VulnerabilitySeverity::Critical) {
            recommendations.push(
                "Address critical vulnerabilities immediately".to_string()
            );
        }
        // Compliance recommendations
        if compliance_issues.iter().any(|i| i.severity == ComplianceSeverity::High) {
            recommendations.push(
                "Implement high-priority compliance recommendations".to_string()
            );
        }
        // General recommendations
        recommendations.extend(vec![
            "Use multi-stage builds to minimize attack surface".to_string(),
            "Scan images before deployment".to_string(),
            "Keep base images up to date".to_string(),
            "Use minimal base images (alpine, distroless)".to_string(),
            "Implement least privilege principle".to_string(),
        ]);
        recommendations
    }
    /// Check if image has non-root user
    fn has_non_root_user(&self, image: &ContainerImage) -> bool {
        image.layers.iter().any(|layer| {
            layer.files.contains(&"/etc/passwd".to_string()) &&
            layer.files.contains(&"/etc/shadow".to_string())
        })
    }
    /// Check if image has read-only root filesystem
    fn has_read_only_root(&self, image: &ContainerImage) -> bool {
        // In a real implementation, check Dockerfile for READONLY mount
        false
    }
    /// Check if image drops capabilities
    fn drops_capabilities(&self, image: &ContainerImage) -> bool {
        // In a real implementation, check Dockerfile for --cap-drop
        false
    }
    /// Check if image has health check
    fn has_health_check(&self, image: &ContainerImage) -> bool {
        image.healthcheck.is_some()
    }
}
/// Container image structure
#[derive(Debug, Clone)]
pub struct ContainerImage {
    /// Image name
    pub name: String,
    /// Image digest
    pub digest: String,
    /// Image layers
    pub layers: Vec<ImageLayer>,
    /// Healthcheck configuration
    pub healthcheck: Option<HealthCheckConfig>,
}
/// Image layer structure
#[derive(Debug, Clone)]
pub struct ImageLayer {
    /// Layer ID
    pub id: String,
    /// Layer size
    pub size: u64,
    /// Installed packages
    pub packages: Vec<String>,
    /// Files in layer
    pub files: Vec<String>,
    /// Is base layer
    pub is_base_layer: bool,
}
impl ImageLayer {
    /// Check if this is the base layer
    pub fn is_base_layer(&self) -> bool {
        self.is_base_layer
    }
}
/// Vulnerability structure
#[derive(Debug, Clone, PartialEq)]
pub struct Vulnerability {
    /// Vulnerability ID (e.g., CVE-2023-1234)
    pub id: String,
    /// Severity level
    pub severity: VulnerabilitySeverity,
    /// Description
    pub description: String,
    /// Affected packages
    pub affected_packages: Vec<String>,
    /// Fixed version
    pub fixed_version: Option<String>,
    /// CVSS score
    pub cvss_score: Option<f64>,
}
/// Vulnerability severity
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VulnerabilitySeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}
/// Compliance issue structure
#[derive(Debug, Clone)]
pub struct ComplianceIssue {
    /// Rule ID (e.g., CIS-4.1)
    pub rule_id: String,
    /// Severity level
    pub severity: ComplianceSeverity,
    /// Description
    pub description: String,
    /// Remediation steps
    pub remediation: String,
}
/// Compliance severity
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComplianceSeverity {
    High,
    Medium,
    Low,
}
/// Secret structure
#[derive(Debug, Clone)]
pub struct Secret {
    /// Secret type
    pub secret_type: String,
    /// File path where secret was found
    pub file_path: String,
    /// Line number
    pub line_number: Option<u32>,
    /// Remediation
    pub remediation: String,
}
/// Scan report structure
#[derive(Debug, Clone)]
pub struct ScanReport {
    /// Image name
    pub image_name: String,
    /// Image digest
    pub image_digest: String,
    /// Scan timestamp
    pub scan_timestamp: std::time::SystemTime,
    /// Vulnerabilities found
    pub vulnerabilities: Vec<Vulnerability>,
    /// Compliance issues
    pub compliance_issues: Vec<ComplianceIssue>,
    /// Secrets found
    pub secrets: Vec<Secret>,
    /// Risk score (0-100)
    pub risk_score: f64,
    /// Recommendations
    pub recommendations: Vec<String>,
}
impl ScanReport {
    /// Get total vulnerability count
    pub fn vulnerability_count(&self) -> usize {
        self.vulnerabilities.len()
    }
    /// Get critical vulnerability count
    pub fn critical_vulnerability_count(&self) -> usize {
        self.vulnerabilities
            .iter()
            .filter(|v| v.severity == VulnerabilitySeverity::Critical)
            .count()
    }
    /// Get high severity issue count
    pub fn high_severity_issue_count(&self) -> usize {
        let vuln_count: _ = self.vulnerabilities
            .iter()
            .filter(|v| v.severity == VulnerabilitySeverity::High)
            .count();
        let compliance_count: _ = self.compliance_issues
            .iter()
            .filter(|i| i.severity == ComplianceSeverity::High)
            .count();
        vuln_count + compliance_count
    }
    /// Check if scan passed
    pub fn passed(&self) -> bool {
        self.risk_score < 70.0 &&
        self.critical_vulnerability_count() == 0 &&
        self.secrets.is_empty()
    }
}
/// Vulnerability database
struct VulnerabilityDatabase {
    /// Known vulnerabilities
    vulnerabilities: Vec<Vulnerability>,
}
impl VulnerabilityDatabase {
    /// Create a new vulnerability database
    fn new() -> Self {
        let vulnerabilities: _ = Self::load_vulnerabilities();
        Self { vulnerabilities }
    }
    /// Load known vulnerabilities
    fn load_vulnerabilities() -> Vec<Vulnerability> {
        vec![
            Vulnerability {
                id: "CVE-2023-1234".to_string(),
                severity: VulnerabilitySeverity::High,
                description: "Buffer overflow in package X".to_string(),
                affected_packages: vec!["package-x".to_string()],
                fixed_version: Some("1.2.3".to_string()),
                cvss_score: Some(7.5),
            },
            Vulnerability {
                id: "CVE-2023-5678".to_string(),
                severity: VulnerabilitySeverity::Critical,
                description: "Remote code execution in package Y".to_string(),
                affected_packages: vec!["package-y".to_string()],
                fixed_version: Some("2.0.0".to_string()),
                cvss_score: Some(9.8),
            },
        ]
    }
    /// Get all vulnerabilities
    fn get_vulnerabilities(&self) -> &[Vulnerability] {
        &self.vulnerabilities
    }
}
/// Scan configuration
#[derive(Debug, Clone)]
pub struct ScanConfig {
    /// Enable vulnerability scanning
    pub scan_vulnerabilities: bool,
    /// Enable compliance scanning
    pub scan_compliance: bool,
    /// Enable secret scanning
    pub scan_secrets: bool,
    /// Minimum severity to report
    pub min_severity: VulnerabilitySeverity,
}
/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Test command
    pub test: Vec<String>,
    /// Interval
    pub interval: u32,
    /// Timeout
    pub timeout: u32,
    /// Retries
    pub retries: u32,
}
/// Error type
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Scan failed: {0}")]
    ScanFailed(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
    #[error("Other error: {0}")]
    Other(String),
}
#[cfg(test)]
mod tests {
    use super::*;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};
    #[test]
    fn test_vulnerability_severity_ordering() {
        assert!(VulnerabilitySeverity::Critical > VulnerabilitySeverity::High);
        assert!(VulnerabilitySeverity::High > VulnerabilitySeverity::Medium);
        assert!(VulnerabilitySeverity::Medium > VulnerabilitySeverity::Low);
        assert!(VulnerabilitySeverity::Low > VulnerabilitySeverity::Info);
    }
    #[test]
    fn test_compliance_severity_ordering() {
        assert!(ComplianceSeverity::High > ComplianceSeverity::Medium);
        assert!(ComplianceSeverity::Medium > ComplianceSeverity::Low);
    }
    #[test]
    fn test_scan_report_passing() {
        let report: _ = ScanReport {
            image_name: "test-image".to_string(),
            image_digest: "sha256:123".to_string(),
            scan_timestamp: std::time::SystemTime::now(),
            vulnerabilities: vec![
                Vulnerability {
                    id: "CVE-2023-1234".to_string(),
                    severity: VulnerabilitySeverity::Medium,
                    description: "Test".to_string(),
                    affected_packages: vec![],
                    fixed_version: None,
                    cvss_score: None,
                },
            ],
            compliance_issues: vec![],
            secrets: vec![],
            risk_score: 50.0,
            recommendations: vec![],
        };
        assert!(report.passed());
    }
    #[test]
    fn test_scan_report_failing() {
        let report: _ = ScanReport {
            image_name: "test-image".to_string(),
            image_digest: "sha256:123".to_string(),
            scan_timestamp: std::time::SystemTime::now(),
            vulnerabilities: vec![
                Vulnerability {
                    id: "CVE-2023-1234".to_string(),
                    severity: VulnerabilitySeverity::Critical,
                    description: "Test".to_string(),
                    affected_packages: vec![],
                    fixed_version: None,
                    cvss_score: None,
                },
            ],
            compliance_issues: vec![],
            secrets: vec![
                Secret {
                    secret_type: "AWS_ACCESS_KEY_ID".to_string(),
                    file_path: "/etc/secret".to_string(),
                    line_number: Some(1),
                    remediation: "Remove secret".to_string(),
                },
            ],
            risk_score: 80.0,
            recommendations: vec![],
        };
        assert!(!report.passed());
    }
}