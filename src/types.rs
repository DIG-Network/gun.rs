use serde_json::Value;
use std::sync::Arc;

/// Message predicate function type
/// 
/// Receives the entire message object (as `serde_json::Value`) and returns:
/// - `true` to accept the message (continue processing)
/// - `false` to reject the message (drop silently)
/// 
/// The predicate is called after signature verification but before message processing.
/// This allows the application layer to implement custom message filtering logic,
/// such as:
/// - Filtering messages based on content
/// - Implementing rate limiting
/// - Enforcing access control policies
/// - Blocking specific message types
/// 
/// # Example
/// ```rust,no_run
/// use gun::MessagePredicate;
/// use std::sync::Arc;
/// 
/// // Only accept "put" messages
/// let predicate: MessagePredicate = Arc::new(|msg| {
///     msg.get("put").is_some()
/// });
/// ```
pub type MessagePredicate = Arc<dyn Fn(&Value) -> bool + Send + Sync>;

