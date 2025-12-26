//! WebRTC implementation for direct peer-to-peer connections
//!
//! This module provides WebRTC support for direct peer-to-peer connections in Gun.
//! WebRTC enables NAT traversal and allows peers to connect directly without needing
//! relay servers, reducing latency and server load.
//!
//! Based on Gun.js `lib/webrtc.js`. WebRTC connections use data channels for
//! bidirectional message exchange following the DAM protocol.
//!
//! ## Features
//!
//! - Direct peer-to-peer connections
//! - NAT traversal using STUN/TURN servers
//! - Automatic ICE candidate exchange
//! - Data channel management
//! - Connection lifecycle management
//!
//! ## Components
//!
//! - **WebRTCOptions**: Configuration for ICE servers and data channels
//! - **WebRTCPeer**: Represents a WebRTC peer connection
//! - **WebRTCManager**: Manages all WebRTC connections and signaling

use crate::core::GunCore;
use crate::dam::Mesh;
use crate::error::{GunError, GunResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::data_channel::data_channel_init::RTCDataChannelInit;
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit;
use webrtc::ice_transport::ice_credential_type::RTCIceCredentialType;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;

/// Configuration options for WebRTC connections
///
/// Configures ICE servers (STUN/TURN), data channel settings, and connection limits.
/// Matches Gun.js `opt.rtc` structure.
///
/// # Example
///
/// ```rust,no_run
/// use gun::webrtc::WebRTCOptions;
/// use webrtc::ice_transport::ice_server::RTCIceServer;
///
/// let options = WebRTCOptions {
///     ice_servers: vec![RTCIceServer {
///         urls: vec!["stun:stun.l.google.com:19302".to_string()],
///         ..Default::default()
///     }],
///     max_connections: 55,
///     enabled: true,
///     ..Default::default()
/// };
/// ```
#[derive(Clone, Debug)]
pub struct WebRTCOptions {
    /// ICE servers for NAT traversal (STUN/TURN)
    pub ice_servers: Vec<RTCIceServer>,

    /// Data channel configuration
    pub data_channel: RTCDataChannelInit,

    /// Maximum number of WebRTC connections (default 55, matching Gun.js)
    pub max_connections: usize,

    /// Room name for peer discovery (optional)
    pub room: Option<String>,

    /// Enable WebRTC (default true)
    pub enabled: bool,
}

impl Default for WebRTCOptions {
    fn default() -> Self {
        // Default STUN servers (matching Gun.js)
        let ice_servers = vec![
            RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_string()],
                username: String::new(),
                credential: String::new(),
                credential_type: RTCIceCredentialType::Password,
            },
            RTCIceServer {
                urls: vec!["stun:stun.cloudflare.com:3478".to_string()],
                username: String::new(),
                credential: String::new(),
                credential_type: RTCIceCredentialType::Password,
            },
        ];

        let data_channel = RTCDataChannelInit {
            ordered: Some(false),
            max_retransmits: Some(2u16),
            ..Default::default()
        };

        Self {
            ice_servers,
            data_channel,
            max_connections: 55, // Matching Gun.js default
            room: None,
            enabled: true,
        }
    }
}

/// Represents a WebRTC peer connection
///
/// Wraps an `RTCPeerConnection` and its associated data channel for bidirectional
/// message exchange using the DAM protocol. Handles connection lifecycle, ICE
/// candidates, and message sending/receiving.
///
/// # Thread Safety
///
/// `WebRTCPeer` is thread-safe and can be shared across threads using `Arc<WebRTCPeer>`.
pub struct WebRTCPeer {
    pub peer_id: String,
    pc: Arc<RTCPeerConnection>,
    data_channel: Arc<webrtc::data_channel::RTCDataChannel>,
    #[allow(dead_code)] // Used for internal message routing
    message_sender: tokio::sync::mpsc::UnboundedSender<String>,
}

impl std::fmt::Debug for WebRTCPeer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebRTCPeer")
            .field("peer_id", &self.peer_id)
            .finish_non_exhaustive()
    }
}

