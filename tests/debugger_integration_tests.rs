use std::time::{SystemTime, UNIX_EPOCH, Duration};
//! Debugger Module Integration Tests
//!
//! Tests for the beejs debugger functionality
//! Stage 58: Debugger Integration

use beejs::debugger::{
    DebuggerEngine, BreakpointManager, Breakpoint, StackFrame, StackTrace,
    VariableInspector, DebugConfig, DebugState, DebugEvent,
    StepType, SourceLocation, SimpleEventListener,
    breakpoint::BreakpointCondition,
    engine::DebugEventListener
};

#[cfg(test)]
mod tests {
    use super::*;

    /// Test 1: BreakpointManager creation
    #[test]
    fn test_breakpoint_manager_creation() {
        let manager = BreakpointManager::new();
        assert_eq!(manager.count(), 0, "Should start with no breakpoints");
    }

    /// Test 2: Add simple breakpoint
    #[test]
    fn test_add_simple_breakpoint() {
        let mut manager = BreakpointManager::new();

        let result = manager.add(
            "test.js".to_string(),
            "test.js".to_string(),
            10,
            0,
        );

        assert!(result.success, "Should add breakpoint successfully");
        assert_eq!(manager.count(), 1, "Should have 1 breakpoint");

        if let Some(breakpoint) = result.data {
            assert_eq!(breakpoint.script_id, "test.js");
            assert_eq!(breakpoint.line_number, 10);
            assert_eq!(breakpoint.column_number, 0);
            assert!(breakpoint.enabled, "Should be enabled by default");
        }
    }

    /// Test 3: Add conditional breakpoint
    #[test]
    fn test_add_conditional_breakpoint() {
        let mut manager = BreakpointManager::new();

        let condition = BreakpointCondition::Expression("x > 10".to_string());
        let result = manager.add_conditional(
            "test.js".to_string(),
            "test.js".to_string(),
            20,
            5,
            condition.clone(),
        );

        assert!(result.success, "Should add conditional breakpoint");
        assert_eq!(manager.count(), 1, "Should have 1 breakpoint");

        if let Some(breakpoint) = result.data {
            assert_eq!(breakpoint.condition, condition);
        }
    }

    /// Test 4: Remove breakpoint
    #[test]
    fn test_remove_breakpoint() {
        let mut manager = BreakpointManager::new();

        let result = manager.add(
            "test fn test_remove_break.js".to_string(),
            "test.js".to_string(),
            10,
            0,
        );

        assert!(result.success, "Should add breakpoint");
        let breakpoint_id = result.data.as_ref().unwrap().id.clone();

        let remove_result = manager.remove_breakpoint(&breakpoint_id);
        assert!(remove_result.success, "Should remove breakpoint successfully");
        assert_eq!(manager.count(), 0, "Should have no breakpoints");
    }

    /// Test 5: Enable and disable breakpoint
    #[test]
    fn test_enable_disable_breakpoint() {
        let mut manager = BreakpointManager::new();

        let result = manager.add(
            "test.js".to_string(),
            "test.js".to_string(),
            10,
            0,
        );
        let breakpoint_id = result.data.as_ref().unwrap().id.clone();

        // Disable breakpoint
        let disable_result = manager.disable_breakpoint(&breakpoint_id);
        assert!(disable_result.success, "Should disable breakpoint");

        let bp = manager.get_breakpoint(&breakpoint_id).unwrap();
        assert!(!bp.enabled, "Should be disabled");

        // Enable breakpoint
        let enable_result = manager.enable_breakpoint(&breakpoint_id);
        assert!(enable_result.success, "Should enable breakpoint");

        let bp = manager.get_breakpoint(&breakpoint_id).unwrap();
        assert!(bp.enabled, "Should be enabled");
    }

    /// Test 6: Find breakpoints by location
    #[test]
    fn test_find_breakpoints_by_location() {
        let mut manager = BreakpointManager::new();

        manager.add("test.js".to_string(), "test.js".to_string(), 10, 0).unwrap();
        manager.add("test.js".to_string(), "test.js".to_string(), 20, 0).unwrap();
        manager.add("other.js".to_string(), "other.js".to_string(), 10, 0).unwrap();

        let found = manager.find_breakpoints("test.js", 10);
        assert_eq!(found.len(), 1, "Should find 1 breakpoint at test.js:10");

        let found = manager.find_script_breakpoints("test.js");
        assert_eq!(found.len(), 2, "Should find 2 breakpoints in test.js");
    }

