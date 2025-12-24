use crate::core::GunCore;
use crate::chain::Chain;
use crate::storage::{SledStorage, LocalStorage, Storage};
use crate::dam::Mesh;
use crate::websocket::{WebSocketClient, WebSocketServer};
use std::sync::Arc;
use crate::error::GunResult;
use tokio::task::JoinHandle;

/// Main Gun instance - entry point for the library
/// Based on Gun.js IGun interface
pub struct Gun {
    core: Arc<GunCore>,
    mesh: Option<Arc<Mesh>>,
    ws_server: Option<JoinHandle<()>>,
}

impl Gun {
    /// Create a new Gun instance
    /// Based on Gun.js Gun() constructor
    pub fn new() -> Self {
        Self {
            core: Arc::new(GunCore::new()),
            mesh: None,
            ws_server: None,
        }
    }

    /// Create a new Gun instance with options
    pub async fn with_options(options: GunOptions) -> GunResult<Self> {
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
            Some(Arc::new(Mesh::new(core.clone())))
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
            for peer_url in &options.peers {
                if let Err(e) = client.connect(peer_url).await {
                    eprintln!("Failed to connect to peer {}: {}", peer_url, e);
                }
            }
        }
        
        Ok(Self {
            core,
            mesh,
            ws_server,
        })
    }

    /// Get a node by key (creates a chain)
    /// Based on Gun.js gun.get(key)
    pub fn get(&self, key: &str) -> Arc<Chain> {
        // Check if key is a soul or needs to be resolved
        // For now, create a chain with the key as soul
        Arc::new(Chain::with_soul(
            self.core.clone(),
            key.to_string(),
            None,
        ))
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
    pub(crate) fn core(&self) -> &Arc<GunCore> {
        &self.core
    }
    
    /// Get the mesh (internal use)
    pub(crate) fn mesh(&self) -> Option<&Arc<Mesh>> {
        self.mesh.as_ref()
    }
}

impl Default for Gun {
    fn default() -> Self {
        Self::new()
    }
}

/// Gun options (configuration)
/// Matches Gun.js opt.peers structure
#[derive(Clone, Debug)]
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
    pub localStorage: bool,
    
    /// Super peer mode (relay server mode)
    /// When true, this peer acts as a relay server for others
    pub super_peer: bool,
    
    /// Port to listen on (for relay server mode)
    pub port: Option<u16>,
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