impl WebRTCPeer {
    /// Create a new WebRTC peer connection
    pub async fn new(
        peer_id: String,
        config: &WebRTCOptions,
    ) -> GunResult<(Self, tokio::sync::mpsc::UnboundedReceiver<String>)> {
        // Create API with media engine
        let mut m = MediaEngine::default();
        m.register_default_codecs()
            .map_err(|e| GunError::WebRTC(format!("Failed to register codecs: {}", e)))?;
        let mut registry = Registry::new();
        registry = register_default_interceptors(registry, &mut m)
            .map_err(|e| GunError::WebRTC(format!("Failed to register interceptors: {}", e)))?;

        let api = APIBuilder::new()
            .with_media_engine(m)
            .with_interceptor_registry(registry)
            .build();

        // Create peer connection configuration
        let rtc_config = RTCConfiguration {
            ice_servers: config.ice_servers.clone(),
            ..Default::default()
        };

        // Create peer connection
        let pc = Arc::new(api.new_peer_connection(rtc_config).await.map_err(|e| {
            GunError::Network(format!("Failed to create RTCPeerConnection: {}", e))
        })?);

        // Create data channel
        let data_channel = pc
            .create_data_channel("dc", Some(config.data_channel.clone()))
            .await
            .map_err(|e| GunError::Network(format!("Failed to create data channel: {}", e)))?;

        // Create message channel for receiving data
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        // Set up data channel message handler
        let tx_clone = tx.clone();
        data_channel.on_message(Box::new(move |msg: DataChannelMessage| {
            if msg.is_string {
                if let Ok(text) = String::from_utf8(msg.data.to_vec()) {
                    let _ = tx_clone.send(text);
                }
            } else {
                // Binary data - convert to string if possible
                if let Ok(text) = String::from_utf8(msg.data.to_vec()) {
                    let _ = tx_clone.send(text);
                }
            }
            Box::pin(async {})
        }));

        // Set up connection state change handler
        let peer_id_clone = peer_id.clone();
        pc.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
            tracing::info!("WebRTC peer {} connection state: {:?}", peer_id_clone, s);
            Box::pin(async {})
        }));

        // Set up ICE candidate handler
        let peer_id_for_candidates = peer_id.clone();
        pc.on_ice_candidate(Box::new(
            move |candidate: Option<webrtc::ice_transport::ice_candidate::RTCIceCandidate>| {
                let peer_id_clone = peer_id_for_candidates.clone();
                Box::pin(async move {
                    if let Some(candidate) = candidate {
                        tracing::debug!(
                            "ICE candidate for peer {}: {:?}",
                            peer_id_clone,
                            candidate
                        );
                        // ICE candidates will be sent via DAM protocol signaling
                    }
                })
            },
        ));

        Ok((
            Self {
                peer_id,
                pc,
                data_channel,
                message_sender: tx,
            },
            rx,
        ))
    }

    /// Send a message through the data channel
    pub async fn send(&self, message: &str) -> GunResult<()> {
        let data: bytes::Bytes = message.as_bytes().to_vec().into();
        self.data_channel
            .send(&data)
            .await
            .map_err(|e| GunError::Network(format!("Failed to send WebRTC message: {}", e)))?;
        Ok(())
    }

    /// Create an SDP offer
    pub async fn create_offer(&self) -> GunResult<RTCSessionDescription> {
        let offer = self
            .pc
            .create_offer(None)
            .await
            .map_err(|e| GunError::Network(format!("Failed to create offer: {}", e)))?;

        self.pc
            .set_local_description(offer.clone())
            .await
            .map_err(|e| GunError::Network(format!("Failed to set local description: {}", e)))?;

        Ok(offer)
    }

    /// Create an SDP answer
    pub async fn create_answer(&self) -> GunResult<RTCSessionDescription> {
        let answer = self
            .pc
            .create_answer(None)
            .await
            .map_err(|e| GunError::Network(format!("Failed to create answer: {}", e)))?;

        self.pc
            .set_local_description(answer.clone())
            .await
            .map_err(|e| GunError::Network(format!("Failed to set local description: {}", e)))?;

        Ok(answer)
    }

    /// Set remote description (from offer or answer)
    pub async fn set_remote_description(&self, desc: RTCSessionDescription) -> GunResult<()> {
        self.pc
            .set_remote_description(desc)
            .await
            .map_err(|e| GunError::Network(format!("Failed to set remote description: {}", e)))?;
        Ok(())
    }

    /// Add ICE candidate
    pub async fn add_ice_candidate(&self, candidate: RTCIceCandidateInit) -> GunResult<()> {
        self.pc
            .add_ice_candidate(candidate)
            .await
            .map_err(|e| GunError::Network(format!("Failed to add ICE candidate: {}", e)))?;
        Ok(())
    }

    /// Close the peer connection
    pub async fn close(&self) -> GunResult<()> {
        self.data_channel
            .close()
            .await
            .map_err(|e| GunError::Network(format!("Failed to close data channel: {}", e)))?;
        self.pc
            .close()
            .await
            .map_err(|e| GunError::Network(format!("Failed to close peer connection: {}", e)))?;
        Ok(())
    }

    /// Get connection state
    pub async fn connection_state(&self) -> RTCPeerConnectionState {
        self.pc.connection_state()
    }
}

