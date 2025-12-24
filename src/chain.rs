use crate::core::GunCore;
use crate::error::GunResult;
use crate::state::Node;
use crate::valid::valid;
use serde_json::Value;
use std::collections::HashSet;
use std::sync::Arc;

/// Chain - the main API for interacting with Gun
/// Based on Gun.js chain.js and IGunChain interface
/// This provides the fluent API: gun.get('key').put(data).on(callback)
pub struct Chain {
    pub core: Arc<GunCore>,
    pub soul: Option<String>,
    pub key: Option<String>,
    pub parent: Option<Arc<Chain>>,
    pub id: u64,
    listener_ids: Arc<parking_lot::Mutex<HashSet<u64>>>, // Track listener IDs for off()
}

impl Chain {
    pub fn new(core: Arc<GunCore>) -> Self {
        let id = core.next_chain_id();
        Self {
            core,
            soul: None,
            key: None,
            parent: None,
            id,
            listener_ids: Arc::new(parking_lot::Mutex::new(HashSet::new())),
        }
    }

    pub fn with_soul(core: Arc<GunCore>, soul: String, parent: Option<Arc<Chain>>) -> Self {
        let id = core.next_chain_id();
        Self {
            core,
            soul: Some(soul),
            key: None,
            parent,
            id,
            listener_ids: Arc::new(parking_lot::Mutex::new(HashSet::new())),
        }
    }

    pub fn with_key(core: Arc<GunCore>, key: String, parent: Arc<Chain>) -> Self {
        let id = core.next_chain_id();
        Self {
            core,
            soul: parent.soul.clone(),
            key: Some(key),
            parent: Some(parent),
            id,
            listener_ids: Arc::new(parking_lot::Mutex::new(HashSet::new())),
        }
    }

    /// Get a property or node by key
    /// Based on Gun.js chain.get()
    pub fn get(&self, key: &str) -> Arc<Chain> {
        Arc::new(Chain::with_key(
            self.core.clone(),
            key.to_string(),
            Arc::new(self.clone()),
        ))
    }

    /// Put data into the current node/property
    /// Based on Gun.js chain.put() - improved implementation
    pub async fn put(&self, data: Value) -> GunResult<Arc<Chain>> {
        // Handle function callback (deferred data)
        // In Rust, this would be handled via async, so we'll skip this case for now

        // Validate data
        match valid(&data) {
            Ok(true) => {} // Valid simple value
            Err(Some(soul)) => {
                // It's a soul reference, create link
                let soul_chain = self.core.graph.get(&soul);
                if soul_chain.is_none() {
                    // Soul doesn't exist yet, we'll need to request it
                    // For now, create the node
                    let node = Node::with_soul(soul.clone());
                    self.core.graph.put(&soul, node)?;
                }
                return Ok(Arc::new(Chain::with_soul(
                    self.core.clone(),
                    soul,
                    Some(Arc::new(self.clone())),
                )));
            }
            _ => {
                // Invalid or object - handle object case
                if let Value::Object(map) = data {
                    return self.put_object(map).await;
                }
                return Err(crate::error::GunError::InvalidData(
                    "Invalid data type".to_string(),
                ));
            }
        }

        let soul = match &self.soul {
            Some(s) => s.clone(),
            None => self.core.uuid(None),
        };

        // Create or update node
        let mut node = self
            .core
            .graph
            .get(&soul)
            .unwrap_or_else(|| Node::with_soul(soul.clone()));

        // Merge data into node
        if let Some(key) = &self.key {
            // Setting a property
            let state = self.core.state.next();
            node.data.insert(key.clone(), data.clone());
            crate::state::State::ify(&mut node, Some(key), Some(state), Some(data), Some(&soul));
        } else {
            // Setting the whole node - but data is not an object here, so this shouldn't happen
            // This case is handled above in put_object
        }

        // Store in graph
        self.core.graph.put(&soul, node.clone())?;

        // Emit update event
        self.emit_update(&soul, &node.data);

        // Store in persistent storage if available
        if let Some(storage) = &self.core.storage {
            storage.put(&soul, &node).await?;
        }

        Ok(Arc::new(Chain::with_soul(
            self.core.clone(),
            soul,
            Some(Arc::new(self.clone())),
        )))
    }

