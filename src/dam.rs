use crate::core::GunCore;
use crate::dup::Dup;
use crate::error::GunResult;
use chia_bls::{PublicKey, SecretKey, Signature, sign, verify};
use serde_json::Value;
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// DAM (Directed Acyclic Mesh) protocol implementation
/// Matches Gun.js mesh.js - handles P2P message routing with deduplication

#[derive(Clone, Debug)]
pub struct Peer {
    pub id: String,
    pub url: String,
    pub pid: Option<String>,                       // peer ID for DAM
    pub tx: Option<mpsc::UnboundedSender<String>>, // WebSocket message sender
    pub batch: Option<String>,                     // batched messages
    pub tail: usize,                               // batch size
    pub queue: Vec<String>,                        // queued messages
    pub last: Option<String>,                      // last message ID sent
    pub retry: i32,
    pub tried: Option<u64>, // timestamp
}

impl Peer {
    pub fn new(url: String) -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        Self {
            id: format!("peer_{}", id),
            url,
            pid: None,
            tx: None,
            batch: None,
            tail: 0,
            queue: vec![],
            last: None,
            retry: 60,
            tried: None,
        }
    }

    /// Set the WebSocket message sender
    pub fn set_sender(&mut self, tx: mpsc::UnboundedSender<String>) {
        self.tx = Some(tx);
    }

    /// Send a message through the WebSocket connection
    pub async fn send(&self, message: &str) -> GunResult<()> {
        if let Some(ref tx) = self.tx {
            tx.send(message.to_string()).map_err(|e| {
                crate::error::GunError::Network(format!("Failed to send message: {}", e))
            })?;
        } else {
            // Peer not connected - message will be dropped
            // Note: In a full implementation, messages could be queued for later delivery
            // Currently, Mesh.send_to_peer_by_id() will return an error if peer is not found
            // This is acceptable behavior as peers will reconnect and sync state
        }
        Ok(())
    }
}

/// DAM Mesh - handles message routing and peer communication
/// Based on Gun.js mesh.js
pub struct Mesh {
    pub dup: Arc<RwLock<Dup>>,
    peers: Arc<RwLock<HashMap<String, Peer>>>,
    core: Arc<GunCore>,
    pub near: Arc<RwLock<usize>>, // number of connected peers
    pub pid: String,              // our peer ID
    opt: MeshOptions,
    secret_key: SecretKey,        // BLS secret key for signing outgoing messages
    public_key: PublicKey,        // BLS public key (our own, for reference)
    peer_public_keys: Arc<RwLock<HashMap<String, PublicKey>>>, // Map peer_id -> public_key for verification
    message_predicate: Option<Arc<dyn Fn(&serde_json::Value) -> bool + Send + Sync>>, // Optional predicate for custom message filtering
}

#[derive(Clone, Debug)]
pub struct MeshOptions {
    pub max_message_size: usize, // default 300MB * 0.3
    pub pack_size: usize,        // batch size
    pub gap: u64,                // batching delay in ms
    pub retry: i32,
    pub lack: u64, // lack timeout
}

impl Default for MeshOptions {
    fn default() -> Self {
        Self {
            max_message_size: (300_000_000.0 * 0.3) as usize,
            pack_size: ((300_000_000.0 * 0.3 * 0.01 * 0.01) as usize),
            gap: 0,
            retry: 60,
            lack: 9000,
        }
    }
}

impl Mesh {
    pub fn new(core: Arc<GunCore>, secret_key: SecretKey, public_key: PublicKey, message_predicate: Option<Arc<dyn Fn(&serde_json::Value) -> bool + Send + Sync>>) -> Self {
        let pid = core.random_id(9);
        Self {
            dup: Arc::new(RwLock::new(Dup::new_default())),
            peers: Arc::new(RwLock::new(HashMap::new())),
            core,
            near: Arc::new(RwLock::new(0)),
            pid,
            opt: MeshOptions::default(),
            secret_key,
            public_key,
            peer_public_keys: Arc::new(RwLock::new(HashMap::new())),
            message_predicate,
        }
    }

