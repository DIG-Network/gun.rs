use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// State module - manages timestamps for conflict resolution
/// Based on Gun.js state.js
const DRIFT: f64 = 0.0;
const D: f64 = 999.0;

#[derive(Clone)]
pub struct State {
    n: Arc<Mutex<f64>>,
    last: Arc<Mutex<f64>>,
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    pub fn new() -> Self {
        Self {
            n: Arc::new(Mutex::new(0.0)),
            last: Arc::new(Mutex::new(f64::NEG_INFINITY)),
        }
    }

    /// Generate a new state timestamp
    /// Returns a timestamp that increases with each call
    /// 
    /// # Panics
    /// This function will panic if the system time is before the Unix epoch,
    /// which should never happen in practice on modern systems.
    pub fn next(&self) -> f64 {
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time is before Unix epoch - this should never happen")
            .as_millis() as f64;

        // std::sync::Mutex::lock() returns Result, handle potential poisoning
        // Mutex poisoning indicates a panic occurred while holding the lock,
        // which is a serious bug but we can still recover by using expect()
        let mut n = self.n.lock().expect("State mutex poisoned - this indicates a serious bug");
        let mut last = self.last.lock().expect("State mutex poisoned - this indicates a serious bug");

        if *last < t {
            *n = 0.0;
            *last = t + DRIFT;
            return *last;
        }

        *last = t + (*n / D) + DRIFT;
        *n += 1.0;
        *last
    }

    /// Get state for a key on a node
    pub fn is(node: &Option<Node>, key: &str) -> Option<f64> {
        if let Some(node) = node {
            if let Some(Value::Object(states)) = node.meta.get(">") {
                if let Some(state) = states.get(key) {
                    if let Some(state_num) = state.as_f64() {
                        return Some(state_num);
                    }
                }
            }
        }
        None
    }

    /// Put a key's state on a node
    pub fn ify(
        node: &mut Node,
        key: Option<&str>,
        state: Option<f64>,
        value: Option<serde_json::Value>,
        soul: Option<&str>,
    ) -> Node {
        if let Some(soul) = soul {
            node.meta
                .insert("#".to_string(), serde_json::Value::String(soul.to_string()));
        }

        if let Some(key) = key {
            if key != "_" {
                let states = node
                    .meta
                    .entry(">".to_string())
                    .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));

                if let Some(state_num) = state {
                    if let serde_json::Value::Object(ref mut map) = states {
                    // from_f64 can fail if state_num is NaN or Infinity
                    // In practice, state_num should always be a valid finite number
                    if let Some(number) = serde_json::Number::from_f64(state_num) {
                        map.insert(key.to_string(), serde_json::Value::Number(number));
                    } else {
                        // Log error but continue - this should never happen in practice
                        tracing::error!("Invalid state number: {} (NaN or Infinity)", state_num);
                    }
                    }
                }

                if let Some(val) = value {
                    node.data.insert(key.to_string(), val);
                }
            }
        }

        node.clone()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Node {
    pub data: serde_json::Map<String, serde_json::Value>,
    pub meta: serde_json::Map<String, serde_json::Value>,
}

impl Node {
    pub fn new() -> Self {
        Self {
            data: serde_json::Map::new(),
            meta: serde_json::Map::new(),
        }
    }

    pub fn with_soul(soul: String) -> Self {
        let mut meta = serde_json::Map::new();
        meta.insert("#".to_string(), serde_json::Value::String(soul));
        Self {
            data: serde_json::Map::new(),
            meta,
        }
    }

    pub fn get_soul(&self) -> Option<String> {
        self.meta
            .get("#")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }
}

impl Default for Node {
    fn default() -> Self {
        Self::new()
    }
}
