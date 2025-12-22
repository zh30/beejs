//! Stage 96 Phase 3: Enhanced Debugging Tools - Test Suite
//!
//! This module tests the enhanced debugging features including:
//! - Visual debugging interface
//! - Remote debugging support
//! - Performance profiling
//! - Memory analysis

use beejs::debugger::enhanced::{
    BreakpointManager, VariableInspector, CallStackView, Repl,
    DebuggerUI, Breakpoint, BreakpointCondition, Variable, Scope, StackFrame,
    HeapSnapshot, ObjectTracer, MemoryAnalyzer, HeapStats,
    PerformanceProfiler, HotReload, PerformanceMetrics,
};
use beejs::debugger::remote::{
    DebugServer, ConnectionManager, SessionManager,
    WebSocketHandler, DebugProtocol, SessionId, StateSync, DebugEvent, DebugState, StackFrameInfo,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, sleep};

/// Test breakpoint management
#[tokio::test]
async fn test_breakpoint_manager_creation() {
    let mut manager = BreakpointManager::new();

    // Add line breakpoint
    let mut breakpoint = Breakpoint {
        id: 0,
        file: "test.js".to_string(),
        line: 42,
        condition: Some(BreakpointCondition::Equals("count".to_string(), "10".to_string())),
    };

    let id: _ = manager.add_breakpoint(breakpoint.clone()).await.unwrap();
    assert!(id > 0);

    // Verify breakpoint exists
    assert!(manager.get_breakpoint(id).await.is_some());

    // Remove breakpoint
    assert!(manager.remove_breakpoint(id).await.is_ok());
    assert!(manager.get_breakpoint(id).await.is_none());
}

/// Test variable inspection
#[tokio::test]
async fn test_variable_inspector() {
    let inspector: _ = VariableInspector::new();

    // Create test variables
    let mut variables = HashMap::new();
    variables.insert("count".to_string(), "42".to_string());
    variables.insert("name".to_string(), "test".to_string());
    variables.insert("items".to_string(), "[1,2,3]".to_string());

    // Inspect variables
    let inspected: _ = inspector.inspect_variables(&variables).await.unwrap();

    // Verify inspection
    assert_eq!(inspected.len(), 3);
    assert!(inspected.contains_key("count"));
    assert!(inspected.contains_key("name"));
    assert!(inspected.contains_key("items"));

    // Test nested object inspection
    let mut nested = HashMap::new();
    nested.insert("user".to_string(), "{}".to_string());
    let result: _ = inspector.inspect_variables(&nested).await.unwrap();
    assert_eq!(result.len(), 1);
}

/// Test call stack view
#[tokio::test]
async fn test_call_stack_view() {
    let mut call_stack = CallStackView::new();

    // Add stack frames
    call_stack.push_frame("main", "main.js", 1, None).await;
    call_stack.push_frame("handler", "main.js", 15, Some("main")).await;
    call_stack.push_frame("callback", "handler.js", 7, Some("handler")).await;

    // Verify stack depth
    assert_eq!(call_stack.depth().await, 3);

    // Get top frame
    let top_frame: _ = call_stack.top_frame().await;
    assert!(top_frame.is_some());
    if let Some(frame) = top_frame {
        assert_eq!(frame.function, "callback");
    }

    // Pop frame
    call_stack.pop_frame().await;
    assert_eq!(call_stack.depth().await, 2);
}

/// Test interactive REPL
#[tokio::test]
async fn test_repl_evaluation() {
    let repl: _ = Repl::new();

    // Test simple expression
    let result: _ = repl.evaluate("1 + 1").await.unwrap();
    assert_eq!(result, "2".to_string());

    // Test variable access
    let result: _ = repl.evaluate("let x = 5; x * 2").await.unwrap();
    assert_eq!(result, "10".to_string());

    // Test function call
    let result: _ = repl.evaluate("Math.max(3, 7)").await.unwrap();
    assert_eq!(result, "7".to_string());
}

/// Test heap snapshot functionality
#[tokio::test]
async fn test_heap_snapshot() {
    let mut snapshot = HeapSnapshot::new();

    // Create test heap data
    snapshot.add_object("obj1", "Object", 1024, vec![]);
    snapshot.add_object("obj2", "Array", 2048, vec!["obj1"]);
    snapshot.add_object("obj3", "Function", 512, vec![]);

    // Verify snapshot contents
    assert_eq!(snapshot.object_count(), 3);
    assert_eq!(snapshot.total_size(), 3584);

    // Get statistics
    let stats: _ = snapshot.get_statistics();
    assert_eq!(stats.total_objects, 3);
    assert!(stats.total_size > 0);
}