    /// Test 7: Increment hit count
    #[test]
    fn test_increment_hit_count() {
        let mut manager = BreakpointManager::new();

        let result = manager.add("test.js".to_string(), "test.js".to_string(), 10, 0).unwrap();
        let breakpoint_id = result.id.clone();

        let increment_result = manager.increment_hit_count(&breakpoint_id);
        assert!(increment_result.success, "Should increment hit count");

        let bp = manager.get_breakpoint(&breakpoint_id).unwrap();
        assert_eq!(bp.hit_count, 1, "Hit count should be 1");
    }

    /// Test 8: DebuggerEngine creation
    #[test]
    fn test_debugger_engine_creation() {
        let config = DebugConfig::default();
        let engine = DebuggerEngine::new(config);

        assert_eq!(engine.get_state(), DebugState::Running, "Should start in Running state");
    }

    /// Test 9: DebuggerEngine with default config
    #[test]
    fn test_debugger_engine_default() {
        let engine = DebuggerEngine::new_default();
        assert_eq!(engine.get_state(), DebugState::Running);
    }

    /// Test 10: Set breakpoint in debugger engine
    #[test]
    fn test_debugger_set_breakpoint() {
        let mut engine = DebuggerEngine::new_default();

        let result = engine.set_breakpoint(
            "test.js".to_string(),
            "test.js".to_string(),
            10,
        );

        assert!(result.success, "Should set breakpoint successfully");

        let breakpoints = engine.get_all_breakpoints();
        assert_eq!(breakpoints.len(), 1, "Should have 1 breakpoint");

        let bp = &breakpoints[0];
        assert_eq!(bp.line_number, 10);
        assert_eq!(bp.script_name, "test.js");
    }

    /// Test 11: Set conditional breakpoint
    #[test]
    fn test_debugger_set_conditional_breakpoint() {
        let mut engine = DebuggerEngine::new_default();

        let condition = BreakpointCondition::HitCount(5);
        let result = engine.set_conditional_breakpoint(
            "test.js".to_string(),
            "test.js".to_string(),
            20,
            condition.clone(),
        );

        assert!(result.success, "Should set conditional breakpoint");

        let breakpoints = engine.get_all_breakpoints();
        assert_eq!(breakpoints.len(), 1, "Should have 1 breakpoint");

        let bp = &breakpoints[0];
        assert_eq!(bp.condition, condition);
    }

    /// Test 12: Remove breakpoint
    #[test]
    fn test_debugger_remove_breakpoint() {
        let mut engine = DebuggerEngine::new_default();

        let result = engine.set_breakpoint(
            "test.js".to_string(),
            "test.js".to_string(),
            10,
        );
        let breakpoint_id = result.data.as_ref().unwrap().id.clone();

        let remove_result = engine.remove_breakpoint(&breakpoint_id);
        assert!(remove_result.success, "Should remove breakpoint");

        let breakpoints = engine.get_all_breakpoints();
        assert_eq!(breakpoints.len(), 0, "Should have no breakpoints");
    }

    /// Test 13: Enable and disable breakpoint
    #[test]
    fn test_debugger_enable_disable_breakpoint() {
        let mut engine = DebuggerEngine::new_default();

        let result = engine.set_breakpoint(
            "test.js".to_string(),
            "test.js".to_string(),
            10,
        );
        let breakpoint_id = result.data.as_ref().unwrap().id.clone();

        let disable_result = engine.disable_breakpoint(&breakpoint_id);
        assert!(disable_result.success, "Should disable breakpoint");

        let enabled_bps = engine.get_enabled_breakpoints();
        assert_eq!(enabled_bps.len(), 0, "Should have no enabled breakpoints");

        let enable_result = engine.enable_breakpoint(&breakpoint_id);
        assert!(enable_result.success, "Should enable breakpoint");

        let enabled_bps = engine.get_enabled_breakpoints();
        assert_eq!(enabled_bps.len(), 1, "Should have 1 enabled breakpoint");
    }

