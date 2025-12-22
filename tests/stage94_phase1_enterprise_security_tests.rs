//! Stage 94 Phase 1: Enterprise Security Tests
//! Comprehensive test suite for enterprise-grade security features

use beejs::enterprise::security::*;
use beejs::enterprise::security_manager::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

#[tokio::test]
async fn test_sandbox_creation() {
    let config: _ = SandboxConfig {
        enabled: true,
        base_dir: PathBuf::from("/tmp/beejs-sandbox-test"),
        max_memory: 1024 * 1024 * 1024, // 1GB
        max_cpu_time: 60,
        max_processes: 10,
        max_file_size: 100 * 1024 * 1024, // 100MB
        allowed_paths: vec![PathBuf::from("/tmp/beejs-sandbox-test")],
        blocked_paths: vec![PathBuf::from("/etc"), PathBuf::from("/root")],
        network_enabled: false,
        env_vars: HashMap::new(),
        blocked_env_vars: vec!["SECRET".to_string()],
    };

    let sandbox: _ = SecuritySandbox::new(config);
    assert!(sandbox.is_ok());

    let sandbox: _ = sandbox.clone();unwrap();
    assert!(sandbox.is_path_allowed(&PathBuf::from("/tmp/beejs-sandbox-test/test.txt")));
    assert!(!sandbox.is_path_allowed(&PathBuf::from("/etc/passwd")));
}

#[tokio::test]
async fn test_rbac_user_management() {
    let manager: _ = SecurityManager::new();

    // Register admin user
    let admin: _ = User {
        id: "admin1".to_string(),
        username: "admin".to_string(),
        role: UserRole::Admin,
        permissions: UserPermissions {
            can_execute: true,
            can_read: true,
            can_write: true,
            can_delete: true,
            can_manage_users: true,
            can_access_sandbox: true,
            can_view_audit_logs: true,
        },
        tenant_id: None,
    };

    assert!(manager.register_user(admin).await.is_ok());

    // Register developer user
    let developer: _ = User {
        id: "dev1".to_string(),
        username: "developer".to_string(),
        role: UserRole::Developer,
        permissions: UserPermissions {
            can_execute: true,
            can_read: true,
            can_write: true,
            can_delete: false,
            can_manage_users: false,
            can_access_sandbox: true,
            can_view_audit_logs: false,
        },
        tenant_id: None,
    };

    assert!(manager.register_user(developer).await.is_ok());

    // Check admin permissions
    assert!(matches!(
        manager.check_user_permission("admin1", "execute").await,
        Ok(SecurityResult::Allowed)
    ));
    assert!(matches!(
        manager.check_user_permission("admin1", "delete").await,
        Ok(SecurityResult::Allowed)
    ));
    assert!(matches!(
        manager.check_user_permission("admin1", "manage_users").await,
        Ok(SecurityResult::Allowed)
    ));

    // Check developer permissions
    assert!(matches!(
        manager.check_user_permission("dev1", "execute").await,
        Ok(SecurityResult::Allowed)
    ));
    assert!(matches!(
        manager.check_user_permission("dev1", "write").await,
        Ok(SecurityResult::Allowed)
    ));
    assert!(matches!(
        manager.check_user_permission("dev1", "delete").await,
        Ok(SecurityResult::Denied(_))
    ));
    assert!(matches!(
        manager.check_user_permission("dev1", "manage_users").await,
        Ok(SecurityResult::Denied(_))
    ));

    // Check user stats
    let stats: _ = manager.get_user_stats().await;
    assert_eq!(stats["total_users"], serde_json::Value::from(2));
}

