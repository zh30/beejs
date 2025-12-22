//! 传输加密 (TLS)
//!
//! 提供 TLS 1.3 配置和证书管理功能

use std::collections::HashMap;
use std::sync::{Arc, Mutex, atomic::Ordering};
use std::time::SystemTime;

use std::cmp::Ordering;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::collections::{HashMap, BTreeMap};
/// TLS 错误
#[derive(Error, Debug)]
pub enum TlsError {
    #[error("Certificate error: {0}")]
    CertificateError(String),
    #[error("Handshake failed")]
    HandshakeFailed,
    #[error("Invalid configuration")]
    InvalidConfig,
}
/// TLS 版本
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TlsVersion {
    V1_2,
    V1_3,
}
impl PartialOrd for TlsVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(match (self, other) {
            (TlsVersion::V1_3, TlsVersion::V1_2) => Ordering::Greater,
            (TlsVersion::V1_2, TlsVersion::V1_3) => Ordering::Less,
            _ => Ordering::Equal,
        })
    }
}
/// 密码套件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CipherSuite {
    Aes256Gcm,
    Aes128Gcm,
    Chacha20Poly1305,
}
/// TLS 配置
#[derive(Debug)]
pub struct TlsConfig {
    pub min_version: TlsVersion,
    pub cipher_suites: Vec<CipherSuite>,
    pub cert_manager: Arc<CertificateManager>,
}
/// 证书管理器
#[derive(Debug)]
pub struct CertificateManager {
    // 证书存储
    certificates: std::collections::HashMap<String, Certificate>,
}
impl CertificateManager {
    pub fn new() -> Result<Self, TlsError> {
        Ok(Self {
            certificates: std::collections::HashMap::new(),
        })
    }
    pub async fn load_certificate(&mut self, cert_data: &[u8], key_data: &[u8]) -> Result<String, TlsError> {
        // 简化的证书加载（生产环境应使用真实的 X.509 解析）
        let cert_id: _ = format!("cert-{}", cert_data.len());
        let certificate: _ = Certificate {
            id: cert_id.clone(),
            data: cert_data.to_vec(),
            private_key: key_data.to_vec(),
            issued_at: std::time::SystemTime::now(),
        };
        self.certificates.insert(cert_id.clone(), certificate);
        Ok(cert_id)
    }
    pub async fn get_certificate(&self, cert_id: &str) -> Result<&Certificate, TlsError> {
        self.certificates.get(cert_id)
            .ok_or_else(|| TlsError::CertificateError(format!("Certificate not found: {}", cert_id))
    }
    pub async fn validate_certificate(&self, cert_id: &str) -> Result<bool, TlsError> {
        // 简化的证书验证（生产环境应验证证书链、过期时间等）
        self.certificates.contains_key(cert_id)
            .then(|| true)
            .ok_or_else(|| TlsError::CertificateError("Invalid certificate".to_string())
    }
}
/// 证书
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    pub id: String,
    pub data: Vec<u8>,
    pub private_key: Vec<u8>,
    pub issued_at: std::time::SystemTime,
}
impl TlsConfig {
    pub fn new() -> Self {
        Self {
            min_version: TlsVersion::V1_3,
            cipher_suites: vec![
                CipherSuite::Aes256Gcm,
                CipherSuite::Chacha20Poly1305,
                CipherSuite::Aes128Gcm,
            ],
            cert_manager: Arc::new(Mutex::new(CertificateManager::new()),.expect("Failed to create certificate manager")),
        }
    }
    pub fn with_min_version(mut self, version: TlsVersion) -> Self {
        self.min_version = version;
        self
    }
    pub fn with_cipher_suites(mut self, cipher_suites: Vec<CipherSuite>) -> Self {
        self.cipher_suites = cipher_suites;
        self
    }
    pub fn validate(&self) -> Result<(), TlsError> {
        if self.cipher_suites.is_empty() {
            return Err(TlsError::InvalidConfig);
        }
        if self.min_version < TlsVersion::V1_3 {
            return Err(TlsError::InvalidConfig);
        }
        Ok(())
    }
}
// 默认实现
impl Default for TlsConfig {
    fn default() -> Self {
        Self::new()
    }
}
impl Default for CertificateManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default certificate manager")
    }
}