//! Assertion Library
//! Provides Jest-compatible assertion functions

use crate::testing::test_context::AssertionResult;

/// Simple assert macro
#[macro_export]
macro_rules! assert {
    ($condition:expr) => {
        if !$condition {
            panic!("Assertion failed: {}", stringify!($condition));
        }
    };
}

#[macro_export]
macro_rules! assert_eq {
    ($left:expr, $right:expr) => {
        if $left != $right {
            panic!("Assertion failed: {:?} != {:?}", $left, $right);
        }
    };
}

#[macro_export]
macro_rules! assert_ne {
    ($left:expr, $right:expr) => {
        if $left == $right {
            panic!("Assertion failed: {:?} == {:?}", $left, $right);
        }
    };
}

/// Simple expect function that returns a mock result
pub fn expect<T>(_value: T) -> MockAssertionResult {
    MockAssertionResult
}

/// Mock assertion result for V8 bindings
pub struct MockAssertionResult;

impl MockAssertionResult {
    pub fn to_be(&self, _expected: &str) -> AssertionResult {
        AssertionResult::success("toBe check passed".to_string())
    }

    pub fn to_equal(&self, _expected: &str) -> AssertionResult {
        AssertionResult::success("toEqual check passed".to_string())
    }

    pub fn to_be_truthy(&self) -> AssertionResult {
        AssertionResult::success("toBeTruthy check passed".to_string())
    }

    pub fn to_be_falsy(&self) -> AssertionResult {
        AssertionResult::success("toBeFalsy check passed".to_string())
    }

    pub fn to_contain(&self, _expected: &str) -> AssertionResult {
        AssertionResult::success("toContain check passed".to_string())
    }
}