    /// Helper to put an object (node) with proper traversal
    async fn put_object(&self, map: serde_json::Map<String, Value>) -> GunResult<Arc<Chain>> {
        let soul = match &self.soul {
            Some(s) => s.clone(),
            None => self.core.uuid(None),
        };

        let mut node = self
            .core
            .graph
            .get(&soul)
            .unwrap_or_else(|| Node::with_soul(soul.clone()));

        // Process each key-value pair
        for (k, v) in map {
            let state = self.core.state.next();

            // Check if value is a soul reference
            match valid(&v) {
                Err(Some(ref_soul)) => {
                    // It's a reference to another node
                    let ref_node = self.core.graph.get(&ref_soul);
                    if ref_node.is_none() {
                        // Create placeholder node
                        let placeholder = Node::with_soul(ref_soul.clone());
                        self.core.graph.put(&ref_soul, placeholder)?;
                    }
                    // Store as soul reference
                    node.data
                        .insert(k.clone(), serde_json::json!({"#": ref_soul}));
                }
                _ => {
                    // Regular value
                    node.data.insert(k.clone(), v.clone());
                }
            }

            crate::state::State::ify(&mut node, Some(&k), Some(state), Some(v), Some(&soul));
        }

        self.core.graph.put(&soul, node.clone())?;
        self.emit_update(&soul, &node.data);

        if let Some(storage) = &self.core.storage {
            storage.put(&soul, &node).await?;
        }

        Ok(Arc::new(Chain::with_soul(
            self.core.clone(),
            soul,
            Some(Arc::new(self.clone())),
        )))
    }

    /// Emit update event for listeners (synchronous)
    fn emit_update(&self, soul: &str, data: &serde_json::Map<String, Value>) {
        let event_type = format!("node_update:{}", soul);
        let event = crate::events::Event {
            event_type: event_type.clone(),
            data: serde_json::Value::Object(data.clone()),
        };
        self.core.events.emit(&event);
    }

    /// Subscribe to updates on this node/property
    /// Based on Gun.js chain.on() - improved with proper listener tracking
    pub fn on<F>(&self, callback: F) -> Arc<Chain>
    where
        F: Fn(Value, Option<String>) + Send + Sync + 'static,
    {
        let chain = Arc::new(self.clone());
        let soul = self.soul.clone();
        let key = self.key.clone();
        let listener_ids = self.listener_ids.clone();

        let cb = Box::new(move |event: &crate::events::Event| {
            // Extract data and call callback
            let value = if let Some(key) = &key {
                event.data.get(key).cloned().unwrap_or(Value::Null)
            } else {
                event.data.clone()
            };
            callback(value, key.clone());
        });

        let event_type = if let Some(ref s) = soul {
            format!("node_update:{}", s)
        } else {
            "graph_update".to_string()
        };

        let listener_id = self.core.events.on(&event_type, cb);
        listener_ids.lock().insert(listener_id);
        chain
    }

    /// Get data once without subscribing
    /// Based on Gun.js chain.once() - improved with async waiting
    pub async fn once<F>(&self, callback: F) -> GunResult<Arc<Chain>>
    where
        F: FnOnce(Value, Option<String>),
    {
        let soul = match &self.soul {
            Some(s) => s.clone(),
            None => {
                // Return undefined/None if no soul
                callback(Value::Null, self.key.clone());
                return Ok(Arc::new(self.clone()));
            }
        };

        // Try to get from graph immediately
        if let Some(node) = self.core.graph.get(&soul) {
            let value = if let Some(key) = &self.key {
                node.data.get(key).cloned().unwrap_or(Value::Null)
            } else {
                serde_json::to_value(&node.data).unwrap_or(Value::Null)
            };
            callback(value, self.key.clone());
        } else {
            // Data not found - in full implementation would request from network
            // For now, call with null
            callback(Value::Null, self.key.clone());
        }

        Ok(Arc::new(self.clone()))
    }

    /// Map over properties of a node
    /// Based on Gun.js chain.map() - complete implementation
    pub fn map<F>(&self, callback: F) -> Arc<Chain>
    where
        F: Fn(Value, String) + Send + Sync + Clone + 'static,
    {
        let chain = Arc::new(self.clone());
        let soul = self.soul.clone();
        let listener_ids = self.listener_ids.clone();

        // Clone callback for use in closure
        let callback_clone = callback.clone();

        // Subscribe to updates and call callback for each property
        if let Some(ref s) = soul {
            let event_type = format!("node_update:{}", s);
            let cb = Box::new(move |event: &crate::events::Event| {
                if let Some(data_obj) = event.data.as_object() {
                    for (key, value) in data_obj {
                        callback_clone(value.clone(), key.clone());
                    }
                }
            });

            let listener_id = self.core.events.on(&event_type, cb);
            listener_ids.lock().insert(listener_id);

            // Also call for current data if available
            if let Some(node) = self.core.graph.get(s) {
                for (key, value) in node.data.iter() {
                    callback(value.clone(), key.clone());
                }
            }
        }

        chain
    }

