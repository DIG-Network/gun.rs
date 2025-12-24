use crate::core::GunCore;
use crate::dup::Dup;
use crate::error::GunResult;
use serde_json::Value;
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
    pub pid: String,              // our peer ID
    opt: MeshOptions,
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
            )
            .await?;
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
        let msg_id = msg
            .get("#")
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
        // Generate message ID if not present
        let mut msg = msg.clone();
        if msg.get("#").is_none() {
            msg["#"] = serde_json::Value::String(self.core.random_id(9));
        }

        let raw = serde_json::to_string(&msg)?;

        if let Some(p) = peer {
            self.send_to_peer_by_id(&raw, &p.id).await?;
        } else {
            // Broadcast to all peers - clone IDs first to avoid holding lock during async calls
            let peer_ids: Vec<String> = {
                let peers = self.peers.read().await;
                peers.keys().cloned().collect()
            };

            // Now send to each peer without holding the lock
            for peer_id in peer_ids {
                if let Err(e) = self.send_to_peer_by_id(&raw, &peer_id).await {
                    eprintln!("Error sending to peer {}: {}", peer_id, e);
                    // Continue sending to other peers even if one fails
                }
            }
        }

        Ok(())
    }

    /// Send raw message to a specific peer by ID
    /// Routes through WebSocket connection if available, otherwise queues
    async fn send_to_peer_by_id(&self, raw: &str, peer_id: &str) -> GunResult<()> {
        // Try to get the sender without holding the lock for long
        let tx_opt = {
            let peers = self.peers.read().await;
            if let Some(peer) = peers.get(peer_id) {
                peer.tx.clone() // Clone the Sender to release the lock immediately
            } else {
                None // Peer not found
            }
        };

        if let Some(tx) = tx_opt {
            // Send immediately through WebSocket (no lock held)
            tx.send(raw.to_string()).map_err(|e| {
                crate::error::GunError::Network(format!(
                    "Failed to send to peer {}: {}",
                    peer_id, e
                ))
            })?;
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
        peers.insert(peer_id.clone(), peer);
        drop(peers);

        if was_new {
            let mut near = self.near.write().await;
            *near += 1;
            drop(near);

            // Note: We skip sending the "hi" message here to avoid potential deadlocks
            // The peer will be able to receive messages once the sender is set
            // In Gun.js, the "hi" message exchange might happen differently
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
}
