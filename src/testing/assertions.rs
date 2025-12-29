// Enhanced Assertion Library
// Stage 93 Phase 3.3 - Extended Matchers
//
// Provides comprehensive assertion matchers including:
// - Extended matchers (toEqual, toMatch, toContain, etc.)
// - Async matchers for Promise-based tests
// - Custom matcher support
// - Deep equality checking
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
    /// Numeric comparisons with boundaries
    GreaterThanOrEqual(T),
    LessThanOrEqual(T),
    /// Array length
    Length(usize),
    /// Truthy/falsy
    Truthy,
    Falsy,
    /// toBeTruthy / toBeFalsy - type-aware checks
    ToBeTruthy,
    ToBeFalsy,
    /// Error throwing validation
    Throw(Option<String>),
    /// Object property existence and value
    HaveProperty(String, Option<serde_json::Value>),
    /// Type instance checking
    InstanceOf(String),
    /// Partial object matching
    MatchObject(serde_json::Value),
    /// Strict equality (no type coercion)
    StrictEqual(serde_json::Value),
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
    // New matchers constructors (Stage 93 Phase 3.4)
    /// toBeTruthy - checks if value is truthy (converts type appropriately)
    pub fn to_be_truthy() -> Self {
        ExtendedMatcher::ToBeTruthy
    }
    /// toBeFalsy - checks if value is falsy (converts type appropriately)
    pub fn to_be_falsy() -> Self {
        ExtendedMatcher::ToBeFalsy
    }
    /// toThrow - validates error throwing (optional error message)
    pub fn to_throw(expected_message: Option<String>) -> Self {
        ExtendedMatcher::Throw(expected_message)
    }
    /// toHaveProperty - checks object property existence (optional expected value)
    pub fn to_have_property(property_name: String, expected_value: Option<serde_json::Value>) -> Self {
        ExtendedMatcher::HaveProperty(property_name, expected_value)
    }
    /// toBeInstanceOf - checks object type
    pub fn to_be_instance_of(type_name: String) -> Self {
        ExtendedMatcher::InstanceOf(type_name)
    }
    /// toMatchObject - partial object matching
    pub fn to_match_object(pattern: serde_json::Value) -> Self {
        ExtendedMatcher::MatchObject(pattern)
    }
    /// toStrictEqual - strict equality (no type coercion)
    pub fn to_strict_equal(expected: serde_json::Value) -> Self {
        ExtendedMatcher::StrictEqual(expected)
    }
    /// toBeGreaterThanOrEqual - numeric comparison
    pub fn to_be_greater_than_or_equal(value: T) -> Self {
        ExtendedMatcher::GreaterThanOrEqual(value)
    }
    /// toBeLessThanOrEqual - numeric comparison
    pub fn to_be_less_than_or_equal(value: T) -> Self {
        ExtendedMatcher::LessThanOrEqual(value)
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
                // Compare as f64 for numbers
                if let (Some(a), Some(b)) = (
                    serde_json::to_value(value).ok().and_then(|v| v.as_f64()),
                    serde_json::to_value(expected).ok().and_then(|v| v.as_f64())
                ) {
                    a > b
                } else {
                    // Fallback to string comparison
                    format!("{:?}", value) > format!("{:?}", expected)
                }
            }
            ExtendedMatcher::LessThan(expected) => {
                // Compare as f64 for numbers
                if let (Some(a), Some(b)) = (
                    serde_json::to_value(value).ok().and_then(|v| v.as_f64()),
                    serde_json::to_value(expected).ok().and_then(|v| v.as_f64())
                ) {
                    a < b
                } else {
                    // Fallback to string comparison
                    format!("{:?}", value) < format!("{:?}", expected)
                }
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
                let json: _ = serde_json::to_value(value).unwrap_or(serde_json::Value::Null);
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
                let json: _ = serde_json::to_value(value).unwrap_or(serde_json::Value::Null);
                match json {
                    serde_json::Value::Null => true,
                    serde_json::Value::Bool(b) => !b,
                    serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0) == 0.0,
                    serde_json::Value::String(s) => s.is_empty() || s == "false" || s == "0",
                    serde_json::Value::Array(arr) => arr.is_empty(),
                    serde_json::Value::Object(obj) => obj.is_empty(),
                }
            }
            // Stage 93 Phase 3.4 - New matchers implementation
            ExtendedMatcher::ToBeTruthy => {
                // toBeTruthy - checks if value is truthy with type awareness
                let json: _ = serde_json::to_value(value).unwrap_or(serde_json::Value::Null);
                match json {
                    serde_json::Value::Null => false,
                    serde_json::Value::Bool(b) => b,
                    serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0) != 0.0,
                    serde_json::Value::String(s) => !s.is_empty() && s != "false" && s != "0",
                    serde_json::Value::Array(arr) => !arr.is_empty(),
                    serde_json::Value::Object(obj) => !obj.is_empty(),
                }
            }
            ExtendedMatcher::ToBeFalsy => {
                // toBeFalsy - checks if value is falsy with type awareness
                let json: _ = serde_json::to_value(value).unwrap_or(serde_json::Value::Null);
                match json {
                    serde_json::Value::Null => true,
                    serde_json::Value::Bool(b) => !b,
                    serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0) == 0.0,
                    serde_json::Value::String(s) => s.is_empty() || s == "false" || s == "0",
                    serde_json::Value::Array(arr) => arr.is_empty(),
                    serde_json::Value::Object(obj) => obj.is_empty(),
                }
            }
            ExtendedMatcher::Throw(expected_message) => {
                // toThrow - validates error throwing
                // Expects a JSON object with "thrown": true and optionally "message"
                if let Ok(json) = serde_json::to_value(value) {
                    if let Some(thrown) = json.get("thrown").and_then(|v| v.as_bool()) {
                        if !thrown {
                            return false;
                        }
                        if let Some(expected) = expected_message {
                            if let Some(actual_msg) = json.get("message").and_then(|v| v.as_str()) {
                                return actual_msg.contains(expected);
                            }
                            return false;
                        }
                        return true;
                    }
                    // Also check for error object
                    if json.get("error").is_some() || json.get("exception").is_some() {
                        return true;
                    }
                }
                false
            }
            ExtendedMatcher::HaveProperty(property_name, expected_value) => {
                // toHaveProperty - checks object property existence and value
                if let Ok(json) = serde_json::to_value(value) {
                    if let serde_json::Value::Object(obj) = json {
                        // Support nested property access (e.g., "address.city")
                        let parts: Vec<&str> = property_name.split('.').collect();
                        let mut current = Some(&obj);

                        for (i, part) in parts.iter().enumerate() {
                            if let Some(value_ref) = current {
                                if let Some(prop_value) = value_ref.get(*part) {
                                    if i == parts.len() - 1 {
                                        // Last part - check value if provided
                                        if let Some(expected) = expected_value {
                                            return prop_value == expected;
                                        }
                                        return true; // Property exists
                                    }
                                    // Continue to nested property
                                    current = prop_value.as_object();
                                } else {
                                    return false; // Property doesn't exist
                                }
                            } else {
                                return false;
                            }
                        }
                        false
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            ExtendedMatcher::InstanceOf(type_name) => {
                // toBeInstanceOf - checks object type
                if let Ok(json) = serde_json::to_value(value) {
                    match json {
                        serde_json::Value::String(_) => type_name == "String",
                        serde_json::Value::Number(_) => type_name == "Number" || type_name == "Number",
                        serde_json::Value::Bool(_) => type_name == "Boolean",
                        serde_json::Value::Array(_) => type_name == "Array",
                        serde_json::Value::Object(_) => type_name == "Object",
                        serde_json::Value::Null => type_name == "Null",
                    }
                } else {
                    false
                }
            }
            ExtendedMatcher::MatchObject(pattern) => {
                // toMatchObject - partial object matching with nested support
                fn matches_partial(value: &serde_json::Value, pattern: &serde_json::Value) -> bool {
                    match (value, pattern) {
                        (serde_json::Value::Object(value_obj), serde_json::Value::Object(pattern_obj)) => {
                            // Check if all pattern properties exist in value with matching values
                            for (key, expected) in pattern_obj {
                                if let Some(actual) = value_obj.get(key) {
                                    // Recursively check for nested partial matching
                                    if !matches_partial(actual, expected) {
                                        return false;
                                    }
                                } else {
                                    return false;
                                }
                            }
                            true
                        }
                        (actual, expected) => actual == expected,
                    }
                }

                if let Ok(value_json) = serde_json::to_value(value) {
                    matches_partial(&value_json, &pattern)
                } else {
                    false
                }
            }
            ExtendedMatcher::StrictEqual(expected) => {
                // toStrictEqual - strict equality without type coercion
                if let Ok(value_json) = serde_json::to_value(value) {
                    // Check exact type match
                    match (&value_json, expected) {
                        (serde_json::Value::String(_), serde_json::Value::String(_)) => {
                            value_json == *expected
                        }
                        (serde_json::Value::Number(_), serde_json::Value::Number(_)) => {
                            // Compare as f64 for numbers
                            let a = value_json.as_f64().unwrap_or(0.0);
                            let b = expected.as_f64().unwrap_or(0.0);
                            (a - b).abs() < f64::EPSILON
                        }
                        (serde_json::Value::Bool(_), serde_json::Value::Bool(_)) => {
                            value_json == *expected
                        }
                        (serde_json::Value::Array(_), serde_json::Value::Array(_)) => {
                            // Strict array comparison - same length and all elements equal
                            let a = value_json.as_array().unwrap();
                            let b = expected.as_array().unwrap();
                            if a.len() != b.len() {
                                return false;
                            }
                            a.iter().zip(b.iter()).all(|(x, y)| {
                                // Recursively compare
                                let x_json = serde_json::json!(x);
                                let y_json = serde_json::json!(y);
                                x_json == y_json
                            })
                        }
                        (serde_json::Value::Object(_), serde_json::Value::Object(_)) => {
                            // Strict object comparison
                            value_json == *expected
                        }
                        (serde_json::Value::Null, serde_json::Value::Null) => true,
                        _ => false,
                    }
                } else {
                    false
                }
            }
            ExtendedMatcher::GreaterThanOrEqual(expected) => {
                // Compare as f64 for numbers
                if let (Some(a), Some(b)) = (
                    serde_json::to_value(value).ok().and_then(|v| v.as_f64()),
                    serde_json::to_value(expected).ok().and_then(|v| v.as_f64())
                ) {
                    a >= b
                } else {
                    // Fallback to string comparison
                    format!("{:?}", value) >= format!("{:?}", expected)
                }
            }
            ExtendedMatcher::LessThanOrEqual(expected) => {
                // Compare as f64 for numbers
                if let (Some(a), Some(b)) = (
                    serde_json::to_value(value).ok().and_then(|v| v.as_f64()),
                    serde_json::to_value(expected).ok().and_then(|v| v.as_f64())
                ) {
                    a <= b
                } else {
                    // Fallback to string comparison
                    format!("{:?}", value) <= format!("{:?}", expected)
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
            // Stage 93 Phase 3.4 - New matchers messages
            ExtendedMatcher::ToBeTruthy => {
                format!("Expected {:?} to be truthy", value)
            }
            ExtendedMatcher::ToBeFalsy => {
                format!("Expected {:?} to be falsy", value)
            }
            ExtendedMatcher::Throw(expected_message) => {
                if let Some(msg) = expected_message {
                    format!("Expected {:?} to throw an error containing '{}'", value, msg)
                } else {
                    format!("Expected {:?} to throw an error", value)
                }
            }
            ExtendedMatcher::HaveProperty(property_name, expected_value) => {
                if let Some(expected) = expected_value {
                    format!("Expected {:?} to have property '{}' with value {:?}", value, property_name, expected)
                } else {
                    format!("Expected {:?} to have property '{}'", value, property_name)
                }
            }
            ExtendedMatcher::InstanceOf(type_name) => {
                format!("Expected {:?} to be an instance of {}", value, type_name)
            }
            ExtendedMatcher::MatchObject(pattern) => {
                format!("Expected {:?} to match object {:?}", value, pattern)
            }
            ExtendedMatcher::StrictEqual(expected) => {
                format!("Expected {:?} to strictly equal {:?}", value, expected)
            }
            ExtendedMatcher::GreaterThanOrEqual(expected) => {
                format!("Expected {:?} to be greater than or equal to {:?}", value, expected)
            }
            ExtendedMatcher::LessThanOrEqual(expected) => {
                format!("Expected {:?} to be less than or equal to {:?}", value, expected)
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_equal_matcher() {
        let matcher: _ = ExtendedMatcher::Equal(42);
        let value: _ = 42;
        assert!(matcher.matches(&value));
        assert!(matcher.message(&value).contains("equal"));
    }
    #[test]
    fn test_contains_matcher() {
        let matcher: _ = ExtendedMatcher::Contains("test".to_string());
        let value: _ = "this is a test string";
        assert!(matcher.matches(&value));
    }
}

// Tests for assertions are in tests/ directory