    /// Handle incoming message (matches mesh.hear)
    pub async fn hear(&self, raw: &str, peer: Option<&Peer>) -> GunResult<()> {
        if raw.is_empty() {
            return Ok(());
        }

        let peer_id = peer.map(|p| p.id.clone()).unwrap_or_else(|| "unknown".to_string());
        eprintln!("DEBUG: mesh.hear() received message from peer {}: {}", peer_id, raw.chars().take(200).collect::<String>());

        // Check message size
        if raw.len() > self.opt.max_message_size {
            if let Some(p) = peer {
                self.say(
                    &serde_json::json!({
                        "dam": "!",
                        "err": "Message too big!"
                    }),
                    Some(p),
                )
                .await?;
            }
            return Ok(());
        }

        // Handle batched messages (JSON array)
        if raw.starts_with('[') {
            let messages: Vec<Value> = serde_json::from_str(raw)?;
            eprintln!("DEBUG: Processing {} batched messages from peer {}", messages.len(), peer_id);
            for msg in messages {
                self.hear_one(&msg, peer).await?;
            }
            return Ok(());
        }

        // Handle single message
        let msg: Value = serde_json::from_str(raw)?;
        self.hear_one(&msg, peer).await?;
        Ok(())
    }

    /// Handle a single message (matches mesh.hear.one)
    async fn hear_one(&self, msg: &Value, peer: Option<&Peer>) -> GunResult<()> {
        // Get message ID (should be SHA256 hash of message without sigs)
        let msg_id = msg
            .get("#")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                crate::error::GunError::Network("Message missing ID (#) field".to_string())
            })?;

        // Create message bytes for verification (without sigs field)
        let mut msg_for_hash = msg.clone();
        msg_for_hash.as_object_mut().unwrap().remove("sigs");
        let msg_bytes = serde_json::to_vec(&msg_for_hash)?;
        
        // Verify that the message ID matches the SHA256 hash of the message (without sigs)
        let mut hasher = Sha256::new();
        hasher.update(&msg_bytes);
        let computed_hash = hasher.finalize();
        let computed_hash_hex = hex::encode(computed_hash);
        
        if msg_id != computed_hash_hex {
            eprintln!("DEBUG: Message ID hash mismatch. Expected: {}, Got: {}", computed_hash_hex, msg_id);
            return Ok(()); // Reject message with invalid hash
        }

        // Verify all signatures in the aggregate before processing
        let sigs_array = if let Some(sigs) = msg.get("sigs").and_then(|v| v.as_array()) {
            if sigs.is_empty() {
                eprintln!("DEBUG: Message missing signatures from peer {:?}", peer.map(|p| &p.id));
                return Ok(()); // Reject message without signatures
            }
            sigs
        } else {
            // Legacy format: try to read single sig/pubkey for backward compatibility
            // But we'll still require sigs array going forward
            eprintln!("DEBUG: Message missing sigs array from peer {:?}", peer.map(|p| &p.id));
            return Ok(()); // Reject message without sigs array
        };
        
        // msg_bytes already computed above for hash verification, reuse it for signature verification
        
        // Verify ALL signatures in the aggregate
        let mut verified_pubkeys = Vec::new();
        for sig_entry in sigs_array {
            let sig_hex = match sig_entry.get("sig").and_then(|v| v.as_str()) {
                Some(hex) => hex,
                None => {
                    eprintln!("DEBUG: Invalid signature entry: missing sig");
                    return Ok(()); // Reject message with invalid signature entry
                }
            };
            let pubkey_hex = match sig_entry.get("pubkey").and_then(|v| v.as_str()) {
                Some(hex) => hex,
                None => {
                    eprintln!("DEBUG: Invalid signature entry: missing pubkey");
                    return Ok(()); // Reject message with invalid signature entry
                }
            };
            
            // Decode signature and public key
            let sig_bytes = match hex::decode(sig_hex) {
                Ok(bytes) => bytes,
                Err(e) => {
                    eprintln!("DEBUG: Invalid signature hex: {}", e);
                    return Ok(()); // Reject message with invalid signature hex
                }
            };
            let pubkey_bytes = match hex::decode(pubkey_hex) {
                Ok(bytes) => bytes,
                Err(e) => {
                    eprintln!("DEBUG: Invalid public key hex: {}", e);
                    return Ok(()); // Reject message with invalid public key hex
                }
            };
            
            // Convert to fixed-size arrays
            if sig_bytes.len() != 96 {
                eprintln!("DEBUG: Invalid signature length: expected 96 bytes, got {}", sig_bytes.len());
                return Ok(()); // Reject message with invalid signature
            }
            if pubkey_bytes.len() != 48 {
                eprintln!("DEBUG: Invalid public key length: expected 48 bytes, got {}", pubkey_bytes.len());
                return Ok(()); // Reject message with invalid public key
            }
            
            let mut sig_array = [0u8; 96];
            sig_array.copy_from_slice(&sig_bytes);
            let mut pubkey_array = [0u8; 48];
            pubkey_array.copy_from_slice(&pubkey_bytes);
            
            let signature = match Signature::from_bytes(&sig_array) {
                Ok(sig) => sig,
                Err(e) => {
                    eprintln!("DEBUG: Invalid signature format: {}", e);
                    return Ok(()); // Reject message with invalid signature
                }
            };
            let sender_pubkey = match PublicKey::from_bytes(&pubkey_array) {
                Ok(pk) => pk,
                Err(e) => {
                    eprintln!("DEBUG: Invalid public key format: {}", e);
                    return Ok(()); // Reject message with invalid public key
                }
            };
            
            // Verify this signature
            if !verify(&signature, &sender_pubkey, &msg_bytes) {
                eprintln!("DEBUG: Signature verification failed for pubkey {} from peer {:?}", pubkey_hex, peer.map(|p| &p.id));
                return Ok(()); // Reject message if any signature is invalid
            }
            
            verified_pubkeys.push(sender_pubkey);
        }
        
        // Store peer's public keys for future reference
        if let Some(p) = peer {
            let mut peer_keys = self.peer_public_keys.write().await;
            for pubkey in &verified_pubkeys {
                peer_keys.insert(format!("{}:{}", p.id, hex::encode(pubkey.to_bytes())), pubkey.clone());
            }
        }
        
        // Check if my signature is already in the aggregate
        let my_pubkey_hex = hex::encode(self.public_key.to_bytes());
        let has_my_sig = sigs_array.iter().any(|sig_obj| {
            sig_obj.get("pubkey")
                .and_then(|v| v.as_str())
                .map(|pk| pk == my_pubkey_hex)
                .unwrap_or(false)
        });
        
        // If my signature is not present, add it and re-broadcast (but exclude the sender)
        if !has_my_sig {
            // Sign the message
            let signature = sign(&self.secret_key, &msg_bytes);
            let signature_hex = hex::encode(signature.to_bytes());
            
            // Add my signature to the array
            let mut updated_msg = msg.clone();
            let mut updated_sigs = sigs_array.to_vec();
            let sig_entry = serde_json::json!({
                "sig": signature_hex,
                "pubkey": my_pubkey_hex
            });
            updated_sigs.push(sig_entry);
            updated_msg["sigs"] = serde_json::Value::Array(updated_sigs);
            
            // Re-broadcast to all peers except the one that sent it to us
            let sender_id = peer.map(|p| p.id.clone());
            let peer_ids: Vec<String> = {
                let peers = self.peers.read().await;
                peers.keys()
                    .filter(|id| Some((**id).clone()) != sender_id)
                    .cloned()
                    .collect()
            };
            
            let updated_raw = serde_json::to_string(&updated_msg)?;
            for peer_id in peer_ids {
                if let Err(e) = self.send_to_peer_by_id(&updated_raw, &peer_id).await {
                    eprintln!("Error re-broadcasting signed message to peer {}: {}", peer_id, e);
                }
            }
        }

        // Check custom message predicate (application-level filtering)
        // This runs after signature verification but before message processing
        if let Some(ref predicate) = self.message_predicate {
            if !predicate(msg) {
                eprintln!("DEBUG: Message rejected by custom predicate from peer {:?}", peer.map(|p| &p.id));
                return Ok(()); // Reject message based on predicate
            }
        }

        // Deduplication check
        {
            let mut dup = self.dup.write().await;
            if dup.check(&msg_id) {
                return Ok(()); // duplicate, ignore
            }
            dup.track(&msg_id);
        }

        // Handle special DAM messages
        if let Some(dam_type) = msg.get("dam").and_then(|v| v.as_str()) {
            match dam_type {
                "!" => {
                    // Error message
                    if let Some(p) = peer {
                        if let Some(err) = msg.get("err").and_then(|v| v.as_str()) {
                            eprintln!("DAM Error from peer {}: {}", p.id, err);
                        }
                    }
                }
                "?" => {
                    // Peer ID exchange
                    if let Some(p) = peer {
                        self.handle_peer_id_exchange(msg, p).await?;
                    }
                }
                "rtc" => {
                    // WebRTC signaling message - these are handled at the Gun level
                    // to avoid circular dependencies between Mesh and WebRTCManager
                    tracing::debug!("Received RTC signaling message via DAM protocol");
                }
                _ => {
                    // Other DAM message types
                }
            }
            return Ok(());
        }

        // Process Gun protocol messages (put, get)
        if let Some(put_data) = msg.get("put") {
            eprintln!("DEBUG: Received put message: {}", serde_json::to_string(msg).unwrap_or_default());
            // Handle put message - update graph and emit node_update event
            // Gun.js format: { put: { soul: { _: { "#": soul, ">": states }, ...data } } }
            // The soul is a KEY in the put object, not a field
            if let Some(put_obj) = put_data.as_object() {
                // Iterate over each soul in the put object
                for (soul, node_data) in put_obj {
                    if let Some(node_obj) = node_data.as_object() {
                        // Extract metadata from "_" field
                        let meta = node_obj.get("_").and_then(|v| v.as_object());
                        let soul_from_meta = meta.and_then(|m| m.get("#")).and_then(|v| v.as_str()).unwrap_or(soul);
                        
                        // Extract state map from ">" field in metadata
                        let states = meta.and_then(|m| m.get(">")).and_then(|v| v.as_object());
                        
                        // Update graph
                        use crate::state::Node;
                        let mut node = self.core.graph.get(soul_from_meta)
                            .unwrap_or_else(|| Node::with_soul(soul_from_meta.to_string()));
                        
                        // Merge all fields from node_obj into node (except "_" which is metadata)
                        for (key, value) in node_obj {
                            if key != "_" {
                                // Get state for this key from states map if available
                                let state = states.and_then(|s| s.get(key))
                                    .and_then(|v| v.as_f64())
                                    .unwrap_or_else(|| self.core.state.next());
                                
                                node.data.insert(key.clone(), value.clone());
                                crate::state::State::ify(&mut node, Some(&key), Some(state), Some(value.clone()), Some(soul_from_meta));
                            }
                        }
                        
                        // Store updated node
                        if let Err(e) = self.core.graph.put(soul_from_meta, node.clone()) {
                            eprintln!("Error updating graph for soul {}: {}", soul_from_meta, e);
                        } else {
                            eprintln!("DEBUG: Updated graph for soul {} (from peer), emitting node_update event. Node data keys: {:?}", soul_from_meta, node.data.keys().collect::<Vec<_>>());
                            // Emit node_update event so once() and on() callbacks get called
                            let event_type = format!("node_update:{}", soul_from_meta);
                            self.core.events.emit(&crate::events::Event {
                                event_type: event_type.clone(),
                                data: serde_json::Value::Object(node.data.clone()),
                            });
                            // Also emit graph_update for listeners that don't have a specific soul yet
                            self.core.events.emit(&crate::events::Event {
                                event_type: "graph_update".to_string(),
                                data: serde_json::json!({
                                    soul_from_meta: serde_json::Value::Object(node.data.clone())
                                }),
                            });
                        }
                    }
                }
            }
        } else if let Some(get_data) = msg.get("get") {
            eprintln!("DEBUG: Received get message: {}", serde_json::to_string(msg).unwrap_or_default());
            // Handle get message - respond with requested data
            if let Some(get_obj) = get_data.as_object() {
                if let Some(soul_val) = get_obj.get("#") {
                    if let Some(soul) = soul_val.as_str() {
                        // Check if we have the requested node
                        if let Some(node) = self.core.graph.get(soul) {
                            // Check if get request has a key (for nested properties)
                            if let Some(key_val) = get_obj.get(".") {
                                if let Some(key) = key_val.as_str() {
                                    // Requesting a specific key - check if it's a soul reference
                                    if let Some(value) = node.data.get(key) {
                                        if let Some(obj) = value.as_object() {
                                            if let Some(soul_ref) = obj.get("#") {
                                                if let Some(ref_soul) = soul_ref.as_str() {
                                                    // It's a soul reference - get the referenced node
                                                    if let Some(ref_node) = self.core.graph.get(ref_soul) {
                                                        let mut put_obj = serde_json::json!({
                                                            "#": ref_soul
                                                        });
                                                        for (k, v) in &ref_node.data {
                                                            put_obj[k] = v.clone();
                                                        }
                                                        let response = serde_json::json!({
                                                            "put": put_obj
                                                        });
                                                        eprintln!("DEBUG: Sending get response for nested soul {} (key: {}) to peer", ref_soul, key);
                                                        if let Some(p) = peer {
                                                            if let Err(e) = self.say(&response, Some(p)).await {
                                                                eprintln!("Error sending get response to peer {}: {}", p.id, e);
                                                            }
                                                        }
                                                    }
                                                }
                                            } else {
                                                // Not a soul reference - return the value directly
                                                let put_obj = serde_json::json!({
                                                    "#": soul,
                                                    key: value.clone()
                                                });
                                                let response = serde_json::json!({
                                                    "put": put_obj
                                                });
                                                eprintln!("DEBUG: Sending get response for key {} in soul {} to peer", key, soul);
                                                if let Some(p) = peer {
                                                    if let Err(e) = self.say(&response, Some(p)).await {
                                                        eprintln!("Error sending get response to peer {}: {}", p.id, e);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                // No key specified - return entire node
                                let mut put_obj = serde_json::json!({
                                    "#": soul
                                });
                                for (key, value) in &node.data {
                                    put_obj[key] = value.clone();
                                }
                                let response = serde_json::json!({
                                    "put": put_obj
                                });
                                eprintln!("DEBUG: Sending get response for soul {} to peer. Response: {}", soul, serde_json::to_string(&response).unwrap_or_default());
                                // Broadcast the response instead of sending to specific peer
                                // This ensures the relay server forwards it properly
                                if let Err(e) = self.say(&response, None).await {
                                    eprintln!("Error broadcasting get response: {}", e);
                                } else {
                                    eprintln!("DEBUG: Get response broadcast successfully");
                                }
                            }
                        } else {
                            eprintln!("DEBUG: Requested soul {} not found in graph", soul);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle peer ID exchange (DAM '?' message)
    async fn handle_peer_id_exchange(&self, msg: &Value, peer: &Peer) -> GunResult<()> {
        if let Some(pid) = msg.get("pid").and_then(|v| v.as_str()) {
            // Update peer PID with minimal lock time
            {
                let mut peers = self.peers.write().await;
                if let Some(p) = peers.get_mut(&peer.id) {
                    p.pid = Some(pid.to_string());
                }
            } // Lock released before calling say()

            // Reply with our PID (lock released to avoid deadlock)
            self.say(
                &serde_json::json!({
                    "dam": "?",
                    "pid": self.pid,
                    "@": msg.get("#")
                }),
                Some(peer),
            )
            .await?;
        }
        Ok(())
    }

    /// Send message to peer(s) (matches mesh.say)
    pub async fn say(&self, msg: &Value, peer: Option<&Peer>) -> GunResult<()> {
        let mut msg = msg.clone();
        
        // Create message bytes for hashing and signing (without sigs field)
        let mut msg_for_hash = msg.clone();
        msg_for_hash.as_object_mut().unwrap().remove("sigs");
        let msg_bytes = serde_json::to_vec(&msg_for_hash)?;
        
        // Generate message ID if not present - use SHA256 hash of message (without sigs)
        if msg.get("#").is_none() {
            let mut hasher = Sha256::new();
            hasher.update(&msg_bytes);
            let hash = hasher.finalize();
            let hash_hex = hex::encode(hash);
            msg["#"] = serde_json::Value::String(hash_hex);
        }
        
        // Check if message already has signatures
        let mut sigs_array = if let Some(sigs) = msg.get("sigs").and_then(|v| v.as_array()) {
            sigs.clone()
        } else {
            Vec::new()
        };
        
        // Check if my signature is already in the array
        let my_pubkey_hex = hex::encode(self.public_key.to_bytes());
        let has_my_sig = sigs_array.iter().any(|sig_obj| {
            sig_obj.get("pubkey")
                .and_then(|v| v.as_str())
                .map(|pk| pk == my_pubkey_hex)
                .unwrap_or(false)
        });
        
        // If my signature is not present, add it
        if !has_my_sig {
            let signature = sign(&self.secret_key, &msg_bytes);
            let signature_hex = hex::encode(signature.to_bytes());
            
            let sig_entry = serde_json::json!({
                "sig": signature_hex,
                "pubkey": my_pubkey_hex
            });
            sigs_array.push(sig_entry);
        }
        
        // Add signatures array to message
        msg["sigs"] = serde_json::Value::Array(sigs_array);

        let raw = serde_json::to_string(&msg)?;

        if let Some(p) = peer {
            self.send_to_peer_by_id(&raw, &p.id).await?;
        } else {
            // Broadcast to all peers - clone IDs first to avoid holding lock during async calls
            let peer_ids: Vec<String> = {
                let peers = self.peers.read().await;
                let ids: Vec<String> = peers.keys().cloned().collect();
                eprintln!("DEBUG: Broadcasting message to {} peers: {:?}", ids.len(), ids);
                ids
            };

            // Now send to each peer without holding the lock
            for peer_id in peer_ids {
                eprintln!("DEBUG: Attempting to send broadcast message to peer {}", peer_id);
                if let Err(e) = self.send_to_peer_by_id(&raw, &peer_id).await {
                    eprintln!("Error sending to peer {}: {}", peer_id, e);
                    // Continue sending to other peers even if one fails
                } else {
                    eprintln!("DEBUG: Successfully sent broadcast message to peer {}", peer_id);
                }
            }
        }

        Ok(())
    }

    /// Send raw message to a specific peer by ID
    /// Routes through WebSocket connection if available, otherwise queues
    pub(crate) async fn send_to_peer_by_id(&self, raw: &str, peer_id: &str) -> GunResult<()> {
        // Try to get the sender without holding the lock for long
        let tx_opt = {
            let peers = self.peers.read().await;
            if let Some(peer) = peers.get(peer_id) {
                eprintln!("DEBUG: Found peer {} with sender available", peer_id);
                peer.tx.clone() // Clone the Sender to release the lock immediately
            } else {
                eprintln!("DEBUG: Peer {} not found in peers list ({} total peers)", peer_id, peers.len());
                None // Peer not found
            }
        };

        if let Some(tx) = tx_opt {
            // Send immediately through WebSocket (no lock held)
            let msg_preview = raw.chars().take(150).collect::<String>();
            eprintln!("DEBUG: Sending message to WebSocket for peer {}: {}", peer_id, msg_preview);
            tx.send(raw.to_string()).map_err(|e| {
                eprintln!("DEBUG: WebSocket send error for peer {}: {}", peer_id, e);
                crate::error::GunError::Network(format!(
                    "Failed to send to peer {}: {}",
                    peer_id, e
                ))
            })?;
            eprintln!("DEBUG: Message sent successfully to WebSocket for peer {}", peer_id);
            return Ok(());
        }

        // No sender available - queue the message
        // Use a short write lock to add to queue
        {
            let mut peers = self.peers.write().await;
            if let Some(peer) = peers.get_mut(peer_id) {
                peer.queue.push(raw.to_string());
                // Don't warn - this is expected during initial connection
            } else {
                // Peer doesn't exist - this is fine, they'll get it when they connect
                return Ok(());
            }
        } // Lock released here

        Ok(())
    }

    /// Send raw message to a specific peer (by Peer reference)
    /// Routes through WebSocket connection if available, otherwise queues
    #[allow(dead_code)] // Used internally for peer communication
    async fn send_to_peer(&self, raw: &str, peer: &Peer) -> GunResult<()> {
        self.send_to_peer_by_id(raw, &peer.id).await
    }

    /// Update peer with WebSocket sender (called when connection is established)
    pub async fn set_peer_sender(
        &self,
        peer_id: &str,
        tx: mpsc::UnboundedSender<String>,
    ) -> GunResult<()> {
        let mut peers = self.peers.write().await;
        if let Some(peer) = peers.get_mut(peer_id) {
            let tx_clone = tx.clone();
            peer.set_sender(tx);
            // Flush any queued messages
            let queue = peer.queue.clone();
            peer.queue.clear();
            drop(peers); // Release lock as soon as possible

            // Send queued messages (outside of lock to avoid deadlocks)
            for msg in queue {
                if let Err(e) = tx_clone.send(msg) {
                    eprintln!("Error sending queued message: {}", e);
                    break;
                }
            }
        } else {
            // Peer not found - this shouldn't happen if hi() was called first
            drop(peers);
            return Err(crate::error::GunError::Network(format!(
                "Peer {} not found in mesh, call hi() first",
                peer_id
            )));
        }
        Ok(())
    }

    /// Add a peer (matches mesh.hi)
    pub async fn hi(&self, peer: Peer) -> GunResult<()> {
        let mut peers = self.peers.write().await;
        let was_new = !peers.contains_key(&peer.id);
        let peer_id = peer.id.clone();
        peers.insert(peer_id.clone(), peer.clone());
        drop(peers);

        if was_new {
            let mut near = self.near.write().await;
            *near += 1;
            drop(near);

            // Send "hi" message to introduce ourselves to the new peer
            // We do this after releasing the lock to avoid deadlocks
            // The "hi" message contains our peer ID (pid) for DAM protocol
            // Use say() to ensure the message is signed
            let hi_message = serde_json::json!({
                "dam": "?",
                "pid": self.pid,
            });
            
            // Send "hi" message using say() which will sign it
            if let Err(e) = self.say(&hi_message, Some(&peer)).await {
                tracing::warn!("Failed to send hi message to peer {}: {}", peer_id, e);
            }
        }
        Ok(())
    }

    /// Remove a peer (matches mesh.bye)
    pub async fn bye(&self, peer_id: &str) -> GunResult<()> {
        let mut peers = self.peers.write().await;
        if peers.remove(peer_id).is_some() {
            let mut near = self.near.write().await;
            if *near > 0 {
                *near -= 1;
            }
        }
        Ok(())
    }

    /// Get the number of connected peers (peers with active WebSocket connections)
    /// Acquires read lock with timeout to avoid indefinite blocking
    pub async fn connected_peer_count(&self) -> usize {
        use tokio::time::{timeout, Duration};

        // Use timeout to prevent indefinite blocking
        match timeout(Duration::from_millis(100), self.peers.read()).await {
            Ok(peers) => peers.values().filter(|p| p.tx.is_some()).count(),
            Err(_) => {
                // Lock acquisition timed out - this shouldn't happen in normal operation
                // but we return 0 to indicate we can't determine the count
                0
            }
        }
    }

    /// Check if any peers are connected
    pub async fn has_connected_peers(&self) -> bool {
        self.connected_peer_count().await > 0
    }

    /// Wait for at least one peer to be connected, with timeout
    pub async fn wait_for_connection(&self, timeout_ms: u64) -> bool {
        use tokio::time::{sleep, Duration, Instant};
        let deadline = Instant::now() + Duration::from_millis(timeout_ms);

        while Instant::now() < deadline {
            if self.has_connected_peers().await {
                return true;
            }
            sleep(Duration::from_millis(100)).await;
        }
        false
    }

    /// Get a peer by ID
    pub async fn get_peer(&self, peer_id: &str) -> Option<Peer> {
        let peers = self.peers.read().await;
        peers.get(peer_id).cloned()
    }
}
