use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Message deduplication module - matches Gun.js dup.js
/// Tracks message IDs to prevent duplicate processing in DAM mesh
pub struct Dup {
    messages: Arc<RwLock<HashMap<String, MessageEntry>>>,
    max_age: Duration,
    max_size: usize,
}

#[derive(Clone, Debug)]
struct MessageEntry {
    was: Instant,
    via: Option<String>,           // peer ID that sent this
    it: Option<serde_json::Value>, // optional message data
}

impl Dup {
    pub fn new(max_size: usize, max_age_ms: u64) -> Self {
        Self {
            messages: Arc::new(RwLock::new(HashMap::new())),
            max_age: Duration::from_millis(max_age_ms),
            max_size,
        }
    }

    /// Default constructor matching Gun.js defaults
    /// max: 999, age: 9000ms
    pub fn default() -> Self {
        Self::new(999, 9000)
    }

    /// Check if message ID was already seen (deduplication check)
    /// Returns true if duplicate, false if new
    pub fn check(&self, id: &str) -> bool {
        let messages = self.messages.read().unwrap();
        if let Some(entry) = messages.get(id) {
            // Check if still valid (not expired)
            if entry.was.elapsed() < self.max_age {
                return true; // duplicate
            }
        }
        false // new message
    }

    /// Track a message ID (marks it as seen)
    /// Returns true if tracked
    pub fn track(&mut self, id: &str) -> bool {
        let mut messages = self.messages.write().unwrap();

        // Check size limit
        if messages.len() >= self.max_size {
            self.drop_expired(&mut messages);
        }

        let entry = messages
            .entry(id.to_string())
            .or_insert_with(|| MessageEntry {
                was: Instant::now(),
                via: None,
                it: None,
            });
        entry.was = Instant::now();
        true
    }

    /// Track with peer info
    pub fn track_with_peer(&mut self, id: &str, peer_id: Option<&str>) {
        let mut messages = self.messages.write().unwrap();
        let entry = messages
            .entry(id.to_string())
            .or_insert_with(|| MessageEntry {
                was: Instant::now(),
                via: None,
                it: None,
            });
        entry.was = Instant::now();
        if let Some(pid) = peer_id {
            entry.via = Some(pid.to_string());
        }
    }

    /// Drop expired entries
    fn drop_expired(&self, messages: &mut HashMap<String, MessageEntry>) {
        let now = Instant::now();
        messages.retain(|_, entry| now.duration_since(entry.was) < self.max_age);
    }

    /// Drop all expired entries
    pub fn drop_expired_all(&self) {
        let mut messages = self.messages.write().unwrap();
        self.drop_expired(&mut messages);
    }

    /// Get peer that sent this message (for routing)
    pub fn get_via(&self, id: &str) -> Option<String> {
        let messages = self.messages.read().unwrap();
        messages.get(id).and_then(|e| e.via.clone())
    }

    /// Store message data with ID
    pub fn store(&mut self, id: &str, data: serde_json::Value) {
        let mut messages = self.messages.write().unwrap();
        if let Some(entry) = messages.get_mut(id) {
            entry.it = Some(data);
        } else {
            messages.insert(
                id.to_string(),
                MessageEntry {
                    was: Instant::now(),
                    via: None,
                    it: Some(data),
                },
            );
        }
    }

    /// Get stored message data
    pub fn get(&self, id: &str) -> Option<serde_json::Value> {
        let messages = self.messages.read().unwrap();
        messages.get(id).and_then(|e| e.it.clone())
    }

    /// Remove a specific ID (used for special cases like DAM self-deduplication)
    pub fn remove(&self, id: &str) {
        let mut messages = self.messages.write().unwrap();
        messages.remove(id);
    }
}

impl Default for Dup {
    fn default() -> Self {
        Self::default()
    }
}