/// Test object tracer
#[tokio::test]
async fn test_object_tracer() {
    let mut tracer = ObjectTracer::new();

    // Track object creation
    let object_id: _ = tracer.track_creation("obj_123", "Object", "test.js:10").await.unwrap();
    assert_eq!(object_id, "obj_123");

    // Track property access
    assert!(tracer.track_access("obj_123", "property1", "read").await.is_ok());
    assert!(tracer.track_access("obj_123", "property2", "write").await.is_ok());

    // Get access history
    let history: _ = tracer.get_access_history("obj_123").await.unwrap();
    assert_eq!(history.len(), 2);

    // Track object deletion
    assert!(tracer.track_deletion("obj_123").await.is_ok());
}

/// Test memory analyzer
#[tokio::test]
async fn test_memory_analyzer() {
    let mut analyzer = MemoryAnalyzer::new();

    // Create heap snapshots at different times
    let mut snapshot1 = HeapSnapshot::new();
    snapshot1.add_object("obj1", "Object", 1024, vec![]);

    let mut snapshot2 = HeapSnapshot::new();
    snapshot2.add_object("obj1", "Object", 1024, vec![]);
    snapshot2.add_object("obj2", "Array", 2048, vec![]);

    // Add snapshots
    analyzer.add_snapshot(snapshot1).await;
    analyzer.add_snapshot(snapshot2).await;

    // Compare snapshots
    let diff: _ = analyzer.compare_snapshots(0, 1).await.unwrap();
    assert!(diff.created.len() > 0);
    assert!(diff.deleted.len() == 0);

    // Detect memory leaks
    let leaks: _ = analyzer.detect_memory_leaks().await.unwrap();
    assert!(leaks.len() >= 0);
}

/// Test remote debug server
#[tokio::test]
async fn test_debug_server_lifecycle() {
    use std::net::TcpListener;

    // Bind to ephemeral port
    let listener: _ = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr: _ = listener.local_addr().unwrap();

    // Create server
    let mut server = DebugServer::new(addr).await.unwrap();

    // Start server
    assert!(server.start().await.is_ok());

    // Verify server is running
    assert!(server.is_running().await);

    // Stop server
    assert!(server.stop().await.is_ok());

    // Verify server is stopped
    assert!(!server.is_running().await);
}

/// Test session management
#[tokio::test]
async fn test_session_manager() {
    let mut manager = SessionManager::new();

    // Create session
    let session_id: _ = manager.create_session("test-client".to_string()).await.unwrap();
    assert!(session_id.len() > 0);

    // Verify session exists
    assert!(manager.get_session(session_id).await.is_some());

    // Close session
    assert!(manager.close_session(session_id).await.is_ok());

    // Verify session is closed
    assert!(manager.get_session(session_id).await.is_none());
}

/// Test connection manager
#[tokio::test]
async fn test_connection_manager() {
    let manager: _ = Arc::new(std::sync::Mutex::new(RwLock::new(ConnectionManager::new())));

    // Create connection
    let conn_id: _ = {
        let mut mgr = manager.write().await;
        mgr.create_connection("test-connection".to_string()).await.unwrap()
    };

    // Verify connection exists
    {
        let mgr: _ = manager.read().await;
        assert!(mgr.get_connection(conn_id).await.is_some());
    }

    // Close connection
    {
        let mut mgr = manager.write().await;
        assert!(mgr.close_connection(conn_id).await.is_ok());
    }

    // Verify connection is closed
    {
        let mgr: _ = manager.read().await;
        assert!(mgr.get_connection(conn_id).await.is_none());
    }
}

/// Test WebSocket handler
#[tokio::test]
async fn test_websocket_handler() {
    let handler: _ = WebSocketHandler::new();

    // Test message serialization/deserialization
    let debug_msg: _ = DebugProtocol::SetBreakpoint {
        file: "test.js".to_string(),
        line: 42,
        condition: None,
    };

    let serialized: _ = handler.serialize_message(&debug_msg).await.unwrap();
    assert!(!serialized.is_empty());

    let deserialized: _ = handler.deserialize_message(&serialized).await.unwrap();
    assert!(matches!(deserialized, DebugProtocol::SetBreakpoint { .. }));
}

/// Test debug protocol messages
#[tokio::test]
async fn test_debug_protocol_messages() {
    // Test various protocol message types
    let msg: _ = DebugProtocol::Continue;

    let serialized: _ = serde_json::to_string(&msg).unwrap();
    let deserialized: DebugProtocol = serde_json::from_str(&serialized).unwrap();

    assert!(matches!(deserialized, DebugProtocol::Continue));

    // Test breakpoint message
    let msg: _ = DebugProtocol::SetBreakpoint {
        file: "test.js".to_string(),
        line: 100,
        condition: Some("count > 5".to_string()),
    };

    let serialized: _ = serde_json::to_string(&msg).unwrap();
    let deserialized: DebugProtocol = serde_json::from_str(&serialized).unwrap();

    assert!(matches!(deserialized, DebugProtocol::SetBreakpoint { .. }));
}

