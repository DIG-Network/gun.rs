use crate::chain::Chain;
use crate::core::GunCore;
use crate::dam::Mesh;
use crate::error::GunResult;
use crate::storage::{LocalStorage, SledStorage, Storage};
use crate::webrtc::{WebRTCManager, WebRTCOptions};
use crate::websocket::{WebSocketClient, WebSocketServer};
use chia_bls::{PublicKey, SecretKey};
use std::sync::Arc;
use tokio::task::JoinHandle;

/// Main Gun instance - entry point for the library
/// 
/// This is the primary interface for interacting with the Gun database.
/// Based on Gun.js IGun interface, providing a realtime, decentralized, offline-first graph database.
/// 
/// # Features
/// - **Realtime**: Automatic synchronization with peers
/// - **Decentralized**: P2P mesh networking with no central server required
/// - **Offline-first**: Works locally, syncs when connected
/// - **Graph database**: Store data as a graph with nodes and properties
/// 
/// # Usage
/// 
/// ```rust,no_run
/// use gun::{Gun, GunOptions};
/// use serde_json::json;
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create a local instance
/// let gun = Gun::new();
/// 
/// // Put data
/// gun.get("user").put(json!({"name": "Alice", "age": 30})).await?;
/// 
/// // Read data
/// gun.get("user").once(|data, _key| {
///     println!("User data: {:?}", data);
/// }).await?;
/// 
/// // Subscribe to updates
/// gun.get("user").on(|data, _key| {
///     println!("User updated: {:?}", data);
/// });
/// 
/// // Connect to peers
/// let options = GunOptions {
///     peers: vec!["ws://relay.example.com/gun".to_string()],
///     ..Default::default()
/// };
/// let gun_with_peers = Gun::with_options(options).await?;
/// # Ok(())
/// # }
/// ```
/// 
/// # Error Handling
/// 
/// Most operations return `GunResult<T>` which is `Result<T, GunError>`.
/// Errors can occur due to:
/// - Network failures (connection issues, timeouts)
/// - Storage errors (disk full, permission denied)
/// - Invalid data (malformed JSON, invalid soul references)
/// - Serialization errors
/// 
/// All errors implement `std::error::Error` and can be converted to strings.
/// 
/// Based on Gun.js IGun interface
pub struct Gun {
    core: Arc<GunCore>,
    mesh: Option<Arc<Mesh>>,
    ws_server: Option<JoinHandle<()>>, // Server handle for graceful shutdown
    #[allow(dead_code)] // Used internally for WebRTC signaling
    webrtc_manager: Option<Arc<WebRTCManager>>, // WebRTC manager for direct P2P connections
    secret_key: SecretKey, // BLS secret key for signing outgoing messages
    public_key: PublicKey, // BLS public key for verifying incoming messages
}

impl Gun {
    /// Create a new Gun instance with default settings
    /// 
    /// Creates a local-only Gun instance without network connectivity or persistent storage.
    /// Use `with_options()` to configure peers, storage, and other settings.
    /// 
    /// # Arguments
    /// * `secret_key` - BLS secret key for signing outgoing messages
    /// * `public_key` - BLS public key for verifying incoming messages (typically derived from secret_key)
    /// 
    /// # Returns
    /// A new `Gun` instance ready for local operations.
    /// 
    /// # Example
    /// ```rust,no_run
    /// use gun::Gun;
    /// use chia_bls::{SecretKey, PublicKey};
    /// 
    /// let secret_key = SecretKey::from_seed(&[0u8; 32]);
    /// let public_key = secret_key.public_key();
    /// let gun = Gun::new(secret_key, public_key);
    /// // Ready to use for local operations
    /// ```
    /// 
    /// Based on Gun.js Gun() constructor
    pub fn new(secret_key: SecretKey, public_key: PublicKey) -> Self {
        Self {
            core: Arc::new(GunCore::new()),
            mesh: None,
            ws_server: None,
            webrtc_manager: None,
            secret_key,
            public_key,
        }
    }

