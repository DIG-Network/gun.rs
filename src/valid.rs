use serde_json::Value;

/// Valid module - validates and checks if data is a soul (reference)
/// Based on Gun.js valid.js - matches the exact validation logic
///
/// Main validation function - matches Gun.js valid()
/// Valid values are: null, string, boolean, number (!Infinity, !NaN), or soul relation
/// Returns:
/// - true if valid (for simple values)
/// - Some(soul_string) if it's a soul reference
/// - false if invalid
pub fn valid(value: &Value) -> Result<bool, Option<String>> {
    match value {
        Value::Null => Ok(true), // null is valid (deletes)

        Value::String(_) => Ok(true), // strings are always valid

        Value::Bool(_) => Ok(true), // booleans are valid

        Value::Number(n) => {
            // Numbers are valid if not Infinity and not NaN
            // v === v checks for NaN (NaN !== NaN)
            if let Some(f) = n.as_f64() {
                if f.is_finite() && f == f {
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

/// Check if a value is a valid soul (reference to another node)
/// Returns Some(soul) if it's a soul string or soul object, None otherwise
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

/// Check if data is valid for gun
/// This is a convenience function for basic validation
pub fn is_valid_data(value: &Value) -> bool {
    match valid(value) {
        Ok(true) => true,
        Err(Some(_)) => true, // Soul reference is valid
        _ => false,
    }
}