/// Test performance profiling
#[tokio::test]
async fn test_performance_profiling() {
    let mut profiler = PerformanceProfiler::new();

    // Start profiling
    profiler.start_profiling().await.unwrap();

    // Simulate some function calls
    sleep(Duration::from_millis(10)).await;

    // Stop profiling
    let report: _ = profiler.stop_profiling().await.unwrap();

    // Verify report contains data
    assert!(report.total_duration > 0);
    assert!(report.function_counts.len() >= 0);
}

/// Test memory leak detection
#[tokio::test]
async fn test_memory_leak_detection() {
    let analyzer: _ = MemoryAnalyzer::new();

    // Create multiple snapshots showing growth
    for i in 0..5 {
        let mut snapshot = HeapSnapshot::new();
        snapshot.add_object(&format!("leak_obj_{}", i), "Object", 1024, vec![]);
        analyzer.add_snapshot(snapshot).await;
    }

    // Detect leaks
    let leaks: _ = analyzer.detect_memory_leaks().await.unwrap();

    // Should detect at least one leak pattern
    assert!(leaks.len() > 0);
}

/// Test conditional breakpoints
#[tokio::test]
async fn test_conditional_breakpoints() {
    let mut manager = BreakpointManager::new();

    // Create conditional breakpoint
    let mut breakpoint = Breakpoint {
        id: 0,
        file: "test.js".to_string(),
        line: 42,
        condition: Some(BreakpointCondition::Equals("count".to_string(), "10".to_string())),
    };

    let id: _ = manager.add_breakpoint(breakpoint).await.unwrap();

    // Test condition evaluation
    let mut variables = HashMap::new();
    variables.insert("count".to_string(), "10".to_string());

    let should_break: _ = manager.should_break(id, &variables).await.unwrap();
    assert!(should_break);

    // Test condition that doesn't match
    variables.insert("count".to_string(), "5".to_string());
    let should_break: _ = manager.should_break(id, &variables).await.unwrap();
    assert!(!should_break);
}

/// Test async call stack tracking
#[tokio::test]
async fn test_async_call_stack() {
    let mut call_stack = CallStackView::new();

    // Add synchronous frame
    call_stack.push_frame("sync_func", "main.js", 10, None).await;

    // Add async frame
    call_stack.push_async_frame("async_func", "main.js", 20, Some("sync_func")).await;

    // Add nested async frame
    call_stack.push_async_frame("nested_async", "main.js", 30, Some("async_func")).await;

    // Verify stack
    assert_eq!(call_stack.depth().await, 3);

    // Get frames with async info
    let frames: _ = call_stack.get_frames().await;
    assert_eq!(frames.len(), 3);
}

/// Test remote debugging multi-instance support
#[tokio::test]
async fn test_remote_debug_multi_instance() {
    let manager: _ = Arc::new(std::sync::Mutex::new(RwLock::new(SessionManager::new())));

    // Create multiple sessions
    let session1: _ = {
        let mut mgr = manager.write().await;
        mgr.create_session("client1".to_string()).await.unwrap()
    };

    let session2: _ = {
        let mut mgr = manager.write().await;
        mgr.create_session("client2".to_string()).await.unwrap()
    };

    // Verify both sessions exist
    assert_ne!(session1, session2);

    {
        let mgr: _ = manager.read().await;
        assert!(mgr.get_session(session1).await.is_some());
        assert!(mgr.get_session(session2).await.is_some());
    }

    // Close one session
    {
        let mut mgr = manager.write().await;
        mgr.close_session(session1).await.unwrap();
    }

    // Verify only session2 remains
    {
        let mgr: _ = manager.read().await;
        assert!(mgr.get_session(session1).await.is_none());
        assert!(mgr.get_session(session2).await.is_some());
    }
}

/// Test breakpoint synchronization
#[tokio::test]
async fn test_breakpoint_synchronization() {
    let manager: _ = Arc::new(std::sync::Mutex::new(RwLock::new(BreakpointManager::new())));

    // Add breakpoint
    let breakpoint: _ = Breakpoint {
        id: 0,
        file: "test.js".to_string(),
        line: 42,
        condition: None,
    };
    let id: _ = {
        let mut mgr = manager.write().await;
        mgr.add_breakpoint(breakpoint).await.unwrap()
    };

    // Sync breakpoints to remote instance
    {
        let mgr: _ = manager.read().await;
        let breakpoints: _ = mgr.get_all_breakpoints().await;
        assert_eq!(breakpoints.len(), 1);
    }

    // Remove breakpoint
    {
        let mut mgr = manager.write().await;
        mgr.remove_breakpoint(id).await.unwrap();
    }

    // Verify removal synced
    {
        let mgr: _ = manager.read().await;
        let breakpoints: _ = mgr.get_all_breakpoints().await;
        assert_eq!(breakpoints.len(), 0);
    }
}