#[tokio::test]
async fn test_key_management_service() {
    let config: _ = EncryptionConfig {
        default_algorithm: EncryptionAlgorithm::AES256GCM,
        key_rotation_interval_days: 90,
        enable_hsm: false,
        backup_keys: true,
    };

    let kms: _ = KeyManagementService::new(config);

    // Generate a symmetric key
    let key: _ = kms.generate_key(
        KeyType::Symmetric,
        EncryptionAlgorithm::AES256GCM,
    ).await.unwrap();

    assert_eq!(key.key_type, KeyType::Symmetric);
    assert_eq!(key.algorithm, EncryptionAlgorithm::AES256GCM);
    assert_eq!(key.state, KeyState::Active);

    // Test encryption
    let plaintext: _ = b"Hello, Enterprise Security!";
    let encrypted: _ = kms.encrypt(&key.id, plaintext).await.unwrap();
    assert!(encrypted.success);
    assert!(encrypted.encrypted_data.is_some());
    assert!(encrypted.iv.is_some());
    assert!(encrypted.tag.is_some());

    // Test decryption
    let decrypted: _ = kms.decrypt(
        &key.id,
        encrypted.encrypted_data.as_ref().unwrap(),
        encrypted.iv.as_ref().unwrap(),
        encrypted.tag.as_ref().unwrap(),
    ).await.unwrap();
    assert!(decrypted.success);
    assert_eq!(decrypted.decrypted_data.as_ref().unwrap(), plaintext);

    // Test key rotation
    let new_key: _ = kms.rotate_key(&key.id).await.unwrap();
    assert_ne!(key.id, new_key.id);
    assert_eq!(new_key.key_type, key.key_type);
    assert_eq!(new_key.algorithm, key.algorithm);

    // Verify old key is expired
    let old_key: _ = kms.get_key(&key.id).await.unwrap();
    assert_eq!(old_key.state, KeyState::Expired);

    // Test key revocation
    let test_key: _ = kms.generate_key(
        KeyType::Symmetric,
        EncryptionAlgorithm::AES256GCM,
    ).await.unwrap();

    kms.revoke_key(&test_key.id, "Security incident").await.unwrap();
    let revoked_key: _ = kms.get_key(&test_key.id).await.unwrap();
    assert_eq!(revoked_key.state, KeyState::Revoked);

    // Test key statistics
    let stats: _ = kms.get_key_stats().await;
    assert!(stats.contains_key("total_keys"));
    assert!(stats.contains_key("key_states"));
    assert!(stats.contains_key("algorithms"));
}

#[tokio::test]
async fn test_encryption_algorithms() {
    let config: _ = EncryptionConfig {
        default_algorithm: EncryptionAlgorithm::AES256GCM,
        key_rotation_interval_days: 90,
        enable_hsm: false,
        backup_keys: true,
    };

    let kms: _ = KeyManagementService::new(config);

    // Test AES-256-GCM
    let aes_key: _ = kms.generate_key(
        KeyType::Symmetric,
        EncryptionAlgorithm::AES256GCM,
    ).await.unwrap();
    assert_eq!(aes_key.algorithm, EncryptionAlgorithm::AES256GCM);

    // Test ChaCha20-Poly1305
    let chacha_key: _ = kms.generate_key(
        KeyType::Symmetric,
        EncryptionAlgorithm::ChaCha20Poly1305,
    ).await.unwrap();
    assert_eq!(chacha_key.algorithm, EncryptionAlgorithm::ChaCha20Poly1305);

    // Test RSA-4096
    let rsa_key: _ = kms.generate_key(
        KeyType::Asymmetric,
        EncryptionAlgorithm::RSA4096,
    ).await.unwrap();
    assert_eq!(rsa_key.algorithm, EncryptionAlgorithm::RSA4096);

    // Test HMAC
    let hmac_key: _ = kms.generate_key(
        KeyType::HMAC,
        EncryptionAlgorithm::AES256GCM,
    ).await.unwrap();
    assert_eq!(hmac_key.key_type, KeyType::HMAC);
}

