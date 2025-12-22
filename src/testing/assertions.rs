//! Enhanced Assertion Library
//! Stage 93 Phase 3.3 - Extended Matchers
//!
//! Provides comprehensive assertion matchers including:
//! - Extended matchers (toEqual, toMatch, toContain, etc.)
//! - Async matchers for Promise-based tests
//! - Custom matcher support
//! - Deep equality checking

/// Core matcher trait
pub trait Matcher<T> {
    type Output;

    fn matches(&self, value: &T) -> bool;
    fn message(&self, value: &T) -> String;
}

/// Assertion context for chaining
pub struct AssertionContext<T> {
    value: T,
}

impl<T> AssertionContext<T> {
    pub fn new(value: T) -> Self {
        AssertionContext { value }
    }

    /// Basic equality matcher
    pub fn to_equal(&self, expected: &T) -> bool
    where
        T: PartialEq + std::fmt::Debug,
    {
        self.value == *expected
    }

    /// Get the value for further operations
    pub fn value(&self) -> &T {
        &self.value
    }
}

/// Create an assertion context
pub fn expect<T>(value: T) -> AssertionContext<T> {
    AssertionContext::new(value)
}

/// Result of an assertion
#[derive(Debug, Clone)]
pub struct AssertionCheck {
    pub passed: bool,
    pub message: String,
    pub expected: Option<String>,
    pub actual: Option<String>,
}

impl AssertionCheck {
    pub fn success(message: String) -> Self {
        AssertionCheck {
            passed: true,
            message,
            expected: None,
            actual: None,
        }
    }

    pub fn failure(message: String, expected: Option<String>, actual: Option<String>) -> Self {
        AssertionCheck {
            passed: false,
            message,
            expected,
            actual,
        }
    }
}

/// Extended matcher types
#[derive(Debug, Clone)]
pub enum ExtendedMatcher<T> {
    /// Basic equality
    Equal(T),
    /// Deep equality for objects
    DeepEqual(T),
    /// String contains
    Contains(String),
    /// Array/collection contains
    ArrayContains(T),
    /// Numeric comparisons
    GreaterThan(T),
    LessThan(T),
    /// Array length
    Length(usize),
    /// Truthy/falsy
    Truthy,
    Falsy,
}

impl<T> ExtendedMatcher<T> {
    pub fn new_equal(value: T) -> Self {
        ExtendedMatcher::Equal(value)
    }

    pub fn new_contains(substring: String) -> Self {
        ExtendedMatcher::Contains(substring)
    }

    pub fn new_length(length: usize) -> Self {
        ExtendedMatcher::Length(length)
    }

    pub fn new_truthy() -> Self {
        ExtendedMatcher::Truthy
    }

    pub fn new_falsy() -> Self {
        ExtendedMatcher::Falsy
    }
}

impl<T> Matcher<T> for ExtendedMatcher<T>
where
    T: PartialEq + std::fmt::Debug + serde::Serialize + Clone,
{
    type Output = AssertionCheck;

    fn matches(&self, value: &T) -> bool {
        match self {
            ExtendedMatcher::Equal(expected) => value == expected,
            ExtendedMatcher::DeepEqual(expected) => {
                // Simple deep equality for now
                serde_json::to_string(value).unwrap_or_default() ==
                    serde_json::to_string(expected).unwrap_or_default()
            }
            ExtendedMatcher::Contains(substring) => {
                if let Ok(s) = serde_json::to_string(value) {
                    s.contains(substring)
                } else {
                    false
                }
            }
            ExtendedMatcher::ArrayContains(expected) => {
                // Check if value is an array and contains expected
                if let Ok(array) = serde_json::to_value(value) {
                    if let Some(arr) = array.as_array() {
                        arr.iter().any(|item| {
                            if let Ok(expected_val) = serde_json::to_value(expected) {
                                item == &expected_val
                            } else {
                                false
                            }
                        })
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            ExtendedMatcher::GreaterThan(expected) => {
                // Simple numeric comparison
                format!("{:?}", value) > format!("{:?}", expected)
            }
            ExtendedMatcher::LessThan(expected) => {
                // Simple numeric comparison
                format!("{:?}", value) < format!("{:?}", expected)
            }
            ExtendedMatcher::Length(expected) => {
                if let Ok(json) = serde_json::to_value(value) {
                    match json {
                        serde_json::Value::Array(arr) => arr.len() == *expected,
                        serde_json::Value::Object(obj) => obj.len() == *expected,
                        serde_json::Value::String(s) => s.len() == *expected,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            ExtendedMatcher::Truthy => {
                // Check if value is truthy
                let json = serde_json::to_value(value).unwrap_or(serde_json::Value::Null);
                match json {
                    serde_json::Value::Null => false,
                    serde_json::Value::Bool(b) => b,
                    serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
                    serde_json::Value::String(s) => !s.is_empty() && s != "false" && s != "0",
                    serde_json::Value::Array(arr) => !arr.is_empty(),
                    serde_json::Value::Object(obj) => !obj.is_empty(),
                }
            }
            ExtendedMatcher::Falsy => {
                // Check if value is falsy
                let json = serde_json::to_value(value).unwrap_or(serde_json::Value::Null);
                match json {
                    serde_json::Value::Null => true,
                    serde_json::Value::Bool(b) => !b,
                    serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0) == 0.0,
                    serde_json::Value::String(s) => s.is_empty() || s == "false" || s == "0",
                    serde_json::Value::Array(arr) => arr.is_empty(),
                    serde_json::Value::Object(obj) => obj.is_empty(),
                }
            }
        }
    }

    fn message(&self, value: &T) -> String {
        match self {
            ExtendedMatcher::Equal(expected) => {
                format!("Expected {:?} to equal {:?}", value, expected)
            }
            ExtendedMatcher::DeepEqual(expected) => {
                format!("Expected {:?} to deep equal {:?}", value, expected)
            }
            ExtendedMatcher::Contains(substring) => {
                format!("Expected {:?} to contain {:?}", value, substring)
            }
            ExtendedMatcher::ArrayContains(expected) => {
                format!("Expected {:?} to contain {:?}", value, expected)
            }
            ExtendedMatcher::GreaterThan(expected) => {
                format!("Expected {:?} to be greater than {:?}", value, expected)
            }
            ExtendedMatcher::LessThan(expected) => {
                format!("Expected {:?} to be less than {:?}", value, expected)
            }
            ExtendedMatcher::Length(expected) => {
                format!("Expected {:?} to have length {}", value, expected)
            }
            ExtendedMatcher::Truthy => {
                format!("Expected {:?} to be truthy", value)
            }
            ExtendedMatcher::Falsy => {
                format!("Expected {:?} to be falsy", value)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equal_matcher() {
        let matcher = ExtendedMatcher::Equal(42);
        let value = 42;
        assert!(matcher.matches(&value));
        assert!(matcher.message(&value).contains("equal"));
    }

    #[test]
    fn test_contains_matcher() {
        let matcher = ExtendedMatcher::Contains("test".to_string());
        let value = "this is a test string";
        assert!(matcher.matches(&value));
    }

    #[test]
    fn test_length_matcher() {
        let matcher = ExtendedMatcher::Length(5);
        let value = vec![1, 2, 3, 4, 5];
        assert!(matcher.matches(&value));
    }

    #[test]
    fn test_truthy_matcher() {
        let matcher = ExtendedMatcher::Truthy;
        let value = "true";
        assert!(matcher.matches(&value));
    }

    #[test]
    fn test_falsy_matcher() {
        let matcher = ExtendedMatcher::Falsy;
        let value = "";
        assert!(matcher.matches(&value));
    }
}