    /// Test 14: Continue execution
    #[test]
    fn test_continue_execution() {
        let engine = DebuggerEngine::new_default();

        // Start in Running state
        assert_eq!(engine.get_state(), DebugState::Running);

        // Pause first
        let _ = engine.pause();
        assert_eq!(engine.get_state(), DebugState::Paused);

        // Continue
        let result = engine.continue_execution();
        assert!(result.success, "Should continue successfully");
        assert_eq!(engine.get_state(), DebugState::Running);
    }

    /// Test 15: Step over
    #[test]
    fn test_step_over() {
        let mut engine = DebuggerEngine::new_default();

        let result = engine.step_over();
        assert!(result.success, "Should step over successfully");
        assert_eq!(engine.get_state(), DebugState::Stepping);
    }

    /// Test 16: Step into
    #[test]
    fn test_step_into() {
        let mut engine = DebuggerEngine::new_default();

        let result = engine.step_into();
        assert!(result.success, "Should step into successfully");
        assert_eq!(engine.get_state(), DebugState::Stepping);
    }

    /// Test 17: Step out
    #[test]
    fn test_step_out() {
        let mut engine = DebuggerEngine::new_default();

        let result = engine.step_out();
        assert!(result.success, "Should step out successfully");
        assert_eq!(engine.get_state(), DebugState::Stepping);
    }

    /// Test 18: Next (step to next statement)
    #[test]
    fn test_next() {
        let mut engine = DebuggerEngine::new_default();

        let result = engine.next();
        assert!(result.success, "Should step next successfully");
        assert_eq!(engine.get_state(), DebugState::Stepping);
    }

    /// Test 19: Pause execution
    #[test]
    fn test_pause() {
        let engine = DebuggerEngine::new_default();

        let result = engine.pause();
        assert!(result.success, "Should pause successfully");
        assert_eq!(engine.get_state(), DebugState::Paused);
    }

    /// Test 20: Terminate debugging
    #[test]
    fn test_terminate() {
        let engine = DebuggerEngine::new_default();

        let result = engine.terminate();
        assert!(result.success, "Should terminate successfully");
        assert_eq!(engine.get_state(), DebugState::Terminated);
    }

    /// Test 21: Get stack trace
    #[test]
    fn test_get_stack_trace() {
        let engine = DebuggerEngine::new_default();

        // Initially no stack trace
        let stack = engine.get_stack_trace();
        assert!(stack.is_none(), "Should have no stack trace initially");
    }

    /// Test 22: Update stack trace
    #[test]
    fn test_update_stack_trace() {
        let engine = DebuggerEngine::new_default();

        let mut stack_trace = StackTrace::new();
        let frame = StackFrame {
            index: 0,
            script_id: "test.js".to_string(),
            script_name: "test.js".to_string(),
            function_name: "main".to_string(),
            line_number: 10,
            column_number: 0,
            is_eval: false,
            is_constructor: false,
            is_async: false,
        };
        stack_trace.frames.push(frame);

        engine.update_stack_trace(stack_trace.clone());

        let retrieved = engine.get_stack_trace();
        assert!(retrieved.is_some(), "Should have stack trace");
        assert_eq!(retrieved.as_ref().unwrap().frames.len(), 1);
    }

    /// Test 23: Get all breakpoints
    #[test]
    fn test_get_all_breakpoints() {
        let mut engine = DebuggerEngine::new_default();

        engine.set_breakpoint("test1.js".to_string(), "test1.js".to_string(), 10).unwrap();
        engine.set_breakpoint("test2.js".to_string(), "test2.js".to_string(), 20).unwrap();

        let all = engine.get_all_breakpoints();
        assert_eq!(all.len(), 2, "Should have 2 breakpoints");
    }

    /// Test 24: Get enabled breakpoints
    #[test]
    fn test_get_enabled_breakpoints() {
        let mut engine = DebuggerEngine::new_default();

        let result1 = engine.set_breakpoint("test1.js".to_string(), "test1.js".to_string(), 10).unwrap();
        let result2 = engine.set_breakpoint("test2.js".to_string(), "test2.js".to_string(), 20).unwrap();

        // Disable one breakpoint
        engine.disable_breakpoint(&result1.id).unwrap();

        let enabled = engine.get_enabled_breakpoints();
        assert_eq!(enabled.len(), 1, "Should have 1 enabled breakpoint");
    }

