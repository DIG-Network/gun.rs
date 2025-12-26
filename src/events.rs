//! Event system for reactive updates
//!
//! This module implements Gun's event emitter pattern, allowing components to subscribe
//! to and emit events. Events are used throughout Gun for:
//! - Notifying listeners when nodes are updated
//! - Triggering network synchronization
//! - Reacting to graph changes
//!
//! Based on Gun.js `onto.js` and event system. The event emitter is thread-safe and
//! supports multiple listeners per event type.

use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Event callback function type
///
/// All event listeners must implement this type. The callback receives the event
/// that was emitted and can access its type and data.
///
/// # Thread Safety
///
/// Callbacks must be `Send + Sync` to be used across threads.
pub type EventCallback = Box<dyn Fn(&Event) + Send + Sync>;

/// An event in the Gun event system
///
/// Events consist of:
/// - `event_type`: A string identifier for the event (e.g., "node_update:user_123")
/// - `data`: Arbitrary JSON data associated with the event
///
/// # Example
///
/// ```rust,no_run
/// use gun::events::Event;
/// use serde_json::json;
///
/// let event = Event {
///     event_type: "node_update:user_123".to_string(),
///     data: json!({"name": "Alice", "age": 30}),
/// };
/// ```
#[derive(Clone, Debug)]
pub struct Event {
    pub event_type: String,
    pub data: serde_json::Value,
}

struct ListenerEntry {
    id: u64,
    callback: Arc<EventCallback>,
}

/// Event emitter for subscribing to and emitting events
///
/// The event emitter maintains a map of event types to their listeners.
/// Multiple listeners can subscribe to the same event type, and all will be
/// called when the event is emitted.
///
/// # Thread Safety
///
/// `EventEmitter` is thread-safe and can be shared across threads using `Arc<EventEmitter>`.
///
/// # Example
///
/// ```rust,no_run
/// use gun::events::{EventEmitter, Event};
/// use serde_json::json;
/// use std::sync::Arc;
///
/// let emitter = Arc::new(EventEmitter::new());
///
/// // Subscribe to events
/// let emitter_clone = emitter.clone();
/// emitter.on("test_event", Box::new(move |event: &Event| {
///     println!("Received event: {:?}", event);
/// }));
///
/// // Emit an event
/// emitter.emit(&Event {
///     event_type: "test_event".to_string(),
///     data: json!({"message": "hello"}),
/// });
/// ```
pub struct EventEmitter {
    listeners: Arc<RwLock<HashMap<String, Vec<ListenerEntry>>>>,
    id_counter: Arc<Mutex<u64>>,
}

impl EventEmitter {
    /// Create a new event emitter
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(RwLock::new(HashMap::new())),
            id_counter: Arc::new(Mutex::new(0)),
        }
    }

    /// Register an event listener
    /// Returns the listener ID for later removal
    /// 
    /// # Panics
    /// This function will panic if the lock is poisoned, which should never happen
    /// in practice since we don't panic while holding the lock.
    pub fn on(&self, event_type: &str, callback: EventCallback) -> u64 {
        let mut listeners = self.listeners.write().expect("EventEmitter lock poisoned");
        let callbacks = listeners.entry(event_type.to_string()).or_default();

        let id = {
            let mut counter = self.id_counter.lock();
            *counter += 1;
            *counter
        };

        callbacks.push(ListenerEntry {
            id,
            callback: Arc::new(callback),
        });
        id
    }

    /// Remove an event listener by ID
    /// 
    /// # Panics
    /// This function will panic if the lock is poisoned, which should never happen
    /// in practice since we don't panic while holding the lock.
    pub fn off(&self, event_type: &str, id: u64) {
        let mut listeners = self.listeners.write().expect("EventEmitter lock poisoned");
        if let Some(callbacks) = listeners.get_mut(event_type) {
            callbacks.retain(|entry| entry.id != id);
            // Remove empty event types
            if callbacks.is_empty() {
                listeners.remove(event_type);
            }
        }
    }

    /// Remove all listeners for an event type
    /// 
    /// # Panics
    /// This function will panic if the lock is poisoned, which should never happen
    /// in practice since we don't panic while holding the lock.
    pub fn off_all(&self, event_type: &str) {
        let mut listeners = self.listeners.write().expect("EventEmitter lock poisoned");
        listeners.remove(event_type);
    }

    /// Emit an event
    /// 
    /// # Panics
    /// This function will panic if the lock is poisoned, which should never happen
    /// in practice since we don't panic while holding the lock.
    pub fn emit(&self, event: &Event) {
        let listeners = self.listeners.read().expect("EventEmitter lock poisoned");
        if let Some(callbacks) = listeners.get(&event.event_type) {
            // Clone callbacks to avoid holding lock during execution
            let callbacks: Vec<Arc<EventCallback>> = callbacks
                .iter()
                .map(|entry| entry.callback.clone())
                .collect();
            drop(listeners); // Release lock before calling callbacks

            for callback in callbacks.iter() {
                callback(event);
            }
        }
    }

    /// Remove all listeners for an event type
    pub fn remove_all_listeners(&self, event_type: &str) {
        self.off_all(event_type);
    }

    /// Get count of listeners for an event type
    /// 
    /// # Panics
    /// This function will panic if the lock is poisoned, which should never happen
    /// in practice since we don't panic while holding the lock.
    pub fn listener_count(&self, event_type: &str) -> usize {
        let listeners = self.listeners.read().expect("EventEmitter lock poisoned");
        listeners.get(event_type).map(|v| v.len()).unwrap_or(0)
    }
}

impl Default for EventEmitter {
    fn default() -> Self {
        Self::new()
    }
}