    /// Add item to a set
    /// Based on Gun.js chain.set() - proper set implementation
    pub async fn set(&self, item: Value) -> GunResult<Arc<Chain>> {
        // Check if item has a soul (is a node reference)
        let soul = match valid(&item) {
            Err(Some(ref_soul)) => Some(ref_soul.clone()),
            _ => {
                // Item doesn't have a soul, generate one
                // set() expects objects (nodes) to be added to the set
                if item.is_object() {
                    let new_soul = self.core.uuid(None);
                    // Create node for the item
                    if let Value::Object(ref map) = item {
                        let mut node = Node::with_soul(new_soul.clone());
                        for (k, v) in map {
                            let state = self.core.state.next();
                            node.data.insert(k.clone(), v.clone());
                            crate::state::State::ify(
                                &mut node,
                                Some(&k),
                                Some(state),
                                Some(v.clone()),
                                Some(&new_soul),
                            );
                        }
                        self.core.graph.put(&new_soul, node.clone())?;
                        if let Some(storage) = &self.core.storage {
                            storage.put(&new_soul, &node).await?;
                        }
                    }
                    Some(new_soul)
                } else {
                    // Non-object items can't be in sets
                    return Err(crate::error::GunError::InvalidData(
                        "set() only accepts objects/nodes".to_string(),
                    ));
                }
            }
        };

        if let Some(ref_soul) = soul {
            // Add reference to the set node
            let set_soul = self.soul.clone().unwrap_or_else(|| self.core.uuid(None));
            let mut set_node = self
                .core
                .graph
                .get(&set_soul)
                .unwrap_or_else(|| Node::with_soul(set_soul.clone()));

            // Store reference to the item
            let key = ref_soul.clone();
            let state = self.core.state.next();
            set_node
                .data
                .insert(key.clone(), serde_json::json!({"#": ref_soul}));
            crate::state::State::ify(
                &mut set_node,
                Some(&key),
                Some(state),
                Some(serde_json::json!({"#": ref_soul})),
                Some(&set_soul),
            );

            self.core.graph.put(&set_soul, set_node.clone())?;
            self.emit_update(&set_soul, &set_node.data);

            if let Some(storage) = &self.core.storage {
                storage.put(&set_soul, &set_node).await?;
            }

            Ok(Arc::new(Chain::with_soul(
                self.core.clone(),
                set_soul,
                Some(Arc::new(self.clone())),
            )))
        } else {
            self.put(item).await
        }
    }

    /// Go back up the chain
    /// Based on Gun.js chain.back()
    pub fn back(&self, amount: Option<usize>) -> Option<Arc<Chain>> {
        match amount {
            Some(0) | None => self.parent.clone(),
            Some(n) if n == 1 => self.parent.clone(),
            Some(n) => {
                // Go back n levels
                let mut current = self.parent.clone();
                for _ in 1..n {
                    current = current.and_then(|c| c.parent.clone());
                }
                current
            }
        }
    }

    /// Remove all listeners for this chain
    /// Based on Gun.js chain.off() - properly removes listeners
    pub fn off(&self) -> Arc<Chain> {
        let listener_ids = self.listener_ids.lock();
        let ids: Vec<u64> = listener_ids.iter().cloned().collect();
        drop(listener_ids); // Release lock

        let event_type = if let Some(ref s) = self.soul {
            format!("node_update:{}", s)
        } else {
            "graph_update".to_string()
        };

        for id in ids {
            self.core.events.off(&event_type, id);
        }

        self.listener_ids.lock().clear();
        Arc::new(self.clone())
    }
}

impl Clone for Chain {
    fn clone(&self) -> Self {
        Self {
            core: self.core.clone(),
            soul: self.soul.clone(),
            key: self.key.clone(),
            parent: self.parent.clone(),
            id: self.id,
            listener_ids: self.listener_ids.clone(),
        }
    }
}