    /// Create a new Gun instance with custom options
    /// 
    /// Configures the Gun instance with peers, storage, WebRTC, and other settings.
    /// 
    /// # Arguments
    /// * `secret_key` - BLS secret key for signing outgoing messages
    /// * `public_key` - BLS public key for verifying incoming messages (typically derived from secret_key)
    /// * `options` - Configuration options including:
    ///   - `peers`: List of WebSocket URLs to connect to
    ///   - `storage_path`: Path for persistent storage
    ///   - `localStorage`: Enable localStorage-like file storage
    ///   - `radisk`: Use SledStorage (more efficient for large datasets)
    ///   - `port`: Port for super peer mode (relay server)
    ///   - `super_peer`: Enable super peer mode
    ///   - `webrtc`: WebRTC configuration for direct P2P connections
    /// 
    /// # Returns
    /// Returns `Ok(Gun)` if initialization succeeds, or `GunError` if there's an error.
    /// 
    /// # Errors
    /// - `GunError::Storage`: If storage initialization fails
    /// - `GunError::Network`: If peer connection setup fails
    /// - `GunError::Io`: If file operations fail
    /// 
    /// # Example
    /// ```rust,no_run
    /// use gun::{Gun, GunOptions};
    /// use chia_bls::{SecretKey, PublicKey};
    /// 
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let secret_key = SecretKey::from_seed(&[0u8; 32]);
    /// let public_key = secret_key.public_key();
    /// let options = GunOptions {
    ///     peers: vec!["ws://relay.example.com/gun".to_string()],
    ///     storage_path: Some("./gun_data".to_string()),
    ///     localStorage: true,
    ///     ..Default::default()
    /// };
    /// 
    /// let gun = Gun::with_options(secret_key, public_key, options).await?;
    /// // Gun instance is ready with peers and storage
    /// # Ok(())
    /// # }
    /// ```
    pub async fn with_options(secret_key: SecretKey, public_key: PublicKey, options: GunOptions) -> GunResult<Self> {
        let core = if options.localStorage || options.storage_path.is_some() {
            let storage: Arc<dyn Storage> = if let Some(ref storage_path) = options.storage_path {
                if options.radisk {
                    // Use SledStorage for radisk mode (more efficient for large datasets)
                    Arc::new(SledStorage::new(storage_path)?)
                } else {
                    // Use LocalStorage (simpler, file-based, localStorage-like)
                    Arc::new(LocalStorage::new(storage_path)?)
                }
            } else {
                // Default localStorage location
                let default_path = "./gun_data";
                Arc::new(LocalStorage::new(default_path)?)
            };
            Arc::new(GunCore::with_storage(storage))
        } else {
            Arc::new(GunCore::new())
        };

        // Create mesh if we have peers or are a super peer
        let mesh = if !options.peers.is_empty() || options.super_peer {
            Some(Arc::new(Mesh::new(core.clone(), secret_key.clone(), public_key.clone())))
        } else {
            None
        };

        let mut ws_server = None;

        // Start WebSocket server if in super peer mode
        if let (Some(ref mesh_ref), Some(port)) = (&mesh, options.port) {
            let server = WebSocketServer::new(core.clone(), mesh_ref.clone(), port);
            let server_clone = server;
            let handle = tokio::spawn(async move {
                if let Err(e) = server_clone.start().await {
                    eprintln!("WebSocket server error: {}", e);
                }
            });
            ws_server = Some(handle);
        }

        // Connect to peer URLs
        if let Some(ref mesh_ref) = mesh {
            let client = WebSocketClient::new(core.clone(), mesh_ref.clone());
            // Connect to all peers (always through public IPs for NAT traversal)
            for peer_url in &options.peers {
                match client.connect(peer_url).await {
                    Ok(_) => {
                        println!("Successfully connected to peer: {}", peer_url);
                    }
                    Err(e) => {
                        eprintln!("Failed to connect to peer {}: {}", peer_url, e);
                        // Continue trying other peers even if one fails
                    }
                }
            }
        }

        // Initialize WebRTC manager if enabled and set it in the mesh
        let webrtc_manager = if options.webrtc.enabled {
            if let Some(ref mesh_ref) = mesh {
                let manager = Arc::new(WebRTCManager::new(
                    core.clone(),
                    mesh_ref.clone(),
                    options.webrtc.clone(),
                ));
                Some(manager)
            } else {
                None
            }
        } else {
            None
        };

        // Set up event listeners for network sync
        if let Some(ref mesh_ref) = mesh {
            let mesh_clone = mesh_ref.clone();
            
            // Listen for network_sync events (emitted by emit_update) and send to peers
            let mesh_for_sync = mesh_clone.clone();
            let core_for_sync = core.clone();
            core.events.on("network_sync", Box::new(move |event: &crate::events::Event| {
                if let Some(soul) = event.data.get("soul").and_then(|v| v.as_str()) {
                    if let Some(data) = event.data.get("data") {
                        // Gun.js expects: { put: { soul: { _: { "#": soul, ">": states }, ...data } } }
                        // The soul must be a KEY in the put object, not a field
                        let core_clone = core_for_sync.clone();
                        let mesh_send = mesh_for_sync.clone();
                        let soul_str = soul.to_string();
                        let data_clone = data.clone();
                        tokio::spawn(async move {
                            // Get the node from graph to include state information
                            if let Some(node) = core_clone.graph.get(&soul_str) {
                                // Build node object with metadata
                                let mut node_obj = serde_json::Map::new();
                                
                                // Add metadata object "_" from node.meta
                                // Gun.js expects: { _: { "#": soul, ">": { key1: state1, key2: state2 } } }
                                if let Some(meta_obj) = node.meta.get("_").and_then(|v| v.as_object()) {
                                    // Use existing meta if available
                                    let mut meta = serde_json::Map::new();
                                    for (k, v) in meta_obj {
                                        meta.insert(k.clone(), v.clone());
                                    }
                                    // Ensure "#" is set
                                    meta.insert("#".to_string(), serde_json::Value::String(soul_str.clone()));
                                    node_obj.insert("_".to_string(), serde_json::Value::Object(meta));
                                } else {
                                    // Build meta from node.meta
                                    let mut meta = serde_json::Map::new();
                                    meta.insert("#".to_string(), serde_json::Value::String(soul_str.clone()));
                                    // Add state map if available
                                    if let Some(states) = node.meta.get(">") {
                                        meta.insert(">".to_string(), states.clone());
                                    }
                                    node_obj.insert("_".to_string(), serde_json::Value::Object(meta));
                                }
                                
                                // Add data fields
                                if let Some(data_obj) = data_clone.as_object() {
                                    for (key, value) in data_obj {
                                        node_obj.insert(key.clone(), value.clone());
                                    }
                                }
                                
                                // Build put message: { put: { soul: node_obj } }
                                // Gun.js expects the soul to be a KEY, not a field
                                let mut put_obj = serde_json::Map::new();
                                put_obj.insert(soul_str.clone(), serde_json::Value::Object(node_obj));
                                
                                let msg = serde_json::json!({
                                    "put": serde_json::Value::Object(put_obj)
                                });
                                eprintln!("DEBUG: Sending put message to peers (Gun.js format): {}", serde_json::to_string(&msg).unwrap_or_default());
                                
                                if let Err(e) = mesh_send.say(&msg, None).await {
                                    eprintln!("Error sending network_sync to peers: {}", e);
                                }
                            }
                        });
                    }
                }
            }));

            // Listen for get_request events and send them to peers
            let mesh_for_get = mesh_clone.clone();
            core.events.on("get_request", Box::new(move |event: &crate::events::Event| {
                // Forward get request to peers
                if let Some(get_data) = event.data.get("get") {
                    let msg = serde_json::json!({
                        "get": get_data
                    });
                    let mesh_send = mesh_for_get.clone();
                    let msg_send = msg.clone();
                    tokio::spawn(async move {
                        if let Err(e) = mesh_send.say(&msg_send, None).await {
                            eprintln!("Error sending get_request to peers: {}", e);
                        }
                    });
                }
            }));
        }

        Ok(Self {
            core,
            mesh,
            ws_server,
            webrtc_manager,
            secret_key,
            public_key,
        })
    }

