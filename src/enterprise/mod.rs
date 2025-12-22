//! Enterprise Module
//! Provides enterprise-grade security, compliance, and governance features

use std::sync::Arc;

pub mod security_manager;
pub mod compliance_manager;

// Stage 96 Phase 2: Enterprise Features
pub mod k8s;
pub mod tenancy;
pub mod monitoring;

pub use security_manager::*;
pub use compliance_manager::*;

// Re-export Stage 96 Phase 2 types
pub use k8s::operator::{
    BeejsCluster, BeejsClusterSpec, BeejsClusterStatus, ClusterPhase,
    Condition, ResourceRequirements, NetworkingConfig, ServiceType,
    IngressConfig, Operator, OperatorConfig, OperatorEvent,
};

pub use tenancy::manager::{
    TenantId, Tenant, TenantStatus, ResourceQuota, SecurityContext,
    NetworkPolicy, RbacRole, Permission, ResourceLimits, ExecutionContext,
    TenancyManager, ResourceUsage, TenantUpdates,
};

pub use monitoring::metrics::{
    Metric, ClusterMetrics, TenantMetrics, SystemMetrics, Alert,
    AlertSeverity, AlertCondition, ComparisonOperator, AlertEvent,
    AlertStatus, MonitoringConfig, MonitoringManager,
};

/// Unified enterprise manager
#[derive(Debug)]
pub struct EnterpriseManager {
    security: Arc<SecurityManager>,
    compliance: Arc<ComplianceManager>,
}

impl EnterpriseManager {
    /// Create a new enterprise manager
    pub fn new() -> Self {
        EnterpriseManager {
            security: Arc::new(SecurityManager::new()),
            compliance: Arc::new(ComplianceManager::new()),
        }
    }

    /// Get security manager
    pub fn security(&self) -> &Arc<SecurityManager> {
        &self.security
    }

    /// Get compliance manager
    pub fn compliance(&self) -> &Arc<ComplianceManager> {
        &self.compliance
    }

    /// Run full enterprise security and compliance check
    pub async fn run_full_audit(&self, script: &str, user_id: &str) -> Result<EnterpriseAuditResult> {
        // Run security check
        let security_result = self.security.enforce_policy(script, user_id).await?;

        // Run compliance checks for multiple frameworks
        let mut compliance_reports = Vec::new();

        let frameworks = vec![
            ComplianceFramework::GDPR,
            ComplianceFramework::HIPAA,
            ComplianceFramework::SOC2,
            ComplianceFramework::ISO27001,
        ];

        for framework in frameworks {
            let report = self.compliance.check_compliance(script, framework).await?;
            compliance_reports.push(report);
        }

        // Calculate overall score
        let avg_score = compliance_reports.iter()
            .map(|r| r.score)
            .sum::<f64>() / compliance_reports.len() as f64;

        let overall_compliant = security_result.allowed && avg_score >= 0.8;

        Ok(EnterpriseAuditResult {
            security: security_result,
            compliance: compliance_reports,
            overall_score: avg_score,
            compliant: overall_compliant,
            timestamp: std::time::SystemTime::now(),
        })
    }
}

impl Default for EnterpriseManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Enterprise audit result
#[derive(Debug)]
pub struct EnterpriseAuditResult {
    pub security: SecurityResult,
    pub compliance: Vec<ComplianceReport>,
    pub overall_score: f64,
    pub compliant: bool,
    pub timestamp: std::time::SystemTime,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_enterprise_manager() {
        let manager = EnterpriseManager::new();

        let script = "console.log('Hello');";
        let user_id = "test_user";

        let result = manager.run_full_audit(script, user_id).await.unwrap();

        assert!(result.overall_score >= 0.0 && result.overall_score <= 1.0);
        assert!(!result.compliance.is_empty());
        assert!(result.timestamp <= std::time::SystemTime::now());
    }

    #[tokio::test]
    async fn test_security_and_compliance() {
        let manager = EnterpriseManager::new();

        // Test with safe code
        let safe_script = "const x = 42;";
        let result = manager.run_full_audit(safe_script, "user1").await.unwrap();
        assert!(result.security.allowed);

        // Test with unsafe code
        let unsafe_script = "eval('alert(1)');";
        let result = manager.run_full_audit(unsafe_script, "user2").await.unwrap();
        assert!(!result.security.allowed);
    }
}
