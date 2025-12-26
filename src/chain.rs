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
            soul: None, // Don't inherit parent's soul - each get() creates a new chain without a soul
                       // The soul will be generated in put_object() when self.soul is None
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

        // Check for content addressing (hash verification for #hash souls)
        if let Some(ref soul) = self.soul {
            if soul.starts_with('#') {
                // Content addressing: verify hash matches content
                let hash_suffix = soul.strip_prefix('#').unwrap_or(soul); // Remove '#' prefix
                let data_string = serde_json::to_string(&data)
                    .map_err(crate::error::GunError::Serialization)?;
                let hash_result = crate::sea::work(
                    data_string.as_bytes(),
                    None,
                    crate::sea::WorkOptions {
                        name: Some("SHA-256".to_string()),
                        encode: Some("base64".to_string()),
                        ..Default::default()
                    },
                )
                .await
                .map_err(|e| crate::error::GunError::Crypto(e.to_string()))?;

                // Compare hash (support both base64 and hex)
                use base64::Engine as _;
                let hash_matches = hash_result == hash_suffix
                    || hex::encode(
                        base64::engine::general_purpose::STANDARD_NO_PAD
                            .decode(&hash_result)
                            .unwrap_or_default(),
                    ) == hash_suffix;

                if !hash_matches {
                    return Err(crate::error::GunError::InvalidData(format!(
                        "Content hash mismatch: expected {}, got {}",
                        hash_suffix, hash_result
                    )));
                }
            }
        }

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

        // If we have a key but no soul, store the value in the parent node
        if let Some(key) = &self.key {
            if let Some(parent) = &self.parent {
                // Try to resolve parent soul
                let mut current_parent: Option<&Chain> = Some(parent.as_ref());
                let mut found_parent_soul: Option<String> = None;
                
                while let Some(p) = current_parent {
                    if let Some(ps) = &p.soul {
                        found_parent_soul = Some(ps.clone());
                        break;
                    }
                    current_parent = p.parent.as_deref();
                }
                
                if let Some(parent_soul) = found_parent_soul {
                    // Store primitive value directly in parent node
                    let mut parent_node = self.core.graph.get(&parent_soul)
                        .unwrap_or_else(|| Node::with_soul(parent_soul.clone()));
                    let state = self.core.state.next();
                    parent_node.data.insert(key.clone(), data.clone());
                    crate::state::State::ify(&mut parent_node, Some(key), Some(state), Some(data.clone()), Some(&parent_soul));
                    self.core.graph.put(&parent_soul, parent_node.clone())?;
                    self.emit_update(&parent_soul, &parent_node.data);
                    
                    // Store in persistent storage if available
                    if let Some(storage) = &self.core.storage {
                        storage.put(&parent_soul, &parent_node).await?;
                    }
                    
                    return Ok(Arc::new(self.clone()));
                }
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
        // Parse soul and check for expiration (<? suffix)
        let (soul, expiration_seconds) = match &self.soul {
            Some(s) => {
                // Check for expiration suffix: soul<?3600 (expires after 3600 seconds)
                if let Some(exp_pos) = s.find("<?") {
                    let soul_part = &s[..exp_pos];
                    let exp_part = &s[exp_pos + 2..];
                    if let Ok(exp_secs) = exp_part.parse::<f64>() {
                        (soul_part.to_string(), Some(exp_secs))
                    } else {
                        (s.clone(), None)
                    }
                } else {
                    (s.clone(), None)
                }
            }
            None => (self.core.uuid(None), None),
        };

        // Check expiration if present (for object puts, we check all keys)
        if let Some(exp_secs) = expiration_seconds {
            let now = chrono::Utc::now().timestamp_millis() as f64;
            let expiration_time = now - (exp_secs * 1000.0);
            if let Some(node) = self.core.graph.get(&soul) {
                for key in map.keys() {
                    if let Some(state) = crate::state::State::is(&Some(node.clone()), key) {
                        if state < expiration_time {
                            // Data is expired, reject
                            return Err(crate::error::GunError::InvalidData(format!(
                                "Data expired for key {}: state {} is older than expiration {}",
                                key, state, expiration_time
                            )));
                        }
                    }
                }
            }
        }

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
                        // Create placeholder node when soul reference doesn't exist yet
                        // This matches Gun.js behavior: creating a reference to a non-existent node
                        // creates a placeholder that can be filled in later when the actual node is received
                        let placeholder = Node::with_soul(ref_soul.clone());
                        self.core.graph.put(&ref_soul, placeholder)?;
                    }
                    // Store as soul reference
                    node.data
                        .insert(k.clone(), serde_json::json!({"#": ref_soul}));
                }
                _ => {
                    // Regular value - but if it's an object, we should store it as-is
                    // Nested objects are stored directly in the node, not as separate nodes
                    // This allows get("level1") to work by extracting from the parent node
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

        // If we have a key, we need to store the soul reference in the parent node
        // This allows once() to find the data later via path resolution
        if let Some(key) = &self.key {
            if let Some(parent) = &self.parent {
                eprintln!("DEBUG: put_object() storing soul reference: key={}, soul={}, parent_soul={:?}", key, soul, parent.soul);
                // Try to resolve or create parent node
                if let Some(parent_soul) = &parent.soul {
                    // Parent has a soul, store reference there
                    // Create parent node if it doesn't exist
                    let mut parent_node = self.core.graph.get(parent_soul)
                        .unwrap_or_else(|| Node::with_soul(parent_soul.clone()));
                    let state = self.core.state.next();
                    let soul_ref = serde_json::json!({"#": soul});
                    eprintln!("DEBUG: Storing soul reference in parent: parent_soul={}, key={}, soul_ref={}", parent_soul, key, serde_json::to_string(&soul_ref).unwrap_or_default());
                    parent_node.data.insert(key.clone(), soul_ref.clone());
                    crate::state::State::ify(&mut parent_node, Some(key), Some(state), Some(soul_ref), Some(parent_soul));
                    self.core.graph.put(parent_soul, parent_node.clone())?;
                    self.emit_update(parent_soul, &parent_node.data);
                } else {
                    // Parent has no soul - create one for it
                    // Use the parent's key if available, otherwise use a deterministic approach
                    let parent_soul = if let Some(parent_key) = &parent.key {
                        // Parent has a key - use it to create deterministic soul
                        use sha2::{Sha256, Digest};
                        use base64::{engine::general_purpose, Engine as _};
                        let mut hasher = Sha256::new();
                        hasher.update(parent_key.as_bytes());
                        let hash = general_purpose::STANDARD_NO_PAD.encode(hasher.finalize());
                        format!("root_{}", hash)
                    } else {
                        // Parent has no key - generate a new soul
                        self.core.uuid(None)
                    };
                    
                    // Get or create parent node
                    let mut parent_node = self.core.graph.get(&parent_soul)
                        .unwrap_or_else(|| Node::with_soul(parent_soul.clone()));
                    
                    // Store the soul reference in the parent node
                    let state = self.core.state.next();
                    let soul_ref = serde_json::json!({"#": soul});
                    parent_node.data.insert(key.clone(), soul_ref.clone());
                    crate::state::State::ify(&mut parent_node, Some(key), Some(state), Some(soul_ref), Some(&parent_soul));
                    self.core.graph.put(&parent_soul, parent_node.clone())?;
                    self.emit_update(&parent_soul, &parent_node.data);
                }
            }
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
        
        // Also emit graph_update for listeners that don't have a specific soul yet
        self.core.events.emit(&crate::events::Event {
            event_type: "graph_update".to_string(),
            data: serde_json::Value::Object(data.clone()),
        });
        
        // Also emit network_sync event for Gun to handle
        let network_event = crate::events::Event {
            event_type: "network_sync".to_string(),
            data: serde_json::json!({
                "soul": soul,
                "data": serde_json::Value::Object(data.clone())
            }),
        };
        self.core.events.emit(&network_event);
    }

    /// Subscribe to updates on this node/property
    /// Based on Gun.js chain.on() - enhanced with change detection and network sync
    /// 
    /// Subscribes to real-time updates for this node or property. The callback will be called:
    /// - Immediately with current data if available
    /// - Whenever the data changes (with change detection to avoid duplicate updates)
    /// - When data is received from the network
    /// 
    /// This method includes:
    /// - Change detection: Only triggers callback when data actually changes
    /// - Network synchronization: Triggers sync requests when local data changes
    /// - Real-time propagation: Receives updates from network peers
    /// 
    /// # Arguments
    /// * `callback` - Closure that receives updates
    ///   - First parameter: The updated data value
    ///   - Second parameter: The key if this is a property access, `None` if node access
    /// 
    /// # Returns
    /// Returns `Arc<Chain>` for method chaining. Use `off()` to unsubscribe.
    /// 
    /// # Example
    /// ```rust,no_run
    /// use gun::Gun;
    /// use serde_json::json;
    /// 
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let gun = Gun::new();
    /// 
    /// // Subscribe to updates
    /// let chain = gun.get("counter");
    /// chain.on(|data, _key| {
    ///     if let Some(value) = data.as_i64() {
    ///         println!("Counter updated: {}", value);
    ///     }
    /// });
    /// 
    /// // Updates will trigger the callback
    /// chain.put(json!(1)).await?;
    /// chain.put(json!(2)).await?;
    /// 
    /// // Unsubscribe when done
    /// chain.off();
    /// # Ok(())
    /// # }
    /// ```
    pub fn on<F>(&self, callback: F) -> Arc<Chain>
    where
        F: Fn(Value, Option<String>) + Send + Sync + Clone + 'static,
    {
        let chain = Arc::new(self.clone());
        let soul = self.soul.clone();
        let key = self.key.clone();
        let listener_ids = self.listener_ids.clone();
        let core = self.core.clone();

        // Store previous value for change detection
        let prev_value: Arc<parking_lot::Mutex<Option<Value>>> = Arc::new(parking_lot::Mutex::new(None));
        
        // Try to resolve soul from path if we don't have one (similar to once())
        // For on(), we always listen to parent node updates if we have a key but no soul
        let resolved_soul = if let Some(ref s) = &soul {
            s.clone()
        } else if let Some(ref k) = &key {
            // Try to resolve path by checking parent node
            if let Some(parent) = &self.parent {
                // Walk up the parent chain to find a node with a soul
                let mut current_parent: Option<&Chain> = Some(parent.as_ref());
                let mut found_parent_soul: Option<String> = None;
                
                while let Some(p) = current_parent {
                    if let Some(ps) = &p.soul {
                        found_parent_soul = Some(ps.clone());
                        break;
                    }
                    current_parent = p.parent.as_deref();
                }
                
                if let Some(parent_soul) = found_parent_soul {
                    // Listen to parent node updates - when parent updates, check if our key changed
                    parent_soul
                } else {
                    // Parent has no soul - use generic event
                    // The parent will get a soul when data is put, and we'll receive updates then
                    "graph_update".to_string()
                }
            } else {
                // No parent - use generic event
                "graph_update".to_string()
            }
        } else {
            // No key - use generic event
            "graph_update".to_string()
        };
        
        // Determine event type
        let event_type = if resolved_soul != "graph_update" {
            format!("node_update:{}", resolved_soul)
        } else {
            "graph_update".to_string()
        };
        
        // Call callback with current data if available (before setting up listener)
        if resolved_soul != "graph_update" {
            if let Some(node) = self.core.graph.get(&resolved_soul) {
                if let Some(ref k) = &key {
                    if let Some(value) = node.data.get(k) {
                        callback(value.clone(), Some(k.clone()));
                        *prev_value.lock() = Some(value.clone());
                    }
                } else {
                    let node_data = serde_json::to_value(&node.data).unwrap_or(Value::Null);
                    callback(node_data.clone(), None);
                    *prev_value.lock() = Some(node_data);
                }
            } else if let Some(ref k) = &key {
                // Check parent node for the key
                if let Some(parent) = &self.parent {
                    // Try to resolve parent soul by walking up the chain
                    let mut current_parent: Option<&Chain> = Some(parent.as_ref());
                    let mut found_parent_soul: Option<String> = None;
                    
                    while let Some(p) = current_parent {
                        if let Some(ps) = &p.soul {
                            found_parent_soul = Some(ps.clone());
                            break;
                        }
                        current_parent = p.parent.as_deref();
                    }
                    
                    if let Some(parent_soul) = found_parent_soul {
                        if let Some(parent_node) = self.core.graph.get(&parent_soul) {
                            if let Some(value) = parent_node.data.get(k) {
                                // Check if it's a soul reference or nested object
                                if let Some(obj) = value.as_object() {
                                    if let Some(soul_ref) = obj.get("#") {
                                        if let Some(soul_str) = soul_ref.as_str() {
                                            if let Some(ref_node) = self.core.graph.get(soul_str) {
                                                let node_data = serde_json::to_value(&ref_node.data).unwrap_or(Value::Null);
                                                callback(node_data.clone(), Some(k.clone()));
                                                *prev_value.lock() = Some(node_data);
                                            } else {
                                                callback(value.clone(), Some(k.clone()));
                                                *prev_value.lock() = Some(value.clone());
                                            }
                                        } else {
                                            callback(value.clone(), Some(k.clone()));
                                            *prev_value.lock() = Some(value.clone());
                                        }
                                    } else {
                                        // Nested object
                                        callback(value.clone(), Some(k.clone()));
                                        *prev_value.lock() = Some(value.clone());
                                    }
                                } else {
                                    callback(value.clone(), Some(k.clone()));
                                    *prev_value.lock() = Some(value.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Now set up listener with cloned values (all values cloned before move)
        let key_for_cb = key.clone();
        let prev_value_for_cb = prev_value.clone();
        let resolved_soul_for_cb = if resolved_soul != "graph_update" {
            Some(resolved_soul.clone())
        } else {
            None
        };
        
        // Clone callback for use in closure
        let callback_for_cb = callback.clone();

        let core_for_resolve = core.clone();
        // Build the key path for nested extraction
        let mut key_path = Vec::new();
        if let Some(ref k) = &key_for_cb {
            key_path.push(k.clone());
            // Walk up the parent chain to collect all keys in the path
            // This handles cases like get("level1").get("level2")
            // Note: The current implementation only stores one key per chain,
            // so for deeply nested paths, we need to reconstruct the path
            // For now, we'll handle single-level nesting in the callback
        }
        
        let cb = Box::new(move |event: &crate::events::Event| {
            // Extract data and detect changes
            let new_value = if let Some(ref k) = &key_for_cb {
                // If we have a key, check if it's in the event data (parent node update)
                if let Some(data_obj) = event.data.as_object() {
                    if let Some(value) = data_obj.get(k) {
                        // Check if it's a soul reference - if so, resolve it
                        if let Some(obj) = value.as_object() {
                            if let Some(soul_ref) = obj.get("#") {
                                if let Some(soul_str) = soul_ref.as_str() {
                                    // It's a soul reference - get the actual node data
                                    if let Some(node) = core_for_resolve.graph.get(soul_str) {
                                        // For a property access, if node has only one key, return that value
                                        // Otherwise return the whole node as object
                                        if node.data.len() == 1 {
                                            node.data.values().next().cloned().unwrap_or(Value::Null)
                                        } else {
                                            serde_json::to_value(&node.data).unwrap_or(Value::Null)
                                        }
                                    } else {
                                        // Node not found yet - return the soul reference for now
                                        value.clone()
                                    }
                                } else {
                                    value.clone()
                                }
                            } else {
                                // Not a soul reference - could be nested object
                                // For deeply nested paths like get("level1").get("level2"),
                                // we need to check if the parent chain has a child key
                                // For now, return the nested object and let the parent chain handle extraction
                                value.clone()
                            }
                        } else {
                            // Primitive value
                            value.clone()
                        }
                    } else {
                        Value::Null
                    }
                } else {
                    Value::Null
                }
            } else {
                event.data.clone()
            };
            
            // Check if value has changed (including null changes)
            let mut prev = prev_value_for_cb.lock();
            let has_changed = prev.as_ref()
                .map(|pv| pv != &new_value)
                .unwrap_or(true);
            
            // For on(), call callback when value changes (even if null)
            if has_changed {
                callback_for_cb(new_value.clone(), key_for_cb.clone());
                *prev = Some(new_value.clone());
            }
        });

        let listener_id = self.core.events.on(&event_type, cb);
        listener_ids.lock().insert(listener_id);
        
        // Also try removing from graph_update as fallback when off() is called
        // Store the event type we used (we'll try both in off())
        
        chain
    }

    /// Get data once without subscribing
    /// Based on Gun.js chain.once() - improved with async waiting and network requests
    /// 
    /// Retrieves data once without creating a subscription. If data is not found locally,
    /// sends a network request via DAM protocol and waits for response with timeout (default 5 seconds).
    /// 
    /// # Arguments
    /// * `callback` - Closure that receives the data and optional key
    ///   - First parameter: The data value (or `Value::Null` if not found)
    ///   - Second parameter: The key if this is a property access, `None` if node access
    /// 
    /// # Returns
    /// Returns `Ok(Arc<Chain>)` for method chaining. The callback is called with:
    /// - The data if found locally or received from network
    /// - `Value::Null` if data not found and network request times out or fails
    /// 
    /// # Errors
    /// Returns `GunError` if there's an error during the operation
    /// 
    /// # Example
    /// ```rust,no_run
    /// use gun::Gun;
    /// use serde_json::json;
    /// 
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let gun = Gun::new();
    /// 
    /// // Put data first
    /// gun.get("user").put(json!({"name": "Alice"})).await?;
    /// 
    /// // Read once
    /// gun.get("user").once(|data, _key| {
    ///     if let Some(obj) = data.as_object() {
    ///         println!("Name: {:?}", obj.get("name"));
    ///     }
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn once<F>(&self, callback: F) -> GunResult<Arc<Chain>>
    where
        F: FnOnce(Value, Option<String>),
    {
        // Try to resolve soul from path if we don't have one
        let mut resolved_soul_opt: Option<String> = None;
        let soul = match &self.soul {
            Some(s) => s.clone(),
            None => {
                // Try to resolve path by checking if we can find data through the parent chain
                // This handles cases like: gun.get("test").get("read_test").put(obj).once(...)
                if let Some(key) = &self.key {
                    if let Some(parent) = &self.parent {
                        // Try to find the soul by looking in parent's node
                        if let Some(parent_soul) = &parent.soul {
                            if let Some(parent_node) = self.core.graph.get(parent_soul) {
                                if let Some(value) = parent_node.data.get(key) {
                                    // Check if it's a soul reference
                                    if let Some(obj) = value.as_object() {
                                        if let Some(soul_ref) = obj.get("#") {
                                            if let Some(soul_str) = soul_ref.as_str() {
                                                // Found a soul reference, use it
                                                let resolved_soul = soul_str.to_string();
                                                eprintln!("DEBUG: once() resolved path to soul: {}", resolved_soul);
                                                // Continue with the resolved soul
                                                if let Some(node) = self.core.graph.get(&resolved_soul) {
                                                    let node_data = serde_json::to_value(&node.data).unwrap_or(Value::Null);
                                                    eprintln!("DEBUG: once() found node locally, calling callback with data");
                                                    callback(node_data, self.key.clone());
                                                    return Ok(Arc::new(self.clone()));
                                                } else {
                                                    eprintln!("DEBUG: once() resolved soul {} but node not found locally, will request from network", resolved_soul);
                                                    // Store resolved soul for network request
                                                    resolved_soul_opt = Some(resolved_soul);
                                                }
                                            } else {
                                                // Object but no "#" key - it's a nested object, return it directly
                                                eprintln!("DEBUG: once() found nested object in parent node");
                                                callback(value.clone(), self.key.clone());
                                                return Ok(Arc::new(self.clone()));
                                            }
                                        } else {
                                            // Object but no "#" key - it's a nested object, return it directly
                                            eprintln!("DEBUG: once() found nested object in parent node");
                                            callback(value.clone(), self.key.clone());
                                            return Ok(Arc::new(self.clone()));
                                        }
                                    } else {
                                        // Not a soul reference - could be a nested object or primitive value
                                        // Return the value directly
                                        eprintln!("DEBUG: once() found value directly in parent node (not a soul reference): {:?}", if value.is_object() { "object" } else { "primitive" });
                                        callback(value.clone(), self.key.clone());
                                        return Ok(Arc::new(self.clone()));
                                    }
                                } else {
                                    eprintln!("DEBUG: once() key {} not found in parent node {}, will request from network", key, parent_soul);
                                    // Key not found in parent - might be nested deeper, need to wait for network
                                }
                            } else {
                                eprintln!("DEBUG: once() parent node {} not found, will request parent node from network first", parent_soul);
                                // Request parent node first
                                let get_request = serde_json::json!({
                                    "get": {
                                        "#": parent_soul
                                    }
                                });
                                self.core.events.emit(&crate::events::Event {
                                    event_type: "get_request".to_string(),
                                    data: get_request,
                                });
                            }
                        } else {
                            // Parent has no soul - try to resolve it by walking up the chain
                            // or by looking for the parent node in the graph
                            eprintln!("DEBUG: once() parent has no soul, trying to resolve parent node");
                            
                            // Walk up the parent chain to find a node with a soul
                            let mut current_parent: Option<&Chain> = Some(parent.as_ref());
                            let mut found_parent_soul: Option<String> = None;
                            
                            while let Some(p) = current_parent {
                                if let Some(ps) = &p.soul {
                                    found_parent_soul = Some(ps.clone());
                                    break;
                                }
                                // Try grandparent
                                current_parent = p.parent.as_deref();
                            }
                            
                            // If we found a parent soul, try to get the value from that node
                            if let Some(parent_soul) = found_parent_soul {
                                if let Some(parent_node) = self.core.graph.get(&parent_soul) {
                                    // Check if parent has our key
                                    if let Some(key) = &self.key {
                                        // First check if parent has the key directly (nested object in parent)
                                        if let Some(value) = parent_node.data.get(key) {
                                            // Check if it's a soul reference or nested object
                                            if let Some(obj) = value.as_object() {
                                                if obj.get("#").is_some() {
                                                    // It's a soul reference - get that node and extract the key from it
                                                    if let Some(soul_str) = obj.get("#").and_then(|v| v.as_str()) {
                                                        if let Some(ref_node) = self.core.graph.get(soul_str) {
                                                            // The key might be in the referenced node
                                                            if let Some(nested_value) = ref_node.data.get(key) {
                                                                eprintln!("DEBUG: once() found key '{}' in referenced node {}", key, soul_str);
                                                                callback(nested_value.clone(), self.key.clone());
                                                                return Ok(Arc::new(self.clone()));
                                                            } else {
                                                                // Return the whole node data
                                                                let node_data = serde_json::to_value(&ref_node.data).unwrap_or(Value::Null);
                                                                eprintln!("DEBUG: once() found soul reference '{}' in resolved parent node {}, returning node data", key, parent_soul);
                                                                callback(node_data, self.key.clone());
                                                                return Ok(Arc::new(self.clone()));
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    // It's a nested object - return it directly
                                                    eprintln!("DEBUG: once() extracting nested object '{}' from resolved parent node {}", key, parent_soul);
                                                    callback(value.clone(), self.key.clone());
                                                    return Ok(Arc::new(self.clone()));
                                                }
                                            } else {
                                                // Primitive value - return it
                                                eprintln!("DEBUG: once() extracting primitive '{}' from resolved parent node {}", key, parent_soul);
                                                callback(value.clone(), self.key.clone());
                                                return Ok(Arc::new(self.clone()));
                                            }
                                        }
                                        
                                        // Key not found directly in parent - check if parent has a soul reference to a node that might contain it
                                        // Walk through all values in parent node to find soul references
                                        for (parent_key, parent_value) in &parent_node.data {
                                            if let Some(obj) = parent_value.as_object() {
                                                if let Some(soul_ref) = obj.get("#") {
                                                    if let Some(soul_str) = soul_ref.as_str() {
                                                        // Found a soul reference - check if the referenced node has our key
                                                        if let Some(ref_node) = self.core.graph.get(soul_str) {
                                                            if let Some(nested_value) = ref_node.data.get(key) {
                                                                eprintln!("DEBUG: once() found key '{}' in node {} referenced by parent key '{}'", key, soul_str, parent_key);
                                                                callback(nested_value.clone(), self.key.clone());
                                                                return Ok(Arc::new(self.clone()));
                                                            }
                                                            
                                                            // Also check if any value in the referenced node is a nested object containing our key
                                                            // This handles deeply nested paths like level1.level2.level3
                                                            for (ref_key, ref_value) in &ref_node.data {
                                                                if let Some(nested_obj) = ref_value.as_object() {
                                                                    // Check if this nested object has our key (for deeply nested like level1.level2.level3)
                                                                    if nested_obj.get("#").is_none() {
                                                                        // Recursively search nested objects
                                                                        fn find_in_nested(obj: &serde_json::Map<String, serde_json::Value>, search_key: &str) -> Option<serde_json::Value> {
                                                                            if let Some(val) = obj.get(search_key) {
                                                                                return Some(val.clone());
                                                                            }
                                                                            for (_, val) in obj {
                                                                                if let Some(nested) = val.as_object() {
                                                                                    if nested.get("#").is_none() {
                                                                                        if let Some(found) = find_in_nested(nested, search_key) {
                                                                                            return Some(found);
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                            None
                                                                        }
                                                                        
                                                                        if let Some(deep_value) = find_in_nested(nested_obj, key) {
                                                                            eprintln!("DEBUG: once() found deeply nested key '{}' in nested object '{}' in node {}", key, ref_key, soul_str);
                                                                            callback(deep_value.clone(), self.key.clone());
                                                                            return Ok(Arc::new(self.clone()));
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    // Not a soul reference - could be a nested object
                                                    // Check if this nested object contains our key (for cases like level1.level2)
                                                    if let Some(nested_obj) = parent_value.as_object() {
                                                        if nested_obj.get(key).is_some() {
                                                            if let Some(deep_value) = nested_obj.get(key) {
                                                                eprintln!("DEBUG: once() found key '{}' in nested object '{}' in parent node {}", key, parent_key, parent_soul);
                                                                callback(deep_value.clone(), self.key.clone());
                                                                return Ok(Arc::new(self.clone()));
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        
                                        eprintln!("DEBUG: once() key '{}' not found in resolved parent node {} or its referenced nodes", key, parent_soul);
                                    }
                                } else {
                                    eprintln!("DEBUG: once() resolved parent soul {} but node not found in graph", parent_soul);
                                }
                            } else {
                                eprintln!("DEBUG: once() could not resolve parent soul by walking up chain");
                            }
                            
                            eprintln!("DEBUG: once() parent has no soul, will try to wait for network data");
                        }
                    }
                }
                // Could not resolve path - wait for network data instead of returning immediately
                // Use resolved soul if we found one, otherwise generate a new one
                resolved_soul_opt.unwrap_or_else(|| self.core.uuid(None));
                // The data might be syncing from another client, so we should wait
                // We'll use a generic listener that waits for any update to the parent path
                if let Some(key) = &self.key {
                    if let Some(parent) = &self.parent {
                        // Set up a listener for parent updates that might contain our key
                        let key_clone = key.clone();
                        let callback_clone = callback;
                        let parent_soul_opt = parent.soul.clone();
                        
                        // Wait for parent node to be updated with our key
                        let data_received: Arc<parking_lot::Mutex<Option<serde_json::Value>>> = Arc::new(parking_lot::Mutex::new(None));
                        let data_received_clone = data_received.clone();
                        let data_ready: Arc<parking_lot::Mutex<bool>> = Arc::new(parking_lot::Mutex::new(false));
                        let data_ready_clone = data_ready.clone();
                        
                        // Listen for parent node updates
                        let event_type = if let Some(ref ps) = parent_soul_opt {
                            format!("node_update:{}", ps)
                        } else {
                            "graph_update".to_string()
                        };
                        
                        let listener_id = self.core.events.on(&event_type, Box::new(move |event: &crate::events::Event| {
                            if let Some(data_obj) = event.data.as_object() {
                                if let Some(value) = data_obj.get(&key_clone) {
                                    // Found our key in the update
                                    *data_received_clone.lock() = Some(value.clone());
                                    *data_ready_clone.lock() = true;
                                }
                            }
                        }));
                        
                        // Also check if data is already available
                        if let Some(ref ps) = parent_soul_opt {
                            if let Some(parent_node) = self.core.graph.get(ps) {
                                if let Some(value) = parent_node.data.get(key) {
                                    // Check if it's a soul reference
                                    if let Some(obj) = value.as_object() {
                                        if let Some(soul_ref) = obj.get("#") {
                                            if let Some(soul_str) = soul_ref.as_str() {
                                                // Found a soul reference, get the node
                                                if let Some(node) = self.core.graph.get(soul_str) {
                                                    let node_data = serde_json::to_value(&node.data).unwrap_or(Value::Null);
                                                    self.core.events.off(&event_type, listener_id);
                                                    callback_clone(node_data, Some(key.clone()));
                                                    return Ok(Arc::new(self.clone()));
                                                }
                                            }
                                        }
                                    }
                                    // Not a soul reference - could be nested object or primitive
                                    // Return the value directly
                                    self.core.events.off(&event_type, listener_id);
                                    callback_clone(value.clone(), Some(key.clone()));
                                    return Ok(Arc::new(self.clone()));
                                }
                            }
                        }
                        
                        // Wait for data with longer timeout (15 seconds for network sync)
                        let timeout_duration = tokio::time::Duration::from_secs(15);
                        let start = std::time::Instant::now();
                        
                        loop {
                            if start.elapsed() > timeout_duration {
                                break; // Timeout
                            }
                            
                            let ready = *data_ready.lock();
                            if ready {
                                break; // Data received
                            }
                            
                            // Also check periodically if data arrived
                            if let Some(ref ps) = parent_soul_opt {
                                if let Some(parent_node) = self.core.graph.get(ps) {
                                    if let Some(value) = parent_node.data.get(key) {
                                        *data_received.lock() = Some(value.clone());
                                        *data_ready.lock() = true;
                                        break;
                                    }
                                }
                            }
                            
                            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                        }
                        
                        // Remove listener
                        self.core.events.off(&event_type, listener_id);
                        
                        // Process result
                        let received_data = data_received.lock().take();
                        let value = received_data.unwrap_or(Value::Null);
                        callback_clone(value, Some(key.clone()));
                        return Ok(Arc::new(self.clone()));
                    }
                }
                // No parent and no soul - return null immediately
                callback(Value::Null, self.key.clone());
                return Ok(Arc::new(self.clone()));
            }
        };

        // Try to get from graph immediately
        // First check if we have a key and can extract from parent (for nested objects)
        // This handles cases where parent chain has no soul but parent node exists
        if let Some(key) = &self.key {
            if let Some(parent) = &self.parent {
                // Try to find parent node even if parent chain has no soul
                // Walk up the parent chain to find a node with a soul
                let mut current_parent: Option<&Chain> = Some(parent.as_ref());
                while let Some(p) = current_parent {
                    if let Some(parent_soul) = &p.soul {
                        if let Some(parent_node) = self.core.graph.get(parent_soul) {
                            if let Some(value) = parent_node.data.get(key) {
                                // Check if it's a soul reference or nested object
                                if let Some(obj) = value.as_object() {
                                    if obj.get("#").is_some() {
                                        // It's a soul reference - try to get that node
                                        if let Some(soul_str) = obj.get("#").and_then(|v| v.as_str()) {
                                            if let Some(ref_node) = self.core.graph.get(soul_str) {
                                                let node_data = serde_json::to_value(&ref_node.data).unwrap_or(Value::Null);
                                                eprintln!("DEBUG: once() found soul reference '{}' in parent node {}", key, parent_soul);
                                                callback(node_data, self.key.clone());
                                                return Ok(Arc::new(self.clone()));
                                            }
                                        }
                                    } else {
                                        // It's a nested object - return it directly
                                        eprintln!("DEBUG: once() extracting nested object '{}' from parent node {}", key, parent_soul);
                                        callback(value.clone(), self.key.clone());
                                        return Ok(Arc::new(self.clone()));
                                    }
                                } else {
                                    // Primitive value - return it
                                    eprintln!("DEBUG: once() extracting primitive '{}' from parent node {}", key, parent_soul);
                                    callback(value.clone(), self.key.clone());
                                    return Ok(Arc::new(self.clone()));
                                }
                            }
                        }
                        break; // Found parent with soul, stop searching
                    }
                    // Try grandparent
                    current_parent = p.parent.as_deref();
                }
            }
        }
        
        // Try to get from graph by soul
        if let Some(node) = self.core.graph.get(&soul) {
            let value = if let Some(key) = &self.key {
                node.data.get(key).cloned().unwrap_or(Value::Null)
            } else {
                serde_json::to_value(&node.data).unwrap_or(Value::Null)
            };
            callback(value, self.key.clone());
            return Ok(Arc::new(self.clone()));
        }

        // Data not found locally - request from network
        // Set up a one-time listener for this soul
        let event_type = format!("node_update:{}", soul);
        
        // Use shared data location since oneshot::Sender can't be cloned
        let data_received: Arc<parking_lot::Mutex<Option<serde_json::Value>>> = Arc::new(parking_lot::Mutex::new(None));
        let data_received_clone = data_received.clone();
        let data_ready: Arc<parking_lot::Mutex<bool>> = Arc::new(parking_lot::Mutex::new(false));
        let data_ready_clone = data_ready.clone();
        
        // Register a one-time listener (using Fn closure, not FnOnce)
        let listener_id = self.core.events.on(&event_type, Box::new(move |event: &crate::events::Event| {
            // Extract the data from the event
            let data = event.data.clone();
            *data_received_clone.lock() = Some(data);
            *data_ready_clone.lock() = true;
        }));
        
        // Emit a get request event that the mesh can listen to
        // Include key if we have one (for nested properties)
        let mut get_obj = serde_json::json!({
            "#": soul
        });
        if let Some(key) = &self.key {
            get_obj["."] = serde_json::Value::String(key.clone());
        }
        let get_request = serde_json::json!({
            "get": get_obj
        });
        eprintln!("DEBUG: Emitting get_request for soul {} with key {:?}", soul, self.key);
        self.core.events.emit(&crate::events::Event {
            event_type: "get_request".to_string(),
            data: get_request,
        });
        
        // Wait for response with timeout (20 seconds for network sync - increased for relay server delays)
        let timeout_duration = tokio::time::Duration::from_secs(20);
        let start = std::time::Instant::now();
        
        // Poll for data with timeout
        loop {
            if start.elapsed() > timeout_duration {
                eprintln!("DEBUG: once() timeout waiting for data for soul {}", soul);
                break; // Timeout
            }
            
            let ready = *data_ready.lock();
            if ready {
                eprintln!("DEBUG: once() data received for soul {}", soul);
                break; // Data received
            }
            
            // Check periodically if data arrived in graph (in case event wasn't emitted)
            if let Some(node) = self.core.graph.get(&soul) {
                let value = if let Some(key) = &self.key {
                    node.data.get(key).cloned().unwrap_or(Value::Null)
                } else {
                    serde_json::to_value(&node.data).unwrap_or(Value::Null)
                };
                if !value.is_null() {
                    eprintln!("DEBUG: once() found data in graph for soul {}, calling callback", soul);
                    *data_received.lock() = Some(value.clone());
                    *data_ready.lock() = true;
                    break;
                }
            }
            
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
        
        // Remove listener
        self.core.events.off(&event_type, listener_id);
        
        // Process result
        let received_data = data_received.lock().take();
        let value = if let Some(data) = received_data {
            if let Some(key) = &self.key {
                data.get(key).cloned().unwrap_or(Value::Null)
            } else {
                data
            }
        } else {
            Value::Null
        };
        callback(value, self.key.clone());

        Ok(Arc::new(self.clone()))
    }

    /// Map over properties of a node
    /// Based on Gun.js chain.map() - complete implementation
    pub fn map<F>(&self, callback: F) -> Arc<Chain>
    where
        F: Fn(Value, String) + Send + Sync + Clone + 'static,
    {
        let chain = Arc::new(self.clone());
        let listener_ids = self.listener_ids.clone();

        // Resolve soul the same way on() does
        let resolved_soul = if let Some(ref s) = &self.soul {
            s.clone()
        } else if let Some(ref k) = &self.key {
            // Try to resolve path by checking parent node
            if let Some(parent) = &self.parent {
                // Walk up the parent chain to find a node with a soul
                let mut current_parent: Option<&Chain> = Some(parent.as_ref());
                let mut found_parent_soul: Option<String> = None;
                
                while let Some(p) = current_parent {
                    if let Some(ps) = &p.soul {
                        found_parent_soul = Some(ps.clone());
                        break;
                    }
                    current_parent = p.parent.as_deref();
                }
                
                if let Some(parent_soul) = found_parent_soul {
                    parent_soul
                } else {
                    "graph_update".to_string()
                }
            } else {
                "graph_update".to_string()
            }
        } else {
            "graph_update".to_string()
        };

        // Clone callback for use in closure
        let callback_clone = callback.clone();
        let key_for_map = self.key.clone();

        // Subscribe to updates and call callback for each property
        if resolved_soul != "graph_update" {
            let event_type = format!("node_update:{}", resolved_soul);
            let core_for_map = self.core.clone();
            let cb = Box::new(move |event: &crate::events::Event| {
                if let Some(data_obj) = event.data.as_object() {
                    // Check if we have a key - if so, the data might be in a referenced node
                    if let Some(ref k) = &key_for_map {
                        if let Some(value) = data_obj.get(k) {
                            // Check if it's a soul reference
                            if let Some(obj) = value.as_object() {
                                if let Some(soul_ref) = obj.get("#") {
                                    if let Some(soul_str) = soul_ref.as_str() {
                                        // It's a soul reference - map over the referenced node
                                        if let Some(ref_node) = core_for_map.graph.get(soul_str) {
                                            for (key, value) in ref_node.data.iter() {
                                                if key != "_" && !key.starts_with('>') {
                                                    callback_clone(value.clone(), key.clone());
                                                }
                                            }
                                            return;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // No key or not a soul reference - map over the event data directly
                    for (key, value) in data_obj {
                        if key != "_" && !key.starts_with('>') {
                            callback_clone(value.clone(), key.clone());
                        }
                    }
                }
            });

            let listener_id = self.core.events.on(&event_type, cb);
            listener_ids.lock().insert(listener_id);

            // Also call for current data if available
            if let Some(node) = self.core.graph.get(&resolved_soul) {
                // Check if we have a key - if so, the data might be in a referenced node
                if let Some(ref k) = &self.key {
                    if let Some(value) = node.data.get(k) {
                        // Check if it's a soul reference
                        if let Some(obj) = value.as_object() {
                            if let Some(soul_ref) = obj.get("#") {
                                if let Some(soul_str) = soul_ref.as_str() {
                                    // It's a soul reference - map over the referenced node
                                    if let Some(ref_node) = self.core.graph.get(soul_str) {
                                        for (key, value) in ref_node.data.iter() {
                                            if key != "_" && !key.starts_with('>') {
                                                callback(value.clone(), key.clone());
                                            }
                                        }
                                        return chain;
                                    }
                                }
                            }
                        }
                    }
                }
                // No key or not a soul reference - map over the node data directly
                for (key, value) in node.data.iter() {
                    if key != "_" && !key.starts_with('>') {
                        callback(value.clone(), key.clone());
                    }
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
                                Some(k),
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
            Some(1) => self.parent.clone(),
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

        // Resolve soul the same way on() does
        let resolved_soul = if let Some(ref s) = &self.soul {
            s.clone()
        } else if let Some(ref k) = &self.key {
            // Try to resolve path by checking parent node
            if let Some(parent) = &self.parent {
                // Walk up the parent chain to find a node with a soul
                let mut current_parent: Option<&Chain> = Some(parent.as_ref());
                let mut found_parent_soul: Option<String> = None;
                
                while let Some(p) = current_parent {
                    if let Some(ps) = &p.soul {
                        found_parent_soul = Some(ps.clone());
                        break;
                    }
                    current_parent = p.parent.as_deref();
                }
                
                if let Some(parent_soul) = found_parent_soul {
                    parent_soul
                } else {
                    "graph_update".to_string()
                }
            } else {
                "graph_update".to_string()
            }
        } else {
            "graph_update".to_string()
        };

        let event_type = if resolved_soul != "graph_update" {
            format!("node_update:{}", resolved_soul)
        } else {
            "graph_update".to_string()
        };

        // Try removing from the resolved event type
        for id in ids.iter() {
            self.core.events.off(&event_type, *id);
        }
        
        // Also try removing from graph_update as fallback (in case listener was registered with different type)
        if event_type != "graph_update" {
            for id in ids.iter() {
                self.core.events.off("graph_update", *id);
            }
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