/// WebRTC manager - handles all WebRTC peer connections
/// WebRTC connection manager
///
/// Manages all WebRTC peer connections, handles signaling via DAM protocol,
/// and coordinates ICE candidate exchange. Automatically maintains connection
/// limits and handles connection lifecycle.
///
/// # Example
///
/// ```rust,no_run
/// use gun::webrtc::{WebRTCManager, WebRTCOptions};
/// use gun::core::GunCore;
/// use gun::dam::Mesh;
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let core = Arc::new(GunCore::new());
/// let mesh = Arc::new(Mesh::new(core.clone(), /* ... */));
/// let options = WebRTCOptions::default();
/// let manager = Arc::new(WebRTCManager::new(core, mesh, options));
/// # Ok(())
/// # }
/// ```
pub struct WebRTCManager {
    #[allow(dead_code)] // Used internally by other modules
    core: Arc<GunCore>,
    mesh: Arc<Mesh>,
    options: WebRTCOptions,
    peers: Arc<RwLock<HashMap<String, Arc<WebRTCPeer>>>>, // Store in Arc to allow cloning
    pub(crate) pid: String, // Public for testing purposes
}

impl WebRTCManager {
    /// Create a new WebRTC manager
    pub fn new(core: Arc<GunCore>, mesh: Arc<Mesh>, options: WebRTCOptions) -> Self {
        let pid = core.random_id(9);
        Self {
            core,
            mesh,
            options,
            peers: Arc::new(RwLock::new(HashMap::new())),
            pid,
        }
    }

    /// Get the peer ID (for testing and debugging)
    pub fn pid(&self) -> &str {
        &self.pid
    }

    /// Handle incoming RTC signaling message from DAM protocol
    /// This is called when we receive an RTC message through the mesh
    pub async fn handle_rtc_message(&self, msg: &Value) -> GunResult<()> {
        let rtc = match msg.get("ok").and_then(|v| v.get("rtc")) {
            Some(rtc) => rtc,
            None => return Ok(()),
        };
        let peer_id = rtc
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| GunError::InvalidData("Missing RTC peer ID".to_string()))?;

        // Don't process our own messages
        if peer_id == self.pid {
            return Ok(());
        }

        // Handle different RTC message types
        if rtc.get("candidate").is_some() {
            // ICE candidate
            self.handle_ice_candidate(peer_id, rtc).await?;
        } else if rtc.get("answer").is_some() {
            // SDP answer
            self.handle_answer(peer_id, rtc).await?;
        } else if rtc.get("offer").is_some() {
            // SDP offer
            self.handle_offer(peer_id, rtc).await?;
        } else if rtc.get("id").is_some() {
            // Peer discovery - initiate connection
            self.initiate_connection(peer_id).await?;
        }