    /// Test 25: Get debug stats
    #[test]
    fn test_get_debug_stats() {
        let engine = DebuggerEngine::new_default();

        let stats = engine.get_stats();
        assert_eq!(stats.breakpoints_set, 0);
        assert_eq!(stats.breakpoints_hit, 0);
        assert_eq!(stats.steps_executed, 0);
        assert!(stats.start_time.elapsed().unwrap().as_secs() >= 0);
    }

    /// Test 26: Should pause at location
    #[test]
    fn test_should_pause_at_location() {
        let mut engine = DebuggerEngine::new_default();

        // Set a breakpoint
        let _ = engine.set_breakpoint(
            "test.js".to_string(),
            "test.js".to_string(),
            10,
        );

        // Should pause at the breakpoint location
        let should_pause = engine.should_pause("test.js", 10);
        assert!(should_pause, "Should pause at breakpoint location");
    }

    /// Test 27: Is running check
    #[test]
    fn test_is_running() {
        let engine = DebuggerEngine::new_default();

        assert!(engine.is_running(), "Should be running initially");

        let _ = engine.pause();
        assert!(!engine.is_running(), "Should not be running after pause");
    }

    /// Test 28: Is paused check
    #[test]
    fn test_is_paused() {
        let engine = DebuggerEngine::new_default();

        assert!(!engine.is_paused(), "Should not be paused initially");

        let _ = engine.pause();
        assert!(engine.is_paused(), "Should be paused after pause()");
    }

    /// Test 29: Is stepping check
    #[test]
    fn test_is_stepping() {
        let mut engine = DebuggerEngine::new_default();

        assert!(!engine.is_stepping(), "Should not be stepping initially");

        let _ = engine.step_over();
        assert!(engine.is_stepping(), "Should be stepping after step_over()");
    }

    /// Test 30: SimpleEventListener
    #[test]
    fn test_simple_event_listener() {
        let listener = SimpleEventListener::new();

        assert_eq!(listener.get_events().len(), 0, "Should start with no events");

        let event = DebugEvent::ProgramStarted {
            script_id: "test.js".to_string(),
        };

        listener.on_event(&event);

        let events = listener.get_events();
        assert_eq!(events.len(), 1, "Should have 1 event");
        assert!(matches!(events[0], DebugEvent::ProgramStarted { .. }));
    }

    /// Test 31: StackTrace creation
    #[test]
    fn test_stack_trace_creation() {
        let stack = StackTrace::new();
        assert_eq!(stack.frames.len(), 0, "Should start with empty frames");
    }

    /// Test 32: StackFrame structure
    #[test]
    fn test_stack_frame_structure() {
        let frame = StackFrame {
            index: 0,
            script_id: "test.js".to_string(),
            script_name: "test.js".to_string(),
            function_name: "func".to_string(),
            line_number: 10,
            column_number: 5,
            is_eval: false,
            is_constructor: false,
            is_async: false,
        };

        assert_eq!(frame.script_name, "test.js");
        assert_eq!(frame.function_name, "func");
        assert_eq!(frame.line_number, 10);
        assert_eq!(frame.column_number, 5);
        assert!(!frame.is_eval);
        assert!(!frame.is_constructor);
    }

    /// Test 33: SourceLocation structure
    #[test]
    fn test_source_location_structure() {
        let location = SourceLocation {
            script_id: "script_id".to_string(),
            script_name: "script_name".to_string(),
            line_number: 42,
            column_number: 10,
        };

        assert_eq!(location.script_id, "script_id");
        assert_eq!(location.script_name, "script_name");
        assert_eq!(location.line_number, 42);
        assert_eq!(location.column_number, 10);
    }

    /// Test 34: Breakpoint matches location
    #[test]
    fn test_breakpoint_matches_location() {
        let breakpoint = Breakpoint {
            id: "bp1".to_string(),
            script_id: "test.js".to_string(),
            script_name: "test.js".to_string(),
            line_number: 10,
            column_number: 0,
            enabled: true,
            condition: BreakpointCondition::None,
            hit_count: 0,
            created_at: std::time::SystemTime::now(),
        };

        assert!(breakpoint.matches_location("test.js", 10));
        assert!(!breakpoint.matches_location("test.js", 20));
        assert!(!breakpoint.matches_location("other.js", 10));
    }

