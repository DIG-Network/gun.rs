//! Message deduplication for DAM protocol
//!
//! This module implements message deduplication to prevent processing the same message
//! multiple times in the DAM mesh network. It tracks message IDs with timestamps and
//! automatically expires old entries.
//!
//! Based on Gun.js `dup.js`. This is critical for preventing infinite loops and
//! duplicate processing in peer-to-peer message routing.
//!
//! ## Features
//!
//! - Tracks message IDs with timestamps
//! - Automatic expiration of old entries
//! - Configurable maximum size and age
//! - Tracks peer information for routing

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Message deduplication tracker
///
/// Tracks message IDs to prevent duplicate processing. Messages are stored with
/// a timestamp and automatically expire after `max_age`. The tracker has a
/// maximum size and will clean up expired entries when full.
///
/// # Default Configuration
///
/// - Max size: 999 messages
/// - Max age: 9000ms (9 seconds)
///
/// # Thread Safety
///
/// `Dup` is thread-safe when wrapped in `Arc<RwLock<Dup>>` (as used in `GunCore`).
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
    /// Create a new deduplication tracker with custom settings
    ///
    /// # Arguments
    /// * `max_size` - Maximum number of message IDs to track
    /// * `max_age_ms` - Maximum age in milliseconds before entries expire
    pub fn new(max_size: usize, max_age_ms: u64) -> Self {
        Self {
            messages: Arc::new(RwLock::new(HashMap::new())),
            max_age: Duration::from_millis(max_age_ms),
            max_size,
        }
    }

    /// Create a deduplication tracker with default settings
    ///
    /// Defaults match Gun.js:
    /// - Max size: 999 messages
    /// - Max age: 9000ms (9 seconds)
    pub fn new_default() -> Self {
        Self::new(999, 9000)
    }

    /// Check if a message ID was already seen
    ///
    /// This performs the deduplication check - if the message ID was seen recently
    /// (within `max_age`), it's considered a duplicate.
    ///
    /// # Arguments
    /// * `id` - The message ID to check
    ///
    /// # Returns
    /// - `true` if the message is a duplicate (already seen and not expired)
    /// - `false` if the message is new or has expired
    ///
    /// # Panics
    /// This function will panic if the lock is poisoned, which should never happen
    /// in practice since we don't panic while holding the lock.
    pub fn check(&self, id: &str) -> bool {
        let messages = self.messages.read().expect("Dup lock poisoned");
        if let Some(entry) = messages.get(id) {
            // Check if still valid (not expired)
            if entry.was.elapsed() < self.max_age {
                return true; // duplicate
            }
        }
        false // new message
    }

    /// Track a message ID (mark it as seen)
    ///
    /// Records the message ID with the current timestamp. If the tracker is full,
    /// expired entries are cleaned up first.
    ///
    /// # Arguments
    /// * `id` - The message ID to track
    ///
    /// # Returns
    /// Always returns `true` if the message was tracked.
    ///
    /// # Panics
    /// This function will panic if the lock is poisoned, which should never happen
    /// in practice since we don't panic while holding the lock.
    pub fn track(&mut self, id: &str) -> bool {
        let mut messages = self.messages.write().expect("Dup lock poisoned");

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

    /// Track a message ID with peer information
    ///
    /// Records the message ID along with which peer sent it. This is useful for
    /// routing and understanding message propagation paths.
    ///
    /// # Arguments
    /// * `id` - The message ID to track
    /// * `peer_id` - Optional peer ID that sent the message
    ///
    /// # Panics
    /// This function will panic if the lock is poisoned, which should never happen
    /// in practice since we don't panic while holding the lock.
    pub fn track_with_peer(&mut self, id: &str, peer_id: Option<&str>) {
        let mut messages = self.messages.write().expect("Dup lock poisoned");
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
    /// 
    /// # Panics
    /// This function will panic if the lock is poisoned, which should never happen
    /// in practice since we don't panic while holding the lock.
    pub fn drop_expired_all(&self) {
        let mut messages = self.messages.write().expect("Dup lock poisoned");
        self.drop_expired(&mut messages);
    }

    /// Get the peer ID that sent a specific message (for routing)
    ///
    /// # Arguments
    /// * `id` - The message ID to look up
    ///
    /// # Returns
    /// The peer ID if the message was tracked with peer information, `None` otherwise.
    ///
    /// # Panics
    /// This function will panic if the lock is poisoned, which should never happen
    /// in practice since we don't panic while holding the lock.
    pub fn get_via(&self, id: &str) -> Option<String> {
        let messages = self.messages.read().expect("Dup lock poisoned");
        messages.get(id).and_then(|e| e.via.clone())
    }

    /// Store message data along with its ID
    ///
    /// This allows retrieving the full message data later, which can be useful
    /// for certain DAM protocol operations.
    ///
    /// # Arguments
    /// * `id` - The message ID
    /// * `data` - The message data to store
    ///
    /// # Panics
    /// This function will panic if the lock is poisoned, which should never happen
    /// in practice since we don't panic while holding the lock.
    pub fn store(&mut self, id: &str, data: serde_json::Value) {
        let mut messages = self.messages.write().expect("Dup lock poisoned");
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

    /// Get stored message data by ID
    ///
    /// # Arguments
    /// * `id` - The message ID to look up
    ///
    /// # Returns
    /// The stored message data if found, `None` otherwise.
    ///
    /// # Panics
    /// This function will panic if the lock is poisoned, which should never happen
    /// in practice since we don't panic while holding the lock.
    pub fn get(&self, id: &str) -> Option<serde_json::Value> {
        let messages = self.messages.read().expect("Dup lock poisoned");
        messages.get(id).and_then(|e| e.it.clone())
    }

    /// Remove a specific message ID from tracking
    ///
    /// This is used for special cases like DAM self-deduplication where we need
    /// to remove an entry before its natural expiration.
    ///
    /// # Arguments
    /// * `id` - The message ID to remove
    ///
    /// # Panics
    /// This function will panic if the lock is poisoned, which should never happen
    /// in practice since we don't panic while holding the lock.
    pub fn remove(&self, id: &str) {
        let mut messages = self.messages.write().expect("Dup lock poisoned");
        messages.remove(id);
    }
}

impl Default for Dup {
    fn default() -> Self {
        Self::new_default()
    }
}
