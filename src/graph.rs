//! Graph storage and conflict resolution
//!
//! This module implements the in-memory graph storage for Gun. The graph stores all
//! nodes by their soul (unique identifier) and provides conflict resolution using
//! the HAM (Hypothetical Amnesia Machine) algorithm.
//!
//! Based on Gun.js graph structure. The graph is thread-safe and can be shared
//! across threads using `Arc<Graph>`.

use crate::error::GunResult;
use crate::state::Node;
use parking_lot::RwLock;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Graph storage for all nodes in the database
///
/// The graph is an in-memory hash map that stores nodes by their soul (unique identifier).
/// It provides thread-safe access and automatic conflict resolution when merging updates.
///
/// # Thread Safety
///
/// `Graph` is thread-safe and uses `parking_lot::RwLock` for concurrent access.
/// Multiple threads can read simultaneously, or a single thread can write exclusively.
///
/// # Conflict Resolution
///
/// When merging nodes, the graph uses the HAM algorithm:
/// - Compare state timestamps for each property
/// - Higher state wins
/// - Merge non-conflicting properties
///
/// # Example
///
/// ```rust,no_run
/// use gun::graph::Graph;
/// use gun::state::Node;
///
/// let graph = Graph::new();
/// let node = Node::with_soul("user_123".to_string());
/// graph.put("user_123", node)?;
///
/// if let Some(loaded_node) = graph.get("user_123") {
///     println!("Found node: {:?}", loaded_node);
/// }
/// ```
#[derive(Clone)]
pub struct Graph {
    nodes: Arc<RwLock<HashMap<String, Node>>>,
}

impl Graph {
    /// Create a new empty graph
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get a node by its soul (unique identifier)
    ///
    /// # Arguments
    /// * `soul` - The unique identifier of the node
    ///
    /// # Returns
    /// The node if found, or `None` if it doesn't exist.
    pub fn get(&self, soul: &str) -> Option<Node> {
        self.nodes.read().get(soul).cloned()
    }

    /// Store a node in the graph by its soul
    ///
    /// If a node with the same soul already exists, it will be overwritten.
    /// For conflict resolution, use [`merge`](Self::merge) instead.
    ///
    /// # Arguments
    /// * `soul` - The unique identifier for the node
    /// * `node` - The node to store
    ///
    /// # Returns
    /// `Ok(())` on success, or a `GunError` if something goes wrong.
    pub fn put(&self, soul: &str, node: Node) -> GunResult<()> {
        self.nodes.write().insert(soul.to_string(), node);
        Ok(())
    }

    /// Check if a node with the given soul exists in the graph
    ///
    /// # Arguments
    /// * `soul` - The unique identifier to check
    ///
    /// # Returns
    /// `true` if the node exists, `false` otherwise.
    pub fn has(&self, soul: &str) -> bool {
        self.nodes.read().contains_key(soul)
    }

    /// Get a copy of all nodes in the graph (for debugging/testing)
    ///
    /// **Warning**: This clones all nodes, which can be expensive for large graphs.
    /// Only use this for debugging or small datasets.
    ///
    /// # Returns
    /// A `HashMap` mapping soul to node for all nodes in the graph.
    pub fn all_nodes(&self) -> HashMap<String, Node> {
        self.nodes.read().clone()
    }

    /// Merge a node into the graph with automatic conflict resolution
    ///
    /// This method implements the HAM (Hypothetical Amnesia Machine) algorithm:
    /// - If the node doesn't exist, it's inserted
    /// - If the node exists, properties are merged based on state timestamps
    /// - Higher state always wins in conflicts
    /// - Non-conflicting properties are preserved
    ///
    /// # Arguments
    /// * `soul` - The unique identifier for the node
    /// * `incoming` - The node to merge in
    /// * `state_fn` - Function that generates the current state timestamp
    ///
    /// # Returns
    /// The merged node after conflict resolution.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use gun::graph::Graph;
    /// use gun::state::{Node, State};
    /// use serde_json::json;
    ///
    /// let graph = Graph::new();
    /// let state = State::new();
    ///
    /// let mut node1 = Node::with_soul("user_123".to_string());
    /// node1.data.insert("name".to_string(), json!("Alice"));
    ///
    /// let merged = graph.merge("user_123", &node1, || state.next())?;
    /// ```
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
        let _current_state = state_fn();

        // Get states from both nodes
        let existing_states = existing
            .meta
            .get(">")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_else(serde_json::Map::new);

        let incoming_states = incoming
            .meta
            .get(">")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_else(serde_json::Map::new);

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
                    // from_f64 can fail if incoming_state is NaN or Infinity
                    // In practice, incoming_state should always be a valid finite number
                    if let Some(number) = serde_json::Number::from_f64(incoming_state) {
                        map.insert(key.clone(), Value::Number(number));
                    } else {
                        // Log error but continue - this should never happen in practice
                        tracing::error!("Invalid incoming state number: {} (NaN or Infinity)", incoming_state);
                    }
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