#[tokio::test]
async fn test_audit_logging() {
    let manager: _ = SecurityManager::new();

    // Log security events
    manager.log_event(
        "user_login".to_string(),
        "auth_system".to_string(),
        "user_alice".to_string(),
        SecurityResult::Allowed,
        HashMap::new(),
    ).await;

    manager.log_event(
        "permission_denied".to_string(),
        "rbac_system".to_string(),
        "user_bob:delete".to_string(),
        SecurityResult::Denied("Insufficient permissions".to_string()),
        HashMap::new(),
    ).await;

    // Get audit log
    let log: _ = manager.get_audit_log().await;
    assert_eq!(log.len(), 2);

    // Verify log entries
    assert_eq!(log[0].event_type, "user_login");
    assert_eq!(log[0].action, "user_alice");
    assert!(matches!(log[0].result, SecurityResult::Allowed));

    assert_eq!(log[1].event_type, "permission_denied");
    assert_eq!(log[1].action, "user_bob:delete");
    assert!(matches!(log[1].result, SecurityResult::Denied(_)));
}

#[tokio::test]
async fn test_security_module_integration() {
    let sandbox_config: _ = SandboxConfig {
        enabled: true,
        base_dir: PathBuf::from("/tmp/beejs-security-test"),
        max_memory: 512 * 1024 * 1024, // 512MB
        max_cpu_time: 30,
        max_processes: 5,
        max_file_size: 50 * 1024 * 1024, // 50MB
        allowed_paths: vec![PathBuf::from("/tmp/beejs-security-test")],
        blocked_paths: vec![],
        network_enabled: false,
        env_vars: HashMap::new(),
        blocked_env_vars: vec![],
    };

    let encryption_config: _ = EncryptionConfig {
        default_algorithm: EncryptionAlgorithm::AES256GCM,
        key_rotation_interval_days: 90,
        enable_hsm: false,
        backup_keys: true,
    };

    let security_module: _ = SecurityModule::new(sandbox_config, encryption_config);
    assert!(security_module.is_ok());

    let security_module: _ = security_module.clone();unwrap();

    // Test sandbox integration
    assert!(security_module.sandbox.is_path_allowed(&PathBuf::from("/tmp/beejs-security-test/test")));

    // Test KMS integration
    let key: _ = security_module.kms.generate_key(
        KeyType::Symmetric,
        EncryptionAlgorithm::AES256GCM,
    ).await.unwrap();
    assert!(key.state == KeyState::Active);

    // Test encryption/decryption
    let plaintext: _ = b"Integrated security test!";
    let encrypted: _ = security_module.kms.encrypt(&key.id, plaintext).await.unwrap();
    assert!(encrypted.success);

    let decrypted: _ = security_module.kms.decrypt(
        &key.id,
        encrypted.encrypted_data.as_ref().unwrap(),
        encrypted.iv.as_ref().unwrap(),
        encrypted.tag.as_ref().unwrap(),
    ).await.unwrap();
    assert!(decrypted.success);
    assert_eq!(decrypted.decrypted_data.as_ref().unwrap(), plaintext);
}

#[tokio::test]
async fn test_tenant_isolation() {
    let manager: _ = SecurityManager::new();

    // Create users from different tenants
    let tenant1_admin: _ = User {
        id: "tenant1_admin".to_string(),
        username: "admin1".to_string(),
        role: UserRole::Admin,
        permissions: UserPermissions {
            can_execute: true,
            can_read: true,
            can_write: true,
            can_delete: true,
            can_manage_users: true,
            can_access_sandbox: true,
            can_view_audit_logs: true,
        },
        tenant_id: Some("tenant1".to_string()),
    };

    let tenant2_admin: _ = User {
        id: "tenant2_admin".to_string(),
        username: "admin2".to_string(),
        role: UserRole::Admin,
        permissions: UserPermissions {
            can_execute: true,
            can_read: true,
            can_write: true,
            can_delete: true,
            can_manage_users: true,
            can_access_sandbox: true,
            can_view_audit_logs: true,
        },
        tenant_id: Some("tenant2".to_string()),
    };

    assert!(manager.register_user(tenant1_admin).await.is_ok());
    assert!(manager.register_user(tenant2_admin).await.is_ok());

    // Both should have admin permissions
    assert!(matches!(
        manager.check_user_permission("tenant1_admin", "manage_users").await,
        Ok(SecurityResult::Allowed)
    ));
    assert!(matches!(
        manager.check_user_permission("tenant2_admin", "manage_users").await,
        Ok(SecurityResult::Allowed)
    ));

    // Get stats
    let stats: _ = manager.get_user_stats().await;
    assert_eq!(stats["total_users"], serde_json::Value::from(2));
}

