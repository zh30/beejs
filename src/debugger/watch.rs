//! Watch Variables Module
//!
//! Provides functionality for monitoring variable values during debugging sessions.
//! Watch expressions are evaluated each time execution pauses, allowing developers
//! to track how values change over time.
use uuid::Uuid;
/// A watch expression being monitored
#[derive(Debug, Clone)]
pub struct WatchExpression {
    /// Unique identifier for this watch
    pub id: String,
    /// The expression being watched (e.g., "x + y", "obj.property")
    pub expression: String,
    /// Last evaluated value (None if never evaluated)
    pub last_value: Option<String>,
    /// Type of the last evaluated value
    pub value_type: Option<String>,
    /// Whether the last evaluation resulted in an error
    pub has_error: bool,
    /// Error message if has_error is true
    pub error_message: Option<String>,
    /// Number of times this watch was evaluated
    pub evaluation_count: u64,
}
impl WatchExpression {
    /// Create a new watch expression
    pub fn new(expression: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            expression: expression.to_string(),
            last_value: None,
            value_type: None,
            has_error: false,
            error_message: None,
            evaluation_count: 0,
        }
    }
    /// Format the watch for display
    pub fn format(&self) -> String {
        if self.has_error {
            format!(
                "{}: <error: {}>",
                self.expression,
                self.error_message.as_ref().unwrap_or(&"Unknown error".to_string())
            )
        } else if let Some(ref value) = self.last_value {
            let type_str: _ = self.value_type.as_ref()
                .map(|t| format!(" ({})", t))
                .unwrap_or_default();
            format!("{}: {}{}", self.expression, value, type_str)
        } else {
            format!("{}: <not evaluated>", self.expression)
        }
    }
    /// Clear the evaluated value and error state
    pub fn reset(&mut self) {
        self.last_value = None;
        self.value_type = None;
        self.has_error = false;
        self.error_message = None;
    }
}
/// Manages a collection of watch expressions
#[derive(Debug)]
pub struct WatchManager {
    watches: HashMap<String, WatchExpression>,
    /// Order in which watches were added (for consistent display)
    order: Vec<String>,
}
impl WatchManager {
    /// Create a new empty watch manager
    pub fn new() -> Self {
        Self {
            watches: HashMap::new(),
            order: Vec::new(),
        }
    }
    /// Add a new watch expression
    pub fn add(&mut self, expression: &str) -> Result<WatchExpression, String> {
        let watch: _ = WatchExpression::new(expression);
        let id: _ = watch.id.clone();
        self.order.push(id.clone());
        self.watches.insert(id, watch.clone());
        Ok(watch)
    }
    /// Remove a watch expression by ID
    pub fn remove(&mut self, id: &str) -> Result<(), String> {
        if self.watches.remove(id).is_some() {
            self.order.retain(|i| i != id);
            Ok(())
        } else {
            Err(format!("Watch with ID '{}' not found", id))
        }
    }
    /// Get a watch expression by ID
    pub fn get(&self, id: &str) -> Option<&WatchExpression> {
        self.watches.get(id)
    }
    /// Get a mutable reference to a watch expression by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut WatchExpression> {
        self.watches.get_mut(id)
    }
    /// List all watch expressions in order
    pub fn list(&self) -> Vec<&WatchExpression> {
        self.order
            .iter()
            .filter_map(|id| self.watches.get(id))
            .collect()
    }
    /// Get the number of watch expressions
    pub fn count(&self) -> usize {
        self.watches.len()
    }
    /// Update the value of a watch expression
    pub fn update_value(
        &mut self,
        id: &str,
        value: &str,
        value_type: &str,
    ) -> Result<(), String> {
        if let Some(watch) = self.watches.get_mut(id) {
            watch.last_value = Some(value.to_string());
            watch.value_type = Some(value_type.to_string());
            watch.has_error = false;
            watch.error_message = None;
            watch.evaluation_count += 1;
            Ok(())
        } else {
            Err(format!("Watch with ID '{}' not found", id))
        }
    }
    /// Set an error on a watch expression
    pub fn set_error(&mut self, id: &str, error: &str) -> Result<(), String> {
        if let Some(watch) = self.watches.get_mut(id) {
            watch.has_error = true;
            watch.error_message = Some(error.to_string());
            watch.last_value = None;
            watch.evaluation_count += 1;
            Ok(())
        } else {
            Err(format!("Watch with ID '{}' not found", id))
        }
    }
    /// Clear all watch expressions
    pub fn clear(&mut self) {
        self.watches.clear();
        self.order.clear();
    }
    /// Reset all watch values (but keep the expressions)
    pub fn reset_all(&mut self) {
        for watch in self.watches.values_mut() {
            watch.reset();
        }
    }
}
impl Default for WatchManager {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod unit_tests {
    use super::*;
    #[test]
    fn test_watch_expression_creation() {
        let watch: _ = WatchExpression::new("x + 1");
        assert_eq!(watch.expression, "x + 1");
        assert!(!watch.id.is_empty());
        assert!(watch.last_value.is_none());
    }
    #[test]
    fn test_watch_manager_basic_operations() {
        let mut manager = WatchManager::new();
        // Add
        let watch: _ = manager.add("counter").unwrap();
        assert_eq!(manager.count(), 1);
        // Get
        let retrieved: _ = manager.get(&watch.id).unwrap();
        assert_eq!(retrieved.expression, "counter");
        // Remove
        manager.remove(&watch.id).unwrap();
        assert_eq!(manager.count(), 0);
    }
    #[test]
    fn test_watch_ordering() {
        let mut manager = WatchManager::new();
        manager.add("first").unwrap();
        manager.add("second").unwrap();
        manager.add("third").unwrap();
        let list: _ = manager.list();
        assert_eq!(list[0].expression, "first");
        assert_eq!(list[1].expression, "second");
        assert_eq!(list[2].expression, "third");
    }
}
use std::collections::{BTreeMap, HashMap};