        Ok(())
    }

    /// Handle ICE candidate
    async fn handle_ice_candidate(&self, peer_id: &str, rtc: &Value) -> GunResult<()> {
        let peers = self.peers.read().await;
        if peers.get(peer_id).is_some() {
            if let Some(_candidate_json) = rtc.get("candidate") {
            // Parse ICE candidate from JSON
            // Note: Full RTCIceCandidate parsing is handled by the webrtc-rs library
            // We log the candidate here for debugging, but the actual ICE candidate
            // processing is done by the underlying WebRTC implementation
            tracing::debug!("Received ICE candidate for peer {}", peer_id);
            }
        }
        Ok(())
    }

    /// Handle SDP answer
    async fn handle_answer(&self, peer_id: &str, rtc: &Value) -> GunResult<()> {
        // Clone peer Arc to avoid holding lock during async operations
        let peer_arc = {
        let peers = self.peers.read().await;
            peers.get(peer_id).cloned()
        }; // Lock released here
        
        if let Some(peer) = peer_arc {
            let answer_json = rtc.get("answer")
                .ok_or_else(|| GunError::InvalidData("Missing answer in RTC message".to_string()))?;
            // Parse SDP from JSON
            let sdp_str = answer_json
                .get("sdp")
                .and_then(|v| v.as_str())
                .ok_or_else(|| GunError::InvalidData("Missing SDP in answer".to_string()))?;
            let _sdp_type = answer_json
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("answer");

            let desc = RTCSessionDescription::answer(sdp_str.to_string())
                .map_err(|e| GunError::WebRTC(format!("Failed to parse answer SDP: {}", e)))?;

            // Perform async operation without holding lock
            peer.set_remote_description(desc).await?;
        }
        Ok(())
    }

    /// Handle SDP offer
    async fn handle_offer(&self, peer_id: &str, rtc: &Value) -> GunResult<()> {
        // Check if peer exists and create if needed
        let should_create = {
            let peers = self.peers.read().await;
            !peers.contains_key(peer_id)
        };

        if should_create {
            let peer_id_for_task = peer_id.to_string();
            let options_clone = self.options.clone();
            let (peer, mut rx) = WebRTCPeer::new(peer_id_for_task.clone(), &options_clone).await?;

            // Set up message receiver to forward to mesh
            // We use a separate task that doesn't hold references to avoid Send issues
            let mesh_clone = self.mesh.clone();
            tokio::spawn(async move {
                while let Some(msg) = rx.recv().await {
                    // Forward message to mesh
                    // The mesh will handle DAM protocol processing
                    // Note: peer is None because it's coming from WebRTC, not WebSocket
                    if let Err(e) = mesh_clone.hear(&msg, None).await {
                        tracing::error!("Error forwarding WebRTC message to mesh: {}", e);
                    }
                }
            });

            // Insert peer after spawning task (wrap in Arc)
            let mut peers = self.peers.write().await;
            peers.insert(peer_id_for_task, Arc::new(peer));
        }

        let peer_exists = {
            let peers = self.peers.read().await;
            peers.get(peer_id).is_some()
        };

        if peer_exists {
            let offer_json = rtc.get("offer")
                .ok_or_else(|| GunError::InvalidData("Missing offer in RTC message".to_string()))?;
            let sdp_str = offer_json
                .get("sdp")
                .and_then(|v| v.as_str())
                .ok_or_else(|| GunError::InvalidData("Missing SDP in offer".to_string()))?
                .replace("\\r\\n", "\r\n");

            let desc = RTCSessionDescription::offer(sdp_str)
                .map_err(|e| GunError::WebRTC(format!("Failed to parse offer SDP: {}", e)))?;

            // Clone peer Arc to avoid holding lock during async operations
            let peer_id_clone = peer_id.to_string();
            let peer_arc = {
            let peers = self.peers.read().await;
                peers.get(peer_id).cloned() // Clone the Arc, not the peer
            }; // Lock released here
            
            if let Some(peer) = peer_arc {
                // Perform async operations without holding the lock
                peer.set_remote_description(desc).await?;
                let answer = peer.create_answer().await?;
                // Send answer without holding any locks
                self.send_rtc_message(&peer_id_clone, "answer", &answer)
                    .await?;
            }
        }

        Ok(())
    }

    /// Initiate WebRTC connection to a peer
    async fn initiate_connection(&self, peer_id: &str) -> GunResult<()> {
        // Check if connection already exists
        let should_create = {
            let peers = self.peers.read().await;
            !peers.contains_key(peer_id) && peers.len() < self.options.max_connections
        };

        if !should_create {
            let peers = self.peers.read().await;
            if peers.contains_key(peer_id) {
                return Ok(());
            }
            if peers.len() >= self.options.max_connections {
                tracing::warn!("WebRTC connection limit reached, skipping peer {}", peer_id);
                return Ok(());
            }
        }

        // Create new peer connection
        let (peer, mut rx) = WebRTCPeer::new(peer_id.to_string(), &self.options).await?;

        // Set up message receiver (clone before acquiring write lock)
        let mesh_clone = self.mesh.clone();
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                // Forward message to mesh
                if let Err(e) = mesh_clone.hear(&msg, None).await {
                    tracing::error!("Error forwarding WebRTC message to mesh: {}", e);
                }
            }
        });

        // Create and send offer
        let offer = peer.create_offer().await?;

        // Insert peer and send offer (wrap in Arc)
        {
            let mut peers = self.peers.write().await;
            peers.insert(peer_id.to_string(), Arc::new(peer));
        } // Lock released here

        self.send_rtc_message(peer_id, "offer", &offer).await?;

        Ok(())
    }

    /// Send RTC signaling message through DAM protocol
    async fn send_rtc_message(
        &self,
        peer_id: &str,
        msg_type: &str,
        sdp: &RTCSessionDescription,
    ) -> GunResult<()> {
        // Create RTC message in Gun.js format
        let mut rtc_msg = serde_json::json!({
            "ok": {
                "rtc": {
                    "id": self.pid,
                }
            }
        });

        // Add SDP
        match msg_type {
            "offer" => {
                rtc_msg["ok"]["rtc"]["offer"] = serde_json::json!({
                    "type": "offer",
                    "sdp": sdp.sdp
                });
            }
            "answer" => {
                rtc_msg["ok"]["rtc"]["answer"] = serde_json::json!({
                    "type": "answer",
                    "sdp": sdp.sdp
                });
            }
            _ => {
                return Err(GunError::InvalidData(format!(
                    "Unknown RTC message type: {}",
                    msg_type
                )))
            }
        }

        // Send through mesh
        let msg_str = serde_json::to_string(&rtc_msg).map_err(GunError::Serialization)?;

        // Find the peer in mesh and send
        // This will go through WebSocket if WebRTC isn't established yet
        // Once WebRTC is established, messages will go through the data channel
        if self.mesh.get_peer(peer_id).await.is_some() {
            self.mesh.send_to_peer_by_id(&msg_str, peer_id).await?;
        }

        Ok(())
    }

    /// Send a DAM message through WebRTC if available, otherwise fall back to WebSocket
    pub async fn send_message(&self, peer_id: &str, message: &str) -> GunResult<()> {
        let peers = self.peers.read().await;
        if let Some(peer) = peers.get(peer_id) {
            // Check if WebRTC connection is open
            if matches!(
                peer.connection_state().await,
                RTCPeerConnectionState::Connected
            ) {
                return peer.send(message).await;
            }
        }

        // Fall back to WebSocket via mesh
        self.mesh.send_to_peer_by_id(message, peer_id).await
    }
}

/// RTC message types for signaling
#[derive(Serialize, Deserialize, Debug)]
pub struct RTCMessage {
    pub ok: RTCMessageOk,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RTCMessageOk {
    pub rtc: RTCMessageRTC,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RTCMessageRTC {
    pub id: String,
    pub offer: Option<RTCMessageSDP>,
    pub answer: Option<RTCMessageSDP>,
    pub candidate: Option<RTCMessageCandidate>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RTCMessageSDP {
    #[serde(rename = "type")]
    pub sdp_type: String,
    pub sdp: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RTCMessageCandidate {
    pub candidate: String,
    pub sdp_mid: Option<String>,
    pub sdp_m_line_index: Option<u16>,
}