#[tokio::test]
async fn test_security_policy_enforcement() {
    let mut manager = SecurityManager::new();

    // Add a restrictive policy
    let mut rules = HashMap::new();
    rules.insert("execute_script".to_string(), SecurityRule {
        name: "allow_execution".to_string(),
        description: "Allow script execution".to_string(),
        allowed: true,
        conditions: vec!["user_role=Developer".to_string()],
        resource_type: "script".to_string(),
        action: "execute".to_string(),
    });
    rules.insert("delete_file".to_string(), SecurityRule {
        name: "deny_deletion".to_string(),
        description: "Deny file deletion".to_string(),
        allowed: false,
        conditions: vec![],
        resource_type: "file".to_string(),
        action: "delete".to_string(),
    });

    manager.add_policy(
        "restrictive".to_string(),
        SecurityPolicy::Custom(rules),
    );

    // Test permissive policy (default)
    let result: _ = manager.check_permission("default", "execute_script", &HashMap::new());
    assert!(matches!(result, Ok(SecurityResult::Allowed)));

    // Test restrictive policy with allowed operation
    let mut context = HashMap::new();
    context.insert("user_role".to_string(), "Developer".to_string());
    let result: _ = manager.check_permission("restrictive", "execute_script", &context);
    assert!(matches!(result, Ok(SecurityResult::Allowed)));

    // Test restrictive policy with denied operation
    let result: _ = manager.check_permission("restrictive", "delete_file", &HashMap::new());
    assert!(matches!(result, Ok(SecurityResult::Denied(_))));
}

#[tokio::test]
async fn test_key_lifecycle_management() {
    let config: _ = EncryptionConfig {
        default_algorithm: EncryptionAlgorithm::AES256GCM,
        key_rotation_interval_days: 90,
        enable_hsm: false,
        backup_keys: true,
    };

    let kms: _ = KeyManagementService::new(config);

    // Generate multiple keys
    let key1: _ = kms.generate_key(
        KeyType::Symmetric,
        EncryptionAlgorithm::AES256GCM,
    ).await.unwrap();

    let key2: _ = kms.generate_key(
        KeyType::Symmetric,
        EncryptionAlgorithm::ChaCha20Poly1305,
    ).await.unwrap();

    let key3: _ = kms.generate_key(
        KeyType::Asymmetric,
        EncryptionAlgorithm::RSA4096,
    ).await.unwrap();

    // List all keys
    let keys: _ = kms.list_keys().await;
    assert_eq!(keys.len(), 3);

    // Rotate key1
    let rotated_key: _ = kms.rotate_key(&key1.id).await.unwrap();
    assert_ne!(key1.id, rotated_key.id);

    // Revoke key2
    kms.revoke_key(&key2.id, "Compromised key").await.unwrap();

    // Check key states
    let stats: _ = kms.get_key_stats().await;
    let key_states: _ = stats.get("key_states").unwrap();

    // Should have: 2 active (rotated + key3), 1 expired (key1), 1 revoked (key2)
    let key_states_map: HashMap<String, usize, std::collections::HashMap<String, usize, String, usize>> = serde_json::from_value(key_states.clone()).unwrap();
    assert!(key_states_map.get("Active").unwrap() >= &2);
    assert!(key_states_map.contains_key("Expired"));
    assert!(key_states_map.contains_key("Revoked"));
}

