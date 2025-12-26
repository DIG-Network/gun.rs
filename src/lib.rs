//! # Gun.rs - A Real-time, Decentralized, Offline-First Graph Database
//!
//! Gun.rs is a Rust implementation of the [Gun.js](https://gun.eco/) protocol, providing a
//! real-time, decentralized, offline-first graph database. It enables peer-to-peer data
//! synchronization without requiring a central server.
//!
//! ## Features
//!
//! - **Real-time Synchronization**: Automatic data synchronization with connected peers
//! - **Decentralized**: P2P mesh networking - no central server required
//! - **Offline-First**: Works locally and syncs when connected
//! - **Graph Database**: Store data as a graph with nodes and relationships
//! - **Conflict Resolution**: Automatic conflict resolution using state timestamps (HAM algorithm)
//! - **Cryptographic Security**: BLS signatures for message authentication
//! - **Pluggable Storage**: Memory, file-based (localStorage-like), or Sled database
//! - **WebRTC Support**: Direct peer-to-peer connections with NAT traversal
//! - **WebSocket Support**: Relay server connections for NAT traversal
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use gun::{Gun, GunOptions};
//! use chia_bls::{SecretKey, PublicKey};
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Generate BLS key pair
//!     let secret_key = SecretKey::from_seed(&[0u8; 32]);
//!     let public_key = secret_key.public_key();
//!
//!     // Create a local Gun instance
//!     let gun = Gun::new(secret_key, public_key);
//!
//!     // Store data
//!     gun.get("user").put(json!({
//!         "name": "Alice",
//!         "age": 30
//!     })).await?;
//!
//!     // Read data once
//!     gun.get("user").once(|data, _key| {
//!         println!("User: {:?}", data);
//!     }).await?;
//!
//!     // Subscribe to updates
//!     gun.get("user").on(|data, _key| {
//!         println!("User updated: {:?}", data);
//!     });
//!
//!     // Connect to peers (optional)
//!     let options = GunOptions {
//!         peers: vec!["ws://relay.example.com/gun".to_string()],
//!         ..Default::default()
//!     };
//!     let gun_with_peers = Gun::with_options(secret_key, public_key, options).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! ### Core Components
//!
//! - **[`Gun`](gun::Gun)**: Main entry point - creates and manages Gun instances
//! - **[`Chain`](chain::Chain)**: Fluent API for data operations (get, put, on, once, map, set)
//! - **[`GunCore`](core::GunCore)**: Core engine managing graph, state, events, and storage
//! - **[`Graph`](graph::Graph)**: In-memory graph storage with conflict resolution
//! - **[`Mesh`](dam::Mesh)**: DAM protocol implementation for P2P message routing
//! - **[`Storage`](storage::Storage)**: Pluggable storage backend trait
//!
//! ### Data Model
//!
//! Gun uses a graph data model where:
//! - **Nodes**: Identified by a unique "soul" (UUID-like string)
//! - **Properties**: Key-value pairs on nodes
//! - **References**: Links between nodes using soul references `{"#": "soul_id"}`
//! - **State**: Timestamps for conflict resolution
//!
//! ### Conflict Resolution
//!
//! Gun uses the HAM (Hypothetical Amnesia Machine) algorithm:
//! - Each property update gets a state timestamp
//! - Higher state wins in conflicts
//! - Automatic merge of non-conflicting updates
//!
//! ### Network Protocol
//!
//! Gun uses the DAM (Directed Acyclic Mesh) protocol:
//! - Message deduplication to prevent loops
//! - Automatic routing through peer mesh
//! - Support for WebSocket relays and WebRTC direct connections
//!
//! ## Examples
//!
//! See the `examples/` directory for comprehensive examples including:
//! - Basic operations (put, get, once, on)
//! - Chain operations (get, put, back, map, set)
//! - Network synchronization
//! - Storage backends
//! - WebRTC connections
//! - Certificate-based security (SEA)
//!
//! ## Security
//!
//! Gun.rs supports cryptographic security through:
//! - **BLS Signatures**: All messages are signed and verified
//! - **SEA (Security, Encryption, Authorization)**: Certificate-based access control
//! - **Message Predicates**: Custom filtering logic for incoming messages
//!
//! ## Performance
//!
//! - In-memory graph for fast reads
//! - Persistent storage options for durability
//! - Message batching and deduplication
//! - Efficient conflict resolution algorithm
//!
//! ## Compatibility
//!
//! Gun.rs aims to be compatible with Gun.js:
//! - Same protocol and message format
//! - Compatible graph structure
//! - Interoperable with JavaScript peers
//!
//! ## License
//!
//! See the LICENSE file for license information.
//!
//! ## Links
//!
//! - [Gun.js Documentation](https://gun.eco/docs)
//! - [Gun.js GitHub](https://github.com/amark/gun)

pub mod chain;
pub mod core;
pub mod dam;
pub mod dup;
pub mod error;
pub mod events;
pub mod graph;
pub mod gun;
pub mod sea;
pub mod state;
pub mod storage;
pub mod types;
pub mod valid;
pub mod webrtc;
pub mod websocket;

pub use chain::Chain;
pub use error::GunError;
pub use gun::{Gun, GunOptions};
pub use sea::*;
pub use types::MessagePredicate;
pub use valid::valid;
pub use valid::{is_valid_data, valid_soul};
pub use webrtc::{WebRTCManager, WebRTCOptions, WebRTCPeer};

#[cfg(test)]
mod tests {
    use super::*;
    use chia_bls::{SecretKey, PublicKey};

    #[tokio::test]
    async fn test_basic_put_get() {
        // Generate BLS key pair for testing
        let secret_key = SecretKey::from_seed(&[0u8; 32]);
        let public_key = secret_key.public_key();
        
        let gun = Gun::new(secret_key, public_key);
        let chain = gun.get("test");
        // In tests, unwrap is acceptable for error handling
        chain
            .put(serde_json::json!({"name": "test"}))
            .await
            .expect("put should succeed in test");

        let mut called = false;
        chain
            .once(|data, _key| {
                called = true;
                assert!(data.is_object());
            })
            .await
            .expect("once should succeed in test");
        assert!(called);
    }
}
