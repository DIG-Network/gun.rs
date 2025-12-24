use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use parking_lot::Mutex;

/// Event system - handles events and hooks
/// Based on Gun.js onto.js and event system

pub type EventCallback = Box<dyn Fn(&Event) + Send + Sync>;

#[derive(Clone, Debug)]
pub struct Event {
    pub event_type: String,
    pub data: serde_json::Value,
}

struct ListenerEntry {
    id: u64,
    callback: Arc<EventCallback>,
}

pub struct EventEmitter {
    listeners: Arc<RwLock<HashMap<String, Vec<ListenerEntry>>>>,
    id_counter: Arc<Mutex<u64>>,
}

impl EventEmitter {
    pub fn new() -> Self {
        Self {
            listeners: Arc::new(RwLock::new(HashMap::new())),
            id_counter: Arc::new(Mutex::new(0)),
        }
    }

    /// Register an event listener
    /// Returns the listener ID for later removal
    pub fn on(&self, event_type: &str, callback: EventCallback) -> u64 {
        let mut listeners = self.listeners.write().unwrap();
        let callbacks = listeners.entry(event_type.to_string())
            .or_insert_with(Vec::new);
        
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
    pub fn off(&self, event_type: &str, id: u64) {
        let mut listeners = self.listeners.write().unwrap();
        if let Some(callbacks) = listeners.get_mut(event_type) {
            callbacks.retain(|entry| entry.id != id);
            // Remove empty event types
            if callbacks.is_empty() {
                listeners.remove(event_type);
            }
        }
    }
    
    /// Remove all listeners for an event type
    pub fn off_all(&self, event_type: &str) {
        let mut listeners = self.listeners.write().unwrap();
        listeners.remove(event_type);
    }

    /// Emit an event
    pub fn emit(&self, event: &Event) {
        let listeners = self.listeners.read().unwrap();
        if let Some(callbacks) = listeners.get(&event.event_type) {
            // Clone callbacks to avoid holding lock during execution
            let callbacks: Vec<Arc<EventCallback>> = callbacks.iter()
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
    pub fn listener_count(&self, event_type: &str) -> usize {
        let listeners = self.listeners.read().unwrap();
        listeners.get(event_type).map(|v| v.len()).unwrap_or(0)
    }
}

impl Default for EventEmitter {
    fn default() -> Self {
        Self::new()
    }
}

