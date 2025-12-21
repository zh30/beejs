//! Stage 87: Offline Mode Tests
//! Test-driven development for offline mode functionality

#[cfg(test)]
mod tests {
    use beejs::edge::local_cache::*;
    use beejs::edge::offline_engine::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_local_code_cache_creation() {
        let cache_dir = PathBuf::from("/tmp/beejs_offline_cache_test");
        let cache = LocalCodeCache::new(cache_dir).await.unwrap();

        assert!(cache.cache_dir().exists());
    }

    #[tokio::test]
    async fn test_store_and_load_script() {
        let cache_dir = PathBuf::from("/tmp/beejs_offline_cache_test_2");
        let cache = LocalCodeCache::new(cache_dir).await.unwrap();

        let script = Script {
            name: "test_script".to_string(),
            content: "console.log('Hello, World!');".to_string(),
            version: "1.0.0".to_string(),
            timestamp: std::time::SystemTime::now(),
        };

        // Store script
        let result = cache.store_script("test_key", &script).await;
        assert!(result.is_ok());

        // Load script
        let loaded = cache.load_script("test_key").await.unwrap();
        assert!(loaded.is_some());

        let loaded_script = loaded.unwrap();
        assert_eq!(loaded_script.name, "test_script");
        assert_eq!(loaded_script.content, "console.log('Hello, World!');");
    }

    #[tokio::test]
    async fn test_load_nonexistent_script() {
        let cache_dir = PathBuf::from("/tmp/beejs_offline_cache_test_3");
        let cache = LocalCodeCache::new(cache_dir).await.unwrap();

        let loaded = cache.load_script("nonexistent").await.unwrap();
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn test_cleanup_expired_caches() {
        let cache_dir = PathBuf::from("/tmp/beejs_offline_cache_test_4");
        let cache = LocalCodeCache::new(cache_dir).await.unwrap();

        let script = Script {
            name: "expired_script".to_string(),
            content: "console.log('expired');".to_string(),
            version: "1.0.0".to_string(),
            timestamp: std::time::SystemTime::now(),
        };

        cache.store_script("expired_key", &script).await.unwrap();

        // Simulate old timestamp
        cache.set_expiration("expired_key", std::time::Duration::from_secs(0)).await;

        let cleaned = cache.cleanup_expired().await.unwrap();
        assert_eq!(cleaned, 1);
    }

    #[tokio::test]
    async fn test_offline_data_store_creation() {
        let db_path = PathBuf::from("/tmp/beejs_offline_db_test.db");
        let _store = OfflineDataStore::new(db_path.clone()).await.unwrap();

        assert!(db_path.exists());
    }

    #[tokio::test]
    async fn test_store_and_load_data() {
        let db_path = PathBuf::from("/tmp/beejs_offline_db_test_2.db");
        let store = OfflineDataStore::new(db_path).await.unwrap();

        let key = "test_key";
        let data = b"Hello, World!";

        // Store data
        let result = store.store_data(key, data).await;
        assert!(result.is_ok());

        // Load data
        let loaded = store.load_data(key).await.unwrap();
        assert!(loaded.is_some());

        assert_eq!(loaded.unwrap(), data);
    }

    #[tokio::test]
    async fn test_offline_execution_engine_creation() {
        let cache_dir = PathBuf::from("/tmp/beejs_offline_cache_test_5");
        let db_path = PathBuf::from("/tmp/beejs_offline_db_test_3.db");

        let cache = LocalCodeCache::new(cache_dir).await.unwrap();
        let store = OfflineDataStore::new(db_path).await.unwrap();

        let engine = OfflineExecutionEngine::new(cache, store).await.unwrap();
        assert!(engine.runtime().is_some());
    }

    #[tokio::test]
    async fn test_execute_offline_script() {
        let cache_dir = PathBuf::from("/tmp/beejs_offline_cache_test_6");
        let db_path = PathBuf::from("/tmp/beejs_offline_db_test_4.db");

        let cache = LocalCodeCache::new(cache_dir).await.unwrap();
        let store = OfflineDataStore::new(db_path).await.unwrap();

        let engine = OfflineExecutionEngine::new(cache, store).await.unwrap();

        let script = "console.log('Hello from offline mode');";

        let result = engine.execute_offline(script).await.unwrap();
        assert!(result.success);
        assert!(result.output.is_some());
    }

    #[tokio::test]
    async fn test_dependency_resolution() {
        let cache_dir = PathBuf::from("/tmp/beejs_offline_cache_test_7");
        let db_path = PathBuf::from("/tmp/beejs_offline_db_test_5.db");

        let cache = LocalCodeCache::new(cache_dir).await.unwrap();
        let store = OfflineDataStore::new(db_path).await.unwrap();

        let engine = OfflineExecutionEngine::new(cache, store).await.unwrap();

        let script = r#"
            const fs = require('fs');
            const path = require('path');
            console.log('script with dependencies');
        "#;

        let result = engine.resolve_dependencies(script).await.unwrap();
        assert!(!result.dependencies.is_empty());
        assert!(result.dependencies.iter().any(|d| d.name == "fs"));
        assert!(result.dependencies.iter().any(|d| d.name == "path"));
    }

    #[tokio::test]
    async fn test_sync_manager_creation() {
        let _sync_manager = SyncManager::new().await.unwrap();
        // SyncManager created successfully
    }

    #[tokio::test]
    async fn test_sync_data() {
        let sync_manager = SyncManager::new().await.unwrap();

        let result = sync_manager.sync_data().await.unwrap();
        assert!(result.synced_items >= 0);
        assert!(result.failed_items >= 0);
    }

    #[tokio::test]
    async fn test_conflict_resolution() {
        let sync_manager = SyncManager::new().await.unwrap();

        let conflicts = vec![
            Conflict {
                key: "test_key".to_string(),
                local_value: b"local".to_vec(),
                remote_value: b"remote".to_vec(),
                timestamp: std::time::SystemTime::now(),
            }
        ];

        let resolutions = sync_manager.resolve_conflicts(&conflicts).await.unwrap();
        assert!(!resolutions.is_empty());

        let resolution = &resolutions[0];
        assert_eq!(resolution.key, "test_key");
        assert!(resolution.resolved);
    }
}