    /// Get a node by key (creates a chain)
    /// Based on Gun.js gun.get(key)
    pub fn get(&self, key: &str) -> Arc<Chain> {
        // Check if key is a soul or needs to be resolved
        // For now, create a chain with the key as soul
        Arc::new(Chain::with_soul(self.core.clone(), key.to_string(), None))
    }

    /// Get the root chain
    pub fn root(&self) -> Arc<Chain> {
        Arc::new(Chain::new(self.core.clone()))
    }

    /// Get state timestamp (for testing/debugging)
    pub fn state(&self) -> f64 {
        self.core.state.next()
    }

    /// Get the core (internal use)
    /// Kept for potential future internal module access
    #[allow(dead_code)]
    pub(crate) fn core(&self) -> &Arc<GunCore> {
        &self.core
    }

    /// Get the mesh (internal use)
    /// Kept for potential future internal module access
    #[allow(dead_code)]
    pub(crate) fn mesh(&self) -> Option<&Arc<Mesh>> {
        self.mesh.as_ref()
    }

    /// Get the number of connected peers
    pub async fn connected_peer_count(&self) -> usize {
        if let Some(ref mesh) = self.mesh {
            mesh.connected_peer_count().await
        } else {
            0
        }
    }

    /// Check if any peers are connected
    pub async fn is_connected(&self) -> bool {
        if let Some(ref mesh) = self.mesh {
            mesh.has_connected_peers().await
        } else {
            false
        }
    }