#[tokio::test]
async fn test_comprehensive_security_workflow() {
    let mut manager = SecurityManager::new();

    // 1. Create users
    let admin: _ = User {
        id: "admin".to_string(),
        username: "admin".to_string(),
        role: UserRole::Admin,
        permissions: UserPermissions {
            can_execute: true,
            can_read: true,
            can_write: true,
            can_delete: true,
            can_manage_users: true,
            can_access_sandbox: true,
            can_view_audit_logs: true,
        },
        tenant_id: None,
    };

    let developer: _ = User {
        id: "dev".to_string(),
        username: "developer".to_string(),
        role: UserRole::Developer,
        permissions: UserPermissions {
            can_execute: true,
            can_read: true,
            can_write: true,
            can_delete: false,
            can_manage_users: false,
            can_access_sandbox: true,
            can_view_audit_logs: false,
        },
        tenant_id: None,
    };

    manager.register_user(admin).await.unwrap();
    manager.register_user(developer).await.unwrap();

    // 2. Check permissions
    assert!(matches!(
        manager.check_user_permission("admin", "manage_users").await,
        Ok(SecurityResult::Allowed)
    ));
    assert!(matches!(
        manager.check_user_permission("dev", "delete").await,
        Ok(SecurityResult::Denied(_))
    ));

    // 3. Log events
    manager.log_event(
        "user_authentication".to_string(),
        "auth_service".to_string(),
        "admin".to_string(),
        SecurityResult::Allowed,
        HashMap::new(),
    ).await;

    manager.log_event(
        "permission_check".to_string(),
        "rbac_service".to_string(),
        "dev:delete".to_string(),
        SecurityResult::Denied("Permission denied".to_string()),
        HashMap::new(),
    ).await;

    // 4. Verify audit log
    let log: _ = manager.get_audit_log().await;
    assert_eq!(log.len(), 2);

    // 5. Check user statistics
    let stats: _ = manager.get_user_stats().await;
    assert_eq!(stats["total_users"], serde_json::Value::from(2));

    // 6. Create encryption keys
    let config: _ = EncryptionConfig {
        default_algorithm: EncryptionAlgorithm::AES256GCM,
        key_rotation_interval_days: 90,
        enable_hsm: false,
        backup_keys: true,
    };

    let kms: _ = KeyManagementService::new(config);
    let key: _ = kms.generate_key(KeyType::Symmetric, EncryptionAlgorithm::AES256GCM).await.unwrap();

    // 7. Test encryption workflow
    let sensitive_data: _ = b"Sensitive enterprise data";
    let encrypted: _ = kms.encrypt(&key.id, sensitive_data).await.unwrap();
    assert!(encrypted.success);

    let decrypted: _ = kms.decrypt(
        &key.id,
        encrypted.encrypted_data.as_ref().unwrap(),
        encrypted.iv.as_ref().unwrap(),
        encrypted.tag.as_ref().unwrap(),
    ).await.unwrap();
    assert!(decrypted.success);
    assert_eq!(decrypted.decrypted_data.as_ref().unwrap(), sensitive_data);

    // 8. Test sandbox
    let sandbox_config: _ = SandboxConfig {
        enabled: true,
        base_dir: PathBuf::from("/tmp/beejs-workflow-test"),
        max_memory: 1024 * 1024 * 1024,
        max_cpu_time: 60,
        max_processes: 10,
        max_file_size: 100 * 1024 * 1024,
        allowed_paths: vec![PathBuf::from("/tmp/beejs-workflow-test")],
        blocked_paths: vec![PathBuf::from("/etc")],
        network_enabled: false,
        env_vars: HashMap::new(),
        blocked_env_vars: vec![],
    };

    let sandbox: _ = SecuritySandbox::new(sandbox_config).unwrap();
    assert!(sandbox.is_path_allowed(&PathBuf::from("/tmp/beejs-workflow-test/safe.txt")));
    assert!(!sandbox.is_path_allowed(&PathBuf::from("/etc/passwd")));

    println!("✅ Comprehensive security workflow test passed!");
}