    /// Test 35: Breakpoint should trigger
    #[test]
    fn test_breakpoint_should_trigger() {
        let mut breakpoint = Breakpoint {
            id: "bp1".to_string(),
            script_id: "test.js".to_string(),
            script_name: "test.js".to_string(),
            line_number: 10,
            column_number: 0,
            enabled: true,
            condition: BreakpointCondition::None,
            hit_count: 0,
            created_at: std::time::SystemTime::now(),
        };

        assert!(breakpoint.should_trigger(), "Should trigger when enabled and no condition");

        // Disable breakpoint
        breakpoint.enabled = false;
        assert!(!breakpoint.should_trigger(), "Should not trigger when disabled");

        // Re-enable
        breakpoint.enabled = true;

        // Test hit count condition
        breakpoint.condition = BreakpointCondition::HitCount(3);
        assert!(!breakpoint.should_trigger(), "Should not trigger before hit count reached");

        breakpoint.hit_count = 3;
        assert!(breakpoint.should_trigger(), "Should trigger when hit count reached");
    }

    /// Test 36: Multiple breakpoints in same location
    #[test]
    fn test_multiple_breakpoints_same_location() {
        let mut manager = BreakpointManager::new();

        manager.add("test.js".to_string(), "test.js".to_string(), 10, 0).unwrap();
        manager.add("test.js".to_string(), "test.js".to_string(), 10, 0).unwrap();

        let found = manager.find_breakpoints("test.js", 10);
        assert_eq!(found.len(), 2, "Should find 2 breakpoints at same location");
    }

    /// Test 37: Clear all breakpoints
    #[test]
    fn test_clear_all_breakpoints() {
        let mut manager = BreakpointManager::new();

        manager.add("test.js".to_string(), "test.js".to_string(), 10, 0).unwrap();
        manager.add("test.js".to_string(), "test.js".to_string(), 20, 0).unwrap();

        assert_eq!(manager.count(), 2, "Should have 2 breakpoints");

        manager.clear_all();
        assert_eq!(manager.count(), 0, "Should have no breakpoints after clear");
    }

    /// Test 38: Breakpoint count methods
    #[test]
    fn test_breakpoint_count_methods() {
        let mut manager = BreakpointManager::new();

        assert_eq!(manager.count(), 0);
        assert_eq!(manager.enabled_count(), 0);

        manager.add("test.js".to_string(), "test.js".to_string(), 10, 0).unwrap();
        manager.add("test.js".to_string(), "test.js".to_string(), 20, 0).unwrap();

        assert_eq!(manager.count(), 2);
        assert_eq!(manager.enabled_count(), 2);

        // Disable one
        let result = manager.add("test.js".to_string(), "test.js".to_string(), 30, 0).unwrap();
        manager.disable_breakpoint(&result.id).unwrap();

        assert_eq!(manager.count(), 3);
        assert_eq!(manager.enabled_count(), 2);
    }

    /// Test 39: Get breakpoint by ID
    #[test]
    fn test_get_breakpoint_by_id() {
        let mut manager = BreakpointManager::new();

        let result = manager.add("test.js".to_string(), "test.js".to_string(), 10, 0).unwrap();
        let breakpoint_id = result.id.clone();

        let bp = manager.get_breakpoint(&breakpoint_id);
        assert!(bp.is_some(), "Should find breakpoint by ID");

        let bp = bp.unwrap();
        assert_eq!(bp.line_number, 10);

        // Non-existent ID
        let bp = manager.get_breakpoint("nonexistent");
        assert!(bp.is_none(), "Should not find non-existent breakpoint");
    }

    /// Test 40: DebuggerEngine state transitions
    #[test]
    fn test_debugger_state_transitions() {
        let mut engine = DebuggerEngine::new_default();

        // Initial state
        assert_eq!(engine.get_state(), DebugState::Running);

        // Pause
        let _ = engine.pause();
        assert_eq!(engine.get_state(), DebugState::Paused);

        // Step
        let _ = engine.step_over();
        assert_eq!(engine.get_state(), DebugState::Stepping);

        // Continue
        let _ = engine.continue_execution();
        assert_eq!(engine.get_state(), DebugState::Running);

        // Terminate
        let _ = engine.terminate();
        assert_eq!(engine.get_state(), DebugState::Terminated);
    }
}