    /// Wait for at least one peer connection to be established
    /// Returns true if connection was established, false if timeout was reached
    pub async fn wait_for_connection(&self, timeout_ms: u64) -> bool {
        if let Some(ref mesh) = self.mesh {
            mesh.wait_for_connection(timeout_ms).await
        } else {
            false
        }
    }

    /// Gracefully shutdown the Gun instance
    /// Closes the WebSocket server and cleans up resources
    pub async fn shutdown(&mut self) -> GunResult<()> {
        // Abort the WebSocket server task if running
        if let Some(handle) = self.ws_server.take() {
            handle.abort();
            // Wait a bit for the task to finish cleanup
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        // Mesh connections will be cleaned up automatically when dropped
        // For explicit cleanup, we could add a mesh.shutdown() method if needed

        Ok(())
    }
}

// Note: Default implementation removed because Gun now requires BLS key pair
// Users must explicitly provide secret_key and public_key

/// Gun options (configuration)
/// Matches Gun.js opt.peers structure
#[derive(Clone, Debug)]
#[allow(non_snake_case)] // localStorage matches Gun.js API
pub struct GunOptions {
    /// Peer URLs (relay servers or other peers)
    /// Can be WebSocket URLs like: "ws://example.com/gun" or "wss://example.com/gun"
    /// These are OPTIONAL - Gun works fully P2P without any relays
    /// Relays are just helpful peers for NAT traversal and connectivity
    pub peers: Vec<String>,

    /// Storage path for persistent storage (sled)
    pub storage_path: Option<String>,

    /// Enable radisk storage (persistent storage on disk)
    pub radisk: bool,

    /// Enable localStorage (browser equivalent - not applicable in Rust, kept for API compatibility)
    #[allow(non_snake_case)] // Matches Gun.js API naming convention
    pub localStorage: bool,

    /// Super peer mode (relay server mode)
    /// When true, this peer acts as a relay server for others
    pub super_peer: bool,

    /// Port to listen on (for relay server mode)
    pub port: Option<u16>,

    /// WebRTC configuration for direct peer-to-peer connections
    pub webrtc: WebRTCOptions,
}

impl Default for GunOptions {
    fn default() -> Self {
        Self {
            peers: vec![],
            storage_path: None,
            radisk: true,
            localStorage: true,
            super_peer: false,
            port: None,
            webrtc: WebRTCOptions::default(),
        }
    }
}

impl GunOptions {
    /// Create options with a relay server
    pub fn with_relay(relay_url: &str) -> Self {
        Self {
            peers: vec![relay_url.to_string()],
            ..Default::default()
        }
    }

    /// Create options with multiple peers (relays)
    pub fn with_peers(peers: Vec<String>) -> Self {
        Self {
            peers,
            ..Default::default()
        }
    }

    /// Create options for relay server mode
    pub fn relay_server(port: u16) -> Self {
        Self {
            port: Some(port),
            super_peer: true,
            ..Default::default()
        }
    }
}
