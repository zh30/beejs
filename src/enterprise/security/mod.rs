//! Enterprise Security Module
//! Provides enterprise-grade security features including sandbox, encryption, and key management

pub mod sandbox;
pub mod encryption;

pub use sandbox::*;
pub use encryption::*;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Unified security manager
#[derive(Debug)]
pub struct SecurityModule {
    pub sandbox: SecuritySandbox,
    pub kms: KeyManagementService,
}

impl SecurityModule {
    /// Create a new security module
    pub fn new(
        sandbox_config: SandboxConfig,
        encryption_config: EncryptionConfig,
    ) -> Result<Self> {
        let sandbox = SecuritySandbox::new(sandbox_config)?;
        let kms = KeyManagementService::new(encryption_config);

        Ok(Self {
            sandbox,
            kms,
        })
    }
}
