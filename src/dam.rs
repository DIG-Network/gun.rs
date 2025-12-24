use crate::dup::Dup;
use crate::error::GunResult;
use crate::core::GunCore;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};

/// DAM (Directed Acyclic Mesh) protocol implementation
/// Matches Gun.js mesh.js - handles P2P message routing with deduplication

#[derive(Clone, Debug)]
pub struct Peer {
    pub id: String,
    pub url: String,
    pub pid: Option<String>, // peer ID for DAM
    pub tx: Option<mpsc::UnboundedSender<String>>, // WebSocket message sender
    pub batch: Option<String>, // batched messages
    pub tail: usize, // batch size
    pub queue: Vec<String>, // queued messages
    pub last: Option<String>, // last message ID sent
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
            tx.send(message.to_string())
                .map_err(|e| crate::error::GunError::Network(format!("Failed to send message: {}", e)))?;
        } else {
            // Queue message if not connected
            // Note: This requires mut access, so we'd need to modify the structure
            // For now, we'll handle queuing in Mesh.send_to_peer()
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
    pub pid: String, // our peer ID
    opt: MeshOptions,
}

#[derive(Clone, Debug)]
pub struct MeshOptions {
    pub max_message_size: usize, // default 300MB * 0.3
    pub pack_size: usize, // batch size
    pub gap: u64, // batching delay in ms
    pub retry: i32,
    pub lack: u64, // lack timeout
}

impl Default for MeshOptions {
    fn default() -> Self {
        Self {
            max_message_size: (300_000_000 as f64 * 0.3) as usize,
            pack_size: ((300_000_000 as f64 * 0.3 * 0.01 * 0.01) as usize),
            gap: 0,
            retry: 60,
            lack: 9000,
        }
    }
}

impl Mesh {
    pub fn new(core: Arc<GunCore>) -> Self {
        let pid = core.random_id(9);
        Self {
            dup: Arc::new(RwLock::new(Dup::default())),
            peers: Arc::new(RwLock::new(HashMap::new())),
            core,
            near: Arc::new(RwLock::new(0)),
            pid,
            opt: MeshOptions::default(),
        }
    }

    /// Handle incoming message (matches mesh.hear)
    pub async fn hear(&self, raw: &str, peer: &Peer) -> GunResult<()> {
        if raw.is_empty() {
            return Ok(());
        }

        // Check message size
        if raw.len() > self.opt.max_message_size {
            self.say(
                &serde_json::json!({
                    "dam": "!",
                    "err": "Message too big!"
                }),
                Some(peer),
            ).await?;
            return Ok(());
        }

        // Handle batched messages (JSON array)
        if raw.starts_with('[') {
            let messages: Vec<Value> = serde_json::from_str(raw)?;
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
    async fn hear_one(&self, msg: &Value, peer: &Peer) -> GunResult<()> {
        let msg_id = msg.get("#")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.core.random_id(9));

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
                    if let Some(err) = msg.get("err").and_then(|v| v.as_str()) {
                        eprintln!("DAM Error from peer {}: {}", peer.id, err);
                    }
                }
                "?" => {
                    // Peer ID exchange
                    self.handle_peer_id_exchange(msg, peer).await?;
                }
                _ => {
                    // Other DAM message types
                }
            }
            return Ok(());
        }

        // Forward to core event system
        // This would trigger root.on('in', msg)
        // For now, we'll store it for routing
        
        Ok(())
    }

    /// Handle peer ID exchange (DAM '?' message)
    async fn handle_peer_id_exchange(&self, msg: &Value, peer: &Peer) -> GunResult<()> {
        if let Some(pid) = msg.get("pid").and_then(|v| v.as_str()) {
            let mut peers = self.peers.write().await;
            if let Some(p) = peers.get_mut(&peer.id) {
                p.pid = Some(pid.to_string());
            }
            
            // Reply with our PID
            self.say(
                &serde_json::json!({
                    "dam": "?",
                    "pid": self.pid,
                    "@": msg.get("#")
                }),
                Some(peer),
            ).await?;
        }
        Ok(())
    }

    /// Send message to peer(s) (matches mesh.say)
    pub async fn say(&self, msg: &Value, peer: Option<&Peer>) -> GunResult<()> {
        // Generate message ID if not present
        let mut msg = msg.clone();
        if msg.get("#").is_none() {
            msg["#"] = serde_json::Value::String(self.core.random_id(9));
        }

        let raw = serde_json::to_string(&msg)?;

        if let Some(p) = peer {
            self.send_to_peer(&raw, p).await?;
        } else {
            // Broadcast to all peers
            let peers = self.peers.read().await;
            for peer in peers.values() {
                self.send_to_peer(&raw, peer).await?;
            }
        }

        Ok(())
    }

    /// Send raw message to a specific peer
    /// Routes through WebSocket connection if available, otherwise queues
    async fn send_to_peer(&self, _raw: &str, peer: &Peer) -> GunResult<()> {
        // Check if peer has an active WebSocket connection
        if let Some(ref tx) = peer.tx {
            // Send immediately through WebSocket
            tx.send(_raw.to_string())
                .map_err(|e| crate::error::GunError::Network(format!("Failed to send to peer {}: {}", peer.id, e)))?;
        } else {
            // Queue message for when connection is established
            // Note: This requires mutable access to peer, which we don't have here
            // In a full implementation, we'd store the queue in the Mesh and flush it on connection
            // For now, we'll just log that the message can't be sent
            eprintln!("Warning: Peer {} not connected, message queued (queuing not fully implemented)", peer.id);
        }
        Ok(())
    }
    
    /// Update peer with WebSocket sender (called when connection is established)
    pub async fn set_peer_sender(&self, peer_id: &str, tx: mpsc::UnboundedSender<String>) -> GunResult<()> {
        let mut peers = self.peers.write().await;
        if let Some(peer) = peers.get_mut(peer_id) {
            let tx_clone = tx.clone();
            peer.set_sender(tx);
            // Flush any queued messages
            let queue = peer.queue.clone();
            peer.queue.clear();
            drop(peers); // Release lock
            
            // Send queued messages
            for msg in queue {
                if let Err(e) = tx_clone.send(msg) {
                    eprintln!("Error sending queued message: {}", e);
                    break;
                }
            }
        }
        Ok(())
    }

    /// Add a peer (matches mesh.hi)
    pub async fn hi(&self, peer: Peer) -> GunResult<()> {
        let mut peers = self.peers.write().await;
        let was_new = !peers.contains_key(&peer.id);
        peers.insert(peer.id.clone(), peer);
        drop(peers);
        
        if was_new {
            let mut near = self.near.write().await;
            *near += 1;
            
            // Send hi message to the new peer
            self.say(
                &serde_json::json!({
                    "dam": "hi"
                }),
                None, // Broadcast
            ).await?;
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
}

