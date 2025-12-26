# Gun.rs

A Rust port of [Gun.js](https://github.com/amark/gun) - a realtime, decentralized, offline-first, graph data synchronization engine.

## ⚠️ Important Disclaimer

**This is an unofficial, vibe-coded port of Gun.js to Rust. This project is:**
- **NOT for production use** - Do not use in production environments
- **NOT actively maintained** - This is an experimental port
- **For testing purposes only** - Use at your own risk
- **No guarantees** - No warranties, no support, use at your own risk

This port was created for experimentation and learning purposes. If you need a production-ready solution, please use the official [Gun.js](https://github.com/amark/gun) implementation.

---

## Table of Contents

1. [Key Crate Concepts](#key-crate-concepts)
2. [Crate Usage](#crate-usage)
3. [Exhaustive Reference Guide](#exhaustive-reference-guide)
4. [Examples](#examples)
5. [Additional Resources](#additional-resources)

---

## Key Crate Concepts

### Graph Database Model

Gun.rs stores data as a **directed graph** where:
- **Nodes** are identified by unique **souls** (self-describing unique IDs)
- **Properties** are key-value pairs stored on nodes
- **Relationships** are formed through soul references
- **No schema required** - data is flexible and dynamic

```rust
// Example graph structure:
// {
//   "user:alice": {
//     "name": "Alice",
//     "age": 30,
//     "friend": "user:bob"  // Reference to another node
//   },
//   "user:bob": {
//     "name": "Bob",
//     "age": 28
//   }
// }
```

### Chain API

The **Chain API** provides a fluent interface for navigating and manipulating the graph:

```rust
// Chain methods are chained together
gun.get("user").get("alice").get("name").put("Alice");
//    ^^^^     ^^^^         ^^^^          ^^^^
//   Chain   Chain        Chain         Chain
```

Each method returns a new `Chain` instance, allowing method chaining. Chains maintain context about their position in the graph hierarchy.

### Souls (Node Identifiers)

**Souls** are deterministic, unique identifiers for nodes. They are:
- **Self-describing**: Generated deterministically from node content
- **Globally unique**: No central authority needed
- **Verifiable**: Can be validated by any peer
- **Stable**: Same data generates the same soul

```rust
// Souls look like: "abc123def456..."
// They're used to reference nodes across the network
```

### State-Based CRDT

Gun.rs uses **Conflict-free Replicated Data Types (CRDTs)** with state-based conflict resolution:
- **HAM (Hypothetical Amnesia Machine) Algorithm**: Resolves conflicts using timestamps and logical clocks
- **Last-Write-Wins**: With tie-breaking based on peer IDs
- **Automatic merging**: Conflicting updates are automatically resolved
- **Eventual consistency**: All peers eventually converge to the same state

### DAM Protocol (Directed Acyclic Mesh)

The **DAM protocol** is Gun's custom P2P networking layer:
- **Message routing**: Efficient message broadcasting through the mesh
- **Deduplication**: Prevents message loops and duplicate processing
- **Peer discovery**: Automatic discovery of nearby peers
- **NAT traversal**: Works behind firewalls with relay support
- **Cryptographic security**: All messages are signed with BLS signatures and verified

### Offline-First Architecture

Gun.rs is designed for **offline-first** operation:
- **Local storage**: Data is persisted locally using pluggable storage backends
- **Sync on connect**: Automatically syncs when peers become available
- **Works offline**: Full functionality available without network connectivity
- **Conflict resolution**: Handles conflicts when sync occurs

### Storage Backends

Multiple storage backends are available:
- **MemoryStorage**: In-memory storage (default, no persistence)
- **LocalStorage**: File-based storage (localStorage-like)
- **SledStorage**: High-performance embedded database (radisk mode)

### Cryptographic Security

Gun.rs uses BLS (Boneh-Lynn-Shacham) signatures for cryptographic security:
- **Message signing**: All outgoing messages are signed with the secret key
- **Message verification**: All incoming messages are verified using public keys
- **Peer authentication**: Each peer maintains a mapping of peer IDs to public keys
- **Tamper detection**: Invalid signatures cause messages to be rejected
- **Message predicates**: Optional custom filtering after signature verification

---

## Crate Usage

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
gun = { git = "https://github.com/DIG-Network/gun.rs" }
chia_bls = "12.2"
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
```

**Note**: Gun.rs requires BLS (Boneh-Lynn-Shacham) key pairs for cryptographic security. All messages are signed and verified using BLS signatures.

### Basic Usage

#### Creating a Gun Instance

```rust
use gun::Gun;
use chia_bls::{SecretKey, PublicKey};

// Generate BLS key pair
let secret_key = SecretKey::from_seed(&[0u8; 32]);
let public_key = secret_key.public_key();

// Simple instance (no networking, in-memory storage)
let gun = Gun::new(secret_key, public_key);

// With options (networking, storage, etc.)
use gun::GunOptions;
let secret_key = SecretKey::from_seed(&[1u8; 32]);
let public_key = secret_key.public_key();
let gun = Gun::with_options(secret_key, public_key, GunOptions {
    peers: vec!["ws://relay.example.com/gun".to_string()],
    localStorage: true,
    storage_path: Some("./gun_data".to_string()),
    ..Default::default()
}).await?;
```

#### Reading and Writing Data

```rust
use gun::Gun;
use chia_bls::{SecretKey, PublicKey};
use serde_json::json;

// Generate BLS key pair
let secret_key = SecretKey::from_seed(&[0u8; 32]);
let public_key = secret_key.public_key();
let gun = Gun::new(secret_key, public_key);

// Write data
gun.get("user").get("alice").put(json!("Alice")).await?;

// Read data once
gun.get("user").get("alice").once(|data, key| {
    println!("User: {:?}", data);
}).await?;

// Subscribe to updates
gun.get("user").get("alice").on(|data, key| {
    println!("Updated: {:?}", data);
});
```

#### Working with Objects

```rust
use serde_json::json;

// Put a complete object
gun.get("user").get("alice").put(json!({
    "name": "Alice",
    "age": 30,
    "email": "alice@example.com"
})).await?;

// Update specific fields
gun.get("user").get("alice").get("age").put(json!(31)).await?;
```

#### Relationships (Soul References)

```rust
// Create two users
gun.get("user").get("alice").put(json!({
    "name": "Alice"
})).await?;

gun.get("user").get("bob").put(json!({
    "name": "Bob"
})).await?;

// Create a relationship (reference)
// Note: In practice, you'd get the soul from the created node
// This is a simplified example
let alice_soul = "..."; // Soul from Alice node
gun.get("user").get("bob").get("friend").put(json!(alice_soul)).await?;
```

#### Arrays/Lists with Map

```rust
// Map over a collection
gun.get("users").map(|data, key| {
    if let Some(name) = data.get("name").and_then(|v| v.as_str()) {
        println!("User: {}", name);
    }
});

// Add items to a collection
for i in 1..=10 {
    gun.get("users").get(&format!("user_{}", i)).put(json!({
        "id": i,
        "name": format!("User {}", i)
    })).await?;
}
```

#### Removing Listeners

```rust
// Remove all listeners from a chain
gun.get("user").get("alice").off();

// Note: off() is chain-aware and removes listeners from the current chain point
```

#### Navigation with Back

```rust
// Navigate back up the chain
let chain = gun.get("user").get("alice").get("name");
let parent = chain.back(Some(1)); // Go back 1 level -> "alice" chain
let root = chain.back(None);      // Go back to root -> gun root
```

### Network Configuration

#### Connecting to Relay Servers

```rust
use gun::{Gun, GunOptions};
use chia_bls::{SecretKey, PublicKey};

// Generate BLS key pair
let secret_key = SecretKey::from_seed(&[0u8; 32]);
let public_key = secret_key.public_key();

// Single relay
let gun = Gun::with_options(
    secret_key.clone(),
    public_key.clone(),
    GunOptions::with_relay("ws://relay.example.com/gun")
).await?;

// Multiple relays for redundancy
let secret_key2 = SecretKey::from_seed(&[1u8; 32]);
let public_key2 = secret_key2.public_key();
let gun = Gun::with_options(
    secret_key2,
    public_key2,
    GunOptions::with_peers(vec![
        "ws://relay1.example.com/gun".to_string(),
        "ws://relay2.example.com/gun".to_string(),
    ])
).await?;
```

#### Running a Relay Server

```rust
use gun::{Gun, GunOptions};
use chia_bls::{SecretKey, PublicKey};

// Generate BLS key pair
let secret_key = SecretKey::from_seed(&[0u8; 32]);
let public_key = secret_key.public_key();

// Start a relay server on port 8765
let gun = Gun::with_options(
    secret_key,
    public_key,
    GunOptions::relay_server(8765)
).await?;

// Server will accept connections from other peers
```

#### WebRTC Configuration

```rust
use gun::{Gun, GunOptions};
use gun::webrtc::WebRTCOptions;
use chia_bls::{SecretKey, PublicKey};

// Generate BLS key pair
let secret_key = SecretKey::from_seed(&[0u8; 32]);
let public_key = secret_key.public_key();

let mut webrtc_opts = WebRTCOptions::default();
webrtc_opts.enabled = true;
webrtc_opts.max_connections = 10;

let mut opts = GunOptions::default();
opts.peers = vec!["ws://relay.example.com/gun".to_string()];
opts.webrtc = webrtc_opts;

let gun = Gun::with_options(secret_key, public_key, opts).await?;
```

### Storage Configuration

#### Using Local Storage

```rust
use gun::{Gun, GunOptions};
use chia_bls::{SecretKey, PublicKey};

// Generate BLS key pair
let secret_key = SecretKey::from_seed(&[0u8; 32]);
let public_key = secret_key.public_key();

let opts = GunOptions {
    localStorage: true,
    storage_path: Some("./gun_data".to_string()),
    ..Default::default()
};

let gun = Gun::with_options(secret_key, public_key, opts).await?;
// Data will be persisted to ./gun_data/
```

#### Using Sled Storage (Radisk Mode)

```rust
use gun::{Gun, GunOptions};
use chia_bls::{SecretKey, PublicKey};

// Generate BLS key pair
let secret_key = SecretKey::from_seed(&[0u8; 32]);
let public_key = secret_key.public_key();

let opts = GunOptions {
    localStorage: true,
    radisk: true,
    storage_path: Some("./gun_data".to_string()),
    ..Default::default()
};

let gun = Gun::with_options(secret_key, public_key, opts).await?;
// Uses high-performance sled database
```

### Connection Management

```rust
// Check connection status
let is_connected = gun.is_connected().await;
let peer_count = gun.connected_peer_count().await;

// Wait for connection with timeout
let connected = gun.wait_for_connection(5000).await; // 5 second timeout

// Graceful shutdown
gun.shutdown().await?;
```

### Error Handling

```rust
use gun::GunError;

match gun.get("key").put(data).await {
    Ok(chain) => {
        // Success
    }
    Err(GunError::InvalidData(msg)) => {
        eprintln!("Invalid data: {}", msg);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
    }
}
```

---

## Exhaustive Reference Guide

### Core Types

#### `Gun`

The main entry point for the Gun.rs library.

**Methods:**

- `new(secret_key: SecretKey, public_key: PublicKey) -> Gun`
  - Creates a new Gun instance with default settings (no networking, in-memory storage)
  - Requires BLS key pair for cryptographic security
  - All messages are signed with the secret key and verified with public keys

- `with_options(secret_key: SecretKey, public_key: PublicKey, options: GunOptions) -> GunResult<Gun>`
  - Creates a Gun instance with custom options
  - Requires BLS key pair for cryptographic security
  - Async function - must be awaited

- `get(key: &str) -> Arc<Chain>`
  - Returns a chain pointing to the specified key
  - Entry point for navigating the graph

- `root() -> Arc<Chain>`
  - Returns a chain pointing to the root of the graph

- `state() -> f64`
  - Returns the current state timestamp (used for conflict resolution)

- `connected_peer_count() -> usize`
  - Returns the number of currently connected peers
  - Async function

- `is_connected() -> bool`
  - Returns true if connected to at least one peer
  - Async function

- `wait_for_connection(timeout_ms: u64) -> bool`
  - Waits for a connection to be established
  - Returns true if connected within timeout, false otherwise
  - Async function

- `shutdown() -> GunResult<()>`
  - Gracefully shuts down the Gun instance
  - Stops servers and closes connections
  - Async function

#### `Chain`

The fluent API for interacting with the graph. All chain methods return `Arc<Chain>` for method chaining.

**Methods:**

- `get(key: &str) -> Arc<Chain>`
  - Navigate to a property or child node
  - Returns a new chain with the key appended

- `put(data: Value) -> GunResult<Arc<Chain>>`
  - Write data to the current chain position
  - Accepts `serde_json::Value` (numbers, strings, booleans, objects, arrays)
  - Objects are automatically expanded into the graph
  - Returns error if data is invalid
  - Async function

- `on<F>(callback: F) -> Arc<Chain>`
  - Subscribe to updates at this chain position
  - Callback signature: `Fn(Value, Option<String>)`
  - Returns immediately (non-blocking)
  - Returns the chain for further chaining
  - Listeners persist until explicitly removed

- `once<F>(callback: F) -> GunResult<Arc<Chain>>`
  - Execute callback once when data is available
  - Callback signature: `Fn(Value, Option<String>)`
  - Returns error if data is already available but callback fails
  - Async function (waits for data)

- `map<F>(callback: F) -> Arc<Chain>`
  - Iterate over child nodes/keys
  - Callback signature: `Fn(Value, Option<String>)`
  - Used for working with collections/arrays
  - Returns immediately (non-blocking)

- `set(item: Value) -> GunResult<Arc<Chain>>`
  - Add an item to a collection (similar to array.push)
  - Generates a unique key for the item
  - Returns the chain with the generated key appended
  - Async function

- `back(amount: Option<usize>) -> Option<Arc<Chain>>`
  - Navigate back up the chain
  - `amount: Some(n)` - go back n levels
  - `amount: None` - go back to root
  - Returns `None` if cannot go back that far

- `off() -> Arc<Chain>`
  - Remove all listeners from this chain position
  - Does not remove listeners from parent/child chains
  - Returns the chain for further operations

#### `GunOptions`

Configuration options for creating a Gun instance.

**Fields:**

- `peers: Vec<String>`
  - List of WebSocket URLs to connect to (relay servers)
  - Empty by default

- `localStorage: bool`
  - Enable local storage persistence
  - Default: `false`

- `storage_path: Option<String>`
  - Path for local storage
  - Default: `None` (uses "./gun_data" if localStorage is true)

- `radisk: bool`
  - Use SledStorage instead of LocalStorage (high-performance mode)
  - Default: `false`

- `super_peer: bool`
  - Enable relay server mode
  - Default: `false`

- `port: Option<u16>`
  - Port to listen on (for relay server mode)
  - Default: `None`

- `webrtc: WebRTCOptions`
  - WebRTC configuration (see `WebRTCOptions` below)
  - Default: `WebRTCOptions::default()`

- `message_predicate: Option<MessagePredicate>`
  - Optional predicate function to filter incoming messages
  - Receives the entire message object and returns `true` to accept, `false` to reject
  - Called after signature verification but before message processing
  - Useful for implementing custom filtering, rate limiting, or access control
  - Default: `None` (all verified messages are accepted)

**Methods:**

- `default() -> GunOptions`
  - Creates default options (no networking, no storage)

- `with_relay(relay_url: &str) -> GunOptions`
  - Convenience method for single relay connection

- `with_peers(peers: Vec<String>) -> GunOptions`
  - Convenience method for multiple peer connections

- `relay_server(port: u16) -> GunOptions`
  - Convenience method for relay server configuration

#### `WebRTCOptions`

Configuration for WebRTC peer-to-peer connections.

**Fields:**

- `ice_servers: Vec<RTCIceServer>`
  - STUN/TURN servers for NAT traversal
  - Default: Google and Cloudflare STUN servers

- `data_channel: RTCDataChannelInit`
  - Data channel configuration
  - Default: unordered, max retransmits 2

- `max_connections: usize`
  - Maximum number of WebRTC connections
  - Default: `55`

- `room: Option<String>`
  - Room name for peer discovery (optional)
  - Default: `None`

- `enabled: bool`
  - Enable/disable WebRTC
  - Default: `true`

**Methods:**

- `default() -> WebRTCOptions`
  - Creates default WebRTC options

#### `GunError`

Error type for Gun.rs operations.

**Variants:**

- `GunError::InvalidData(String)`
  - Invalid data provided (e.g., invalid type, invalid structure)

- `GunError::Storage(sled::Error)`
  - Storage operation failed (from sled database)

- `GunError::Serialization(serde_json::Error)`
  - JSON serialization/deserialization failed

- `GunError::Network(String)`
  - Network operation failed (connection lost, timeout, etc.)

- `GunError::InvalidSoul(String)`
  - Invalid soul (node ID) format

- `GunError::NodeNotFound`
  - Requested node doesn't exist in the graph

- `GunError::Io(std::io::Error)`
  - I/O operation failed (file read/write, etc.)

- `GunError::UrlParseError(url::ParseError)`
  - URL parsing failed (invalid peer URL)

- `GunError::WebRTC(String)`
  - WebRTC operation failed (connection, signaling, etc.)

- `GunError::Crypto(String)`
  - Cryptographic operation failed (encryption, signing, etc.)

### Module Reference

#### `gun::chain`
- `Chain` - Main chain API type

#### `gun::core`
- `GunCore` - Core graph database engine (internal)

#### `gun::dam`
- `Mesh` - DAM protocol mesh networking (internal)

#### `gun::error`
- `GunError` - Error types
- `GunResult<T>` - Result type alias: `Result<T, GunError>`

#### `gun::graph`
- Graph data structures (internal)

#### `gun::storage`
- `Storage` - Storage trait
- `MemoryStorage` - In-memory storage
- `LocalStorage` - File-based storage
- `SledStorage` - Sled database storage

#### `gun::webrtc`
- `WebRTCOptions` - WebRTC configuration
- `WebRTCManager` - WebRTC manager (internal)
- `WebRTCPeer` - WebRTC peer connection (internal)

#### `gun::websocket`
- WebSocket client and server (internal)

#### `gun::sea`
- Security, Encryption, Authorization module (partial implementation)

#### `gun::types`
- `MessagePredicate` - Message filtering predicate type for custom message filtering

### Type Aliases

- `GunResult<T>` = `Result<T, GunError>`
- `MessagePredicate` = `Arc<dyn Fn(&serde_json::Value) -> bool + Send + Sync>`
  - Function type for custom message filtering

### Constants

None currently exported.

---

## Examples

### Basic Example

```rust
use gun::Gun;
use chia_bls::{SecretKey, PublicKey};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    
    let gun = Gun::new(secret_key, public_key);
    
    // Write data
    gun.get("user").get("alice").put(json!("Alice")).await?;
    
    // Read data
    gun.get("user").get("alice").once(|data, _key| {
        println!("User: {:?}", data);
    }).await?;
    
    Ok(())
}
```

### Real-time Sync Example

```rust
use gun::{Gun, GunOptions};
use chia_bls::{SecretKey, PublicKey};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    
    let gun = Gun::with_options(
        secret_key,
        public_key,
        GunOptions::with_relay("ws://relay.example.com/gun")
    ).await?;
    
    // Subscribe to updates
    gun.get("chat").get("messages").on(|data, key| {
        if let Some(text) = data.get("text").and_then(|v| v.as_str()) {
            println!("New message: {}", text);
        }
    });
    
    // Send a message
    gun.get("chat").get("messages").set(json!({
        "text": "Hello, world!",
        "author": "Alice"
    })).await?;
    
    // Keep running
    tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    
    Ok(())
}
```

### WebRTC Example

See `examples/two_clients_webrtc.rs` for a complete WebRTC example demonstrating:
- Two clients with WebRTC enabled
- Direct peer-to-peer communication
- NAT traversal

### Relay Server Example

See `examples/two_clients.rs` for a complete example demonstrating:
- Two clients connecting via relay
- Data synchronization
- Real-time updates

Run examples:
```bash
cargo run --example two_clients
cargo run --example two_clients_webrtc
```

---

## Additional Resources

### Protocol Documentation

- **DAM Protocol**: See [NETWORKING.md](NETWORKING.md)
- **Relay Servers**: See [RELAY_SERVERS.md](RELAY_SERVERS.md)
- **WebRTC Status**: See [WEBRTC_STATUS.md](WEBRTC_STATUS.md)

### Testing

See `tests/` directory for comprehensive test suites:
- Unit tests
- Integration tests
- WebRTC tests
- Lock contention tests
- Stress tests

### Architecture Notes

**Important**: Gun.js uses a custom **DAM (Directed Acyclic Mesh) protocol** over WebSocket, NOT libp2p. For 1:1 behavioral compatibility, we use:
- `tokio-tungstenite` for WebSocket transport (matches Gun.js)
- Custom DAM protocol implementation (matches mesh.js)
- Message deduplication (matches dup.js)

### License

MIT OR Apache-2.0 OR Zlib (same as Gun.js)