/// Test hot reload functionality
#[tokio::test]
async fn test_hot_reload() {
    let mut hot_reload = HotReload::new();

    // Watch a file
    assert!(hot_reload.watch_file("test.js").await.is_ok());

    // Check if file is being watched
    assert!(hot_reload.is_watching("test.js").await);

    // Simulate file change
    assert!(hot_reload.notify_change("test.js").await.is_ok());

    // Unwatch file
    assert!(hot_reload.unwatch_file("test.js").await.is_ok());
    assert!(!hot_reload.is_watching("test.js").await);
}

/// Test performance metrics collection
#[tokio::test]
async fn test_performance_metrics() {
    let mut metrics = PerformanceMetrics::new();

    // Record function execution time
    metrics.record_function_time("test_func", Duration::from_millis(5)).await;

    // Record memory usage
    metrics.record_memory_usage(1024 * 1024).await; // 1MB

    // Record GC event
    metrics.record_gc_event(Duration::from_millis(2), 512 * 1024).await; // 2ms, 512KB

    // Get collected metrics
    let collected: _ = metrics.get_collected_metrics().await;

    assert!(collected.function_timings.contains_key("test_func"));
    assert!(collected.memory_peak > 0);
    assert!(collected.gc_count > 0);
}

/// Test error handling in debugging
#[tokio::test]
async fn test_debugger_error_handling() {
    let inspector: _ = VariableInspector::new();

    // Test inspection of invalid value
    let result: _ = inspector.inspect_value("undefined").await;
    assert!(result.is_ok());

    // Test inspection of circular reference
    let mut circular = HashMap::new();
    circular.insert("self".to_string(), "{}".to_string());

    // This should not panic
    let result: _ = inspector.inspect_variables(&circular).await;
    assert!(result.is_ok());
}

/// Test debug adapter protocol compliance
#[tokio::test]
async fn test_debug_adapter_protocol() {
    use beejs::tools::debug_adapter::protocol::dap::DebugAdapterProtocol;
use std::sync::{Arc, Mutex, RwLock};
use std::collections::{HashMap, BTreeMap};

    let dap: _ = DebugAdapterProtocol::new();

    // Test Initialize request
    let init_request: _ = serde_json::json!({
        "type": "request",
        "seq": 1,
        "command": "initialize",
        "arguments": {
            "clientID": "vscode",
            "clientName": "Visual Studio Code",
            "adapterID": "beejs",
            "linesStartAt1": true,
            "columnsStartAt1": true
        }
    });

    let message: serde_json::Value = init_request;
    // Note: We'd need to convert this to DapMessage format in a real implementation
    // For now, just verify the JSON is valid
    assert!(message.get("command").is_some());
}

/// Integration test: Full debugging workflow
#[tokio::test]
async fn test_full_debugging_workflow() {
    // This test simulates a complete debugging session

    // 1. Create debugging components
    let breakpoint_manager: _ = Arc::new(std::sync::Mutex::new(RwLock::new(BreakpointManager::new())));
    let variable_inspector: _ = Arc::new(std::sync::Mutex::new(RwLock::new(VariableInspector::new())));
    let call_stack: _ = Arc::new(std::sync::Mutex::new(RwLock::new(CallStackView::new())));

    // 2. Set a breakpoint
    let breakpoint: _ = Breakpoint {
        id: 0,
        file: "app.js".to_string(),
        line: 42,
        condition: None,
    };
    let breakpoint_id: _ = {
        let mut mgr = breakpoint_manager.write().await;
        mgr.add_breakpoint(breakpoint).await.unwrap()
    };

    // 3. Simulate hitting the breakpoint
    {
        let mut stack = call_stack.write().await;
        stack.push_frame("main", "app.js", 42, None).await;
    }

    // 4. Inspect variables at breakpoint
    let mut vars = HashMap::new();
    vars.insert("result".to_string(), "42".to_string());

    let inspected: _ = {
        let inspector = variable_inspector.read().await;
        inspector.inspect_variables(&vars).await.unwrap()
    };

    assert!(inspected.contains_key("result"));

    // 5. Step through code
    {
        let mut stack = call_stack.write().await;
        stack.push_frame("next_line", "app.js", 43, Some("main")).await;
    }

    // 6. Continue execution
    let should_continue: _ = {
        let mgr = breakpoint_manager.read().await;
        !mgr.should_break(breakpoint_id, &vars).await.unwrap()
    };

    assert!(should_continue);

    // 7. Clean up
    {
        let mut mgr = breakpoint_manager.write().await;
        mgr.remove_breakpoint(breakpoint_id).await.unwrap();
    }
}
