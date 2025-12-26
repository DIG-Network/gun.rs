//! Data validation and soul reference detection
//!
//! This module validates data values according to Gun's rules and detects soul references.
//! Based on Gun.js `valid.js` - matches the exact validation logic.
//!
//! ## Valid Data Types
//!
//! - `null` - Valid (represents deletion)
//! - `string` - Always valid
//! - `boolean` - Always valid
//! - `number` - Valid if not `Infinity` or `NaN`
//! - Soul reference - Object with only `{"#": "soul_id"}` key
//! - Objects - Not directly valid (must be stored as nodes)
//! - Arrays - Not directly supported

use serde_json::Value;

/// Validate a value according to Gun's rules and detect soul references
///
/// This is the main validation function matching Gun.js `valid()`. It checks if
/// a value is valid for storage in Gun and detects if it's a soul reference.
///
/// # Arguments
/// * `value` - The JSON value to validate
///
/// # Returns
///
/// - `Ok(true)` - Valid simple value (null, string, boolean, or valid number)
/// - `Err(Some(soul))` - It's a soul reference object `{"#": soul}`
/// - `Ok(false)` - Invalid value (Infinity, NaN, array, or complex object)
///
/// # Example
///
/// ```rust,no_run
/// use gun::valid::valid;
/// use serde_json::json;
///
/// // Valid simple value
/// assert_eq!(valid(&json!("hello")), Ok(true));
///
/// // Valid number
/// assert_eq!(valid(&json!(42)), Ok(true));
///
/// // Invalid number
/// assert_eq!(valid(&json!(f64::INFINITY)), Ok(false));
///
/// // Soul reference
/// match valid(&json!({"#": "user_123"})) {
///     Err(Some(soul)) => println!("Found soul reference: {}", soul),
///     _ => {}
/// }
/// ```
pub fn valid(value: &Value) -> Result<bool, Option<String>> {
    match value {
        Value::Null => Ok(true), // null is valid (deletes)

        Value::String(_) => Ok(true), // strings are always valid

        Value::Bool(_) => Ok(true), // booleans are valid

        Value::Number(n) => {
            // Numbers are valid if not Infinity and not NaN
            // v === v checks for NaN (NaN !== NaN)
            if let Some(f) = n.as_f64() {
                if f.is_finite() && !f.is_nan() {
                    Ok(true)
                } else {
                    Ok(false) // Infinity or NaN
                }
            } else {
                Ok(true) // Integer is always valid
            }
        }

        Value::Object(obj) => {
            // Check if it's a soul reference: object with only "#" key
            if obj.len() == 1 {
                if let Some(soul_val) = obj.get("#") {
                    if let Some(soul) = soul_val.as_str() {
                        // It's a soul reference
                        return Err(Some(soul.to_string()));
                    }
                }
            }
            // Regular objects are not directly valid (must be nodes with soul)
            // But we allow them for put() to handle
            Ok(false)
        }

        Value::Array(_) => {
            // Arrays are not directly supported in Gun.js
            // They need special algorithms for concurrency
            Ok(false)
        }
    }
}

/// Check if a value is a valid soul reference
///
/// This checks if a value represents a soul reference, either as:
/// - A soul reference object: `{"#": "soul_id"}`
/// - A string that could be a soul ID
///
/// # Arguments
/// * `value` - The JSON value to check
///
/// # Returns
/// `Some(soul_string)` if it's a soul reference, `None` otherwise.
///
/// # Example
///
/// ```rust,no_run
/// use gun::valid::valid_soul;
/// use serde_json::json;
///
/// // Soul reference object
/// assert_eq!(valid_soul(&json!({"#": "user_123"})), Some("user_123".to_string()));
///
/// // Soul string
/// assert_eq!(valid_soul(&json!("user_123")), Some("user_123".to_string()));
///
/// // Not a soul
/// assert_eq!(valid_soul(&json!("hello")), Some("hello".to_string())); // Strings are treated as potential souls
/// ```
pub fn valid_soul(value: &Value) -> Option<String> {
    match valid(value) {
        Err(Some(soul)) => Some(soul), // It's a soul reference object
        Ok(true) => {
            // Check if it's a string that could be a soul
            if let Value::String(s) = value {
                if !s.is_empty() {
                    Some(s.clone())
                } else {
                    None
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Check if data is valid for storage in Gun
///
/// This is a convenience function that returns `true` if the value is either:
/// - A valid simple value (validated by [`valid`](valid))
/// - A soul reference
///
/// # Arguments
/// * `value` - The JSON value to check
///
/// # Returns
/// `true` if the value is valid for Gun, `false` otherwise.
///
/// # Example
///
/// ```rust,no_run
/// use gun::valid::is_valid_data;
/// use serde_json::json;
///
/// assert!(is_valid_data(&json!("hello")));
/// assert!(is_valid_data(&json!(42)));
/// assert!(is_valid_data(&json!({"#": "user_123"})));
/// assert!(!is_valid_data(&json!(f64::INFINITY)));
/// ```
pub fn is_valid_data(value: &Value) -> bool {
    match valid(value) {
        Ok(true) => true,
        Err(Some(_)) => true, // Soul reference is valid
        _ => false,
    }
}
