//! Stage 75: Watch Variables Tests
//!
//! Tests for the watch variable functionality in the debugger.
//! Watch expressions allow developers to monitor variable values
//! during debugging sessions.

use beejs::debugger::{
    DebuggerEngine, DebugConfig, DebugState,
};

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================
    // Watch Expression Manager Tests
    // =========================================

    /// Test 1: WatchManager creation
    #[test]
    fn test_watch_manager_creation() {
        use beejs::debugger::watch::WatchManager;

        let manager = WatchManager::new();
        assert_eq!(manager.count(), 0, "Should start with no watches");
    }

    /// Test 2: Add watch expression
    #[test]
    fn test_add_watch_expression() {
        use beejs::debugger::watch::WatchManager;

        let mut manager = WatchManager::new();
        let result = manager.add("x + y");

        assert!(result.is_ok(), "Should add watch successfully");
        assert_eq!(manager.count(), 1, "Should have 1 watch");

        let watch = result.unwrap();
        assert_eq!(watch.expression, "x + y");
        assert!(!watch.id.is_empty(), "Should have an ID");
    }

    /// Test 3: Remove watch expression
    #[test]
    fn test_remove_watch_expression() {
        use beejs::debugger::watch::WatchManager;

        let mut manager = WatchManager::new();
        let watch = manager.add("counter").unwrap();
        let watch_id = watch.id.clone();

        assert_eq!(manager.count(), 1);

        let result = manager.remove(&watch_id);
        assert!(result.is_ok(), "Should remove watch successfully");
        assert_eq!(manager.count(), 0, "Should have no watches");
    }

    /// Test 4: Remove non-existent watch
    #[test]
    fn test_remove_nonexistent_watch() {
        use beejs::debugger::watch::WatchManager;

        let mut manager = WatchManager::new();
        let result = manager.remove("nonexistent");

        assert!(result.is_err(), "Should fail to remove non-existent watch");
    }

    /// Test 5: List all watches
    #[test]
    fn test_list_all_watches() {
        use beejs::debugger::watch::WatchManager;

        let mut manager = WatchManager::new();
        manager.add("x").unwrap();
        manager.add("y").unwrap();
        manager.add("z").unwrap();

        let watches = manager.list();
        assert_eq!(watches.len(), 3, "Should have 3 watches");
    }

    /// Test 6: Get watch by ID
    #[test]
    fn test_get_watch_by_id() {
        use beejs::debugger::watch::WatchManager;

        let mut manager = WatchManager::new();
        let watch = manager.add("myVar").unwrap();
        let watch_id = watch.id.clone();

        let retrieved = manager.get(&watch_id);
        assert!(retrieved.is_some(), "Should find watch by ID");
        assert_eq!(retrieved.unwrap().expression, "myVar");
    }

    /// Test 7: Update watch value
    #[test]
    fn test_update_watch_value() {
        use beejs::debugger::watch::WatchManager;

        let mut manager = WatchManager::new();
        let watch = manager.add("counter").unwrap();
        let watch_id = watch.id.clone();

        // Update the value
        let result = manager.update_value(&watch_id, "42", "number");
        assert!(result.is_ok(), "Should update value successfully");

        let watch = manager.get(&watch_id).unwrap();
        assert_eq!(watch.last_value, Some("42".to_string()));
        assert_eq!(watch.value_type, Some("number".to_string()));
    }

    /// Test 8: Clear all watches
    #[test]
    fn test_clear_all_watches() {
        use beejs::debugger::watch::WatchManager;

        let mut manager = WatchManager::new();
        manager.add("a").unwrap();
        manager.add("b").unwrap();
        manager.add("c").unwrap();

        assert_eq!(manager.count(), 3);

        manager.clear();
        assert_eq!(manager.count(), 0, "Should have no watches after clear");
    }

    /// Test 9: Watch expression with error
    #[test]
    fn test_watch_with_error() {
        use beejs::debugger::watch::WatchManager;

        let mut manager = WatchManager::new();
        let watch = manager.add("invalidExpr").unwrap();
        let watch_id = watch.id.clone();

        // Set an error on the watch
        let result = manager.set_error(&watch_id, "ReferenceError: invalidExpr is not defined");
        assert!(result.is_ok(), "Should set error successfully");

        let watch = manager.get(&watch_id).unwrap();
        assert!(watch.has_error, "Should have error flag set");
        assert!(watch.error_message.is_some());
    }

    /// Test 10: Duplicate watch expressions
    #[test]
    fn test_duplicate_watch_expressions() {
        use beejs::debugger::watch::WatchManager;

        let mut manager = WatchManager::new();
        manager.add("x").unwrap();
        let result = manager.add("x");

        // Should allow duplicate expressions (they might be in different contexts)
        assert!(result.is_ok(), "Should allow duplicate expressions");
        assert_eq!(manager.count(), 2);
    }

    // =========================================
    // DebuggerEngine Watch Integration Tests
    // =========================================

    /// Test 11: DebuggerEngine add watch
    #[test]
    fn test_debugger_engine_add_watch() {
        let mut engine = DebuggerEngine::new_default();

        let result = engine.add_watch("myVariable");
        assert!(result.success, "Should add watch through engine");

        let watches = engine.get_all_watches();
        assert_eq!(watches.len(), 1);
    }

    /// Test 12: DebuggerEngine remove watch
    #[test]
    fn test_debugger_engine_remove_watch() {
        let mut engine = DebuggerEngine::new_default();

        let result = engine.add_watch("counter");
        let watch_id = result.data.as_ref().unwrap().id.clone();

        let remove_result = engine.remove_watch(&watch_id);
        assert!(remove_result.success, "Should remove watch through engine");

        let watches = engine.get_all_watches();
        assert_eq!(watches.len(), 0);
    }

    /// Test 13: DebuggerEngine get watch count
    #[test]
    fn test_debugger_engine_watch_count() {
        let mut engine = DebuggerEngine::new_default();

        assert_eq!(engine.get_watch_count(), 0);

        engine.add_watch("a");
        engine.add_watch("b");

        assert_eq!(engine.get_watch_count(), 2);
    }

    /// Test 14: DebuggerEngine clear all watches
    #[test]
    fn test_debugger_engine_clear_watches() {
        let mut engine = DebuggerEngine::new_default();

        engine.add_watch("x");
        engine.add_watch("y");
        engine.add_watch("z");

        engine.clear_all_watches();

        assert_eq!(engine.get_watch_count(), 0);
    }

    // =========================================
    // Watch Expression Evaluation Tests (Stubs)
    // =========================================

    /// Test 15: Watch expression structure
    #[test]
    fn test_watch_expression_structure() {
        use beejs::debugger::watch::WatchExpression;

        let watch = WatchExpression::new("x + y");

        assert_eq!(watch.expression, "x + y");
        assert!(watch.last_value.is_none());
        assert!(watch.value_type.is_none());
        assert!(!watch.has_error);
        assert!(watch.error_message.is_none());
    }

    /// Test 16: Watch expression formatting
    #[test]
    fn test_watch_expression_formatting() {
        use beejs::debugger::watch::WatchExpression;

        let mut watch = WatchExpression::new("counter");
        watch.last_value = Some("42".to_string());
        watch.value_type = Some("number".to_string());

        let formatted = watch.format();
        assert!(formatted.contains("counter"));
        assert!(formatted.contains("42"));
    }

    /// Test 17: Watch expression with complex value
    #[test]
    fn test_watch_expression_complex_value() {
        use beejs::debugger::watch::WatchExpression;

        let mut watch = WatchExpression::new("obj.property");
        watch.last_value = Some("{\"key\": \"value\"}".to_string());
        watch.value_type = Some("object".to_string());

        assert_eq!(watch.expression, "obj.property");
        assert_eq!(watch.value_type, Some("object".to_string()));
    }

    /// Test 18: Watch stats in debug stats
    #[test]
    fn test_watch_stats() {
        let mut engine = DebuggerEngine::new_default();

        engine.add_watch("x");
        engine.add_watch("y");

        let stats = engine.get_stats();
        assert_eq!(stats.watches_added, 2, "Should track watch additions");
    }
}
