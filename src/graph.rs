use crate::error::GunResult;
use crate::state::Node;
use parking_lot::RwLock;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Graph - stores all nodes in the database
/// Based on Gun.js graph structure
#[derive(Clone)]
pub struct Graph {
    nodes: Arc<RwLock<HashMap<String, Node>>>,
}

impl Graph {
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get a node by soul
    pub fn get(&self, soul: &str) -> Option<Node> {
        self.nodes.read().get(soul).cloned()
    }

    /// Put a node into the graph
    pub fn put(&self, soul: &str, node: Node) -> GunResult<()> {
        self.nodes.write().insert(soul.to_string(), node);
        Ok(())
    }

    /// Check if a node exists
    pub fn has(&self, soul: &str) -> bool {
        self.nodes.read().contains_key(soul)
    }

    /// Get all nodes (for debugging/testing)
    pub fn all_nodes(&self) -> HashMap<String, Node> {
        self.nodes.read().clone()
    }

    /// Merge a node into the graph with conflict resolution
    /// Based on HAM (Hypothetical Amnesia Machine) algorithm
    pub fn merge(
        &self,
        soul: &str,
        incoming: &Node,
        state_fn: impl Fn() -> f64,
    ) -> GunResult<Node> {
        let mut nodes = self.nodes.write();
        let existing = nodes.get(soul);

        if let Some(existing_node) = existing {
            // Merge logic - resolve conflicts based on state timestamps
            let merged = Self::merge_nodes(existing_node, incoming, state_fn)?;
            nodes.insert(soul.to_string(), merged.clone());
            Ok(merged)
        } else {
            nodes.insert(soul.to_string(), incoming.clone());
            Ok(incoming.clone())
        }
    }

    /// Merge two nodes resolving conflicts based on state
    fn merge_nodes(
        existing: &Node,
        incoming: &Node,
        state_fn: impl Fn() -> f64,
    ) -> GunResult<Node> {
        let mut merged = existing.clone();
        let current_state = state_fn();

        // Get states from both nodes
        let existing_states = existing
            .meta
            .get(">")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_else(|| serde_json::Map::new());

        let incoming_states = incoming
            .meta
            .get(">")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_else(|| serde_json::Map::new());

        // Merge data fields based on state comparison
        for (key, incoming_value) in incoming.data.iter() {
            let existing_state = existing_states
                .get(key)
                .and_then(|v| v.as_f64())
                .unwrap_or(f64::NEG_INFINITY);

            let incoming_state = incoming_states
                .get(key)
                .and_then(|v| v.as_f64())
                .unwrap_or(f64::NEG_INFINITY);

            if incoming_state >= existing_state {
                merged.data.insert(key.clone(), incoming_value.clone());

                // Update state
                let states = merged
                    .meta
                    .entry(">".to_string())
                    .or_insert_with(|| Value::Object(serde_json::Map::new()));

                if let Value::Object(ref mut map) = states {
                    map.insert(
                        key.clone(),
                        Value::Number(serde_json::Number::from_f64(incoming_state).unwrap()),
                    );
                }
            }
        }

        // Merge meta fields
        for (key, value) in incoming.meta.iter() {
            if key != ">" && key != "#" {
                merged.meta.insert(key.clone(), value.clone());
            }
        }

        Ok(merged)
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new()
    }
}
