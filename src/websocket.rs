use crate::core::GunCore;
use crate::dam::{Mesh, Peer};
use crate::error::GunResult;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

/// WebSocket client connection handler
pub struct WebSocketClient {
    core: Arc<GunCore>,
    mesh: Arc<Mesh>,
}

impl WebSocketClient {
    pub fn new(core: Arc<GunCore>, mesh: Arc<Mesh>) -> Self {
        Self { core, mesh }
    }

    /// Connect to a peer URL with automatic reconnection
    /// Returns when connection is established or fails
    pub async fn connect(&self, url: &str) -> GunResult<()> {
        let url_str = url.to_string();
        let core = self.core.clone();
        let mesh = self.mesh.clone();

        // Ensure we use the public URL (convert localhost/127.0.0.1 to public IP if needed)
        let public_url = Self::ensure_public_url(&url_str)?;

        let mut retry_count = 0;
        let max_retries = 10;
        let base_wait = Duration::from_millis(500);

        loop {
            match Self::connect_once(&public_url, core.clone(), mesh.clone()).await {
                Ok(_) => {
                    // Connection successful, wait a bit for handshake and peer registration to complete
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    return Ok(());
                }
                Err(e) => {
                    retry_count += 1;
                    if retry_count >= max_retries {
                        return Err(crate::error::GunError::Network(format!(
                            "Max reconnection attempts reached for {}: {}",
                            public_url, e
                        )));
                    }

                    // Exponential backoff: wait = base_wait * 2^retry_count (capped at 2 seconds)
                    let wait_time = base_wait * (1 << retry_count.min(4));
                    eprintln!(
                        "Connection failed for {}, retrying in {:?}... (attempt {})",
                        public_url, wait_time, retry_count
                    );
                    tokio::time::sleep(wait_time).await;
                }
            }
        }
    }

    /// Ensure URL uses public IP (not localhost/127.0.0.1)
    /// For relay servers, we should always connect via their public IP
    fn ensure_public_url(url: &str) -> GunResult<String> {
        let parsed = Url::parse(url)?;

        // Check if host is localhost or 127.0.0.1
        if let Some(host) = parsed.host_str() {
            if host == "localhost" || host == "127.0.0.1" || host == "::1" {
                // This is a localhost URL - for testing, we might want to allow it
                // but in production, we should fail or resolve to public IP
                // For now, we'll allow it but warn
                eprintln!("Warning: Connecting to localhost URL: {}. This should use a public IP for NAT traversal.", url);
            }
        }

        Ok(url.to_string())
    }

    async fn connect_once(url: &str, core: Arc<GunCore>, mesh: Arc<Mesh>) -> GunResult<()> {
        // Convert http/https to ws/wss
        let ws_url = url
            .replace("http://", "ws://")
            .replace("https://", "wss://");
        let url = Url::parse(&ws_url).map_err(|e| {
            crate::error::GunError::Network(format!("Invalid URL {}: {}", ws_url, e))
        })?;

        // Connect to the WebSocket server (always uses public IP since we're connecting to a remote URL)
        let (ws_stream, response) = connect_async(url.clone()).await.map_err(|e| {
            crate::error::GunError::Network(format!(
                "WebSocket connection failed to {}: {}",
                ws_url, e
            ))
        })?;

        // Verify connection was successful (status 101 Switching Protocols)
        if response.status() != 101 {
            return Err(crate::error::GunError::Network(format!(
                "WebSocket handshake failed with status: {}",
                response.status()
            )));
        }

        let peer = Peer::new(ws_url.clone());
        let peer_id = peer.id.clone();

        // Create channel for sending messages
        let (tx, mut rx) = mpsc::unbounded_channel();

        // Add peer to mesh FIRST (this registers the peer for message routing)
        mesh.hi(peer.clone()).await?;

        // Set sender in mesh AFTER adding peer (so peer exists in the map)
        mesh.set_peer_sender(&peer_id, tx.clone()).await?;

        // Split WebSocket stream
        let (mut write, mut read) = ws_stream.split();

        // Spawn task to handle incoming messages
        let mesh_clone = mesh.clone();
        let peer_for_read = peer.clone();
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Err(e) = mesh_clone.hear(&text, &peer_for_read).await {
                            eprintln!("Error handling message from {}: {}", peer_for_read.id, e);
                        }
                    }
                    Ok(Message::Close(_)) => {
                        break;
                    }
                    Err(e) => {
                        eprintln!("WebSocket error from {}: {}", peer_for_read.id, e);
                        break;
                    }
                    _ => {}
                }
            }
            // Cleanup on disconnect
            let _ = mesh_clone.bye(&peer_for_read.id).await;
        });

        // Spawn task to send outgoing messages
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if write.send(Message::Text(message)).await.is_err() {
                    break;
                }
            }
        });

        Ok(())
    }
}

/// WebSocket server for relay mode
pub struct WebSocketServer {
    core: Arc<GunCore>,
    mesh: Arc<Mesh>,
    port: u16,
}

impl WebSocketServer {
    pub fn new(core: Arc<GunCore>, mesh: Arc<Mesh>, port: u16) -> Self {
        Self { core, mesh, port }
    }

    /// Start the WebSocket server
    pub async fn start(&self) -> GunResult<()> {
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&addr).await?;
        println!("Gun.rs WebSocket server listening on ws://{}", addr);

        while let Ok((stream, addr)) = listener.accept().await {
            let core = self.core.clone();
            let mesh = self.mesh.clone();
            tokio::spawn(Self::handle_connection(stream, addr, core, mesh));
        }

        Ok(())
    }

    async fn handle_connection(
        stream: TcpStream,
        addr: std::net::SocketAddr,
        core: Arc<GunCore>,
        mesh: Arc<Mesh>,
    ) {
        let ws_stream = match accept_async(stream).await {
            Ok(ws) => ws,
            Err(e) => {
                eprintln!("Error accepting WebSocket connection: {}", e);
                return;
            }
        };

        let peer_url = format!("ws://{}", addr);
        let mut peer = Peer::new(peer_url.clone());
        let peer_id = peer.id.clone();

        // Create channel for sending messages
        let (tx, mut rx) = mpsc::unbounded_channel();
        peer.set_sender(tx.clone());

        if let Err(e) = mesh.hi(peer.clone()).await {
            eprintln!("Error adding peer: {}", e);
            return;
        }

        // Set sender in mesh
        if let Err(e) = mesh.set_peer_sender(&peer_id, tx.clone()).await {
            eprintln!("Error setting peer sender: {}", e);
        }

        let (mut write, mut read) = ws_stream.split();

        // Spawn task to handle incoming messages
        let mesh_clone = mesh.clone();
        let peer_for_read = peer.clone();
        let handle_task = tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Err(e) = mesh_clone.hear(&text, &peer_for_read).await {
                            eprintln!("Error handling message: {}", e);
                        }
                    }
                    Ok(Message::Close(_)) => {
                        break;
                    }
                    Err(e) => {
                        eprintln!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        // Spawn task to send outgoing messages
        let send_task = tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if write.send(Message::Text(message)).await.is_err() {
                    break;
                }
            }
        });

        // Wait for either task to complete (connection closed)
        tokio::select! {
            _ = handle_task => {},
            _ = send_task => {},
        }

        // Cleanup on disconnect
        if let Err(e) = mesh.bye(&peer_id).await {
            eprintln!("Error removing peer: {}", e);
        }
    }
}
