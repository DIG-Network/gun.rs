# Gun.rs

A Rust port of [Gun.js](https://github.com/amark/gun) - a realtime, decentralized, offline-first, graph data synchronization engine.

## Features

- **Graph Database**: Store data as a graph with nodes and relationships
- **Real-time Sync**: Synchronize data in real-time across peers
- **P2P Networking**: Peer-to-peer mesh networking for decentralized communication
- **Offline-First**: Works offline and syncs when online
- **State-based CRDT**: Conflict-free replicated data types with state-based resolution
- **WebSocket Support**: WebSocket server and client support
- **Local Storage**: Persistent local storage with sled

## Quick Start

```rust
use gun::Gun;
use serde_json::json;

let gun = Gun::new();

// Save data (matches JavaScript: gun.get('user').get('name').put('Alice'))
gun.get("user").get("name").put(json!("Alice")).await?;

// Read data (matches JavaScript: gun.get('user').get('name').once((data, key) => {...}))
gun.get("user").get("name").once(|data, key| {
    println!("Name: {:?}", data);
}).await?;

// Subscribe to updates (matches JavaScript: gun.get('user').on((data, key) => {...}))
gun.get("user").on(|data, key| {
    println!("Updated: {:?}", data);
});
```

See the `examples/` directory for more examples matching the JavaScript version.

## Relay Servers (Optional)

Gun.js is **fully decentralized** - no central server required! Relay servers are optional peers that help with connectivity:

```rust
use gun::{Gun, GunOptions};

// Connect to your relay server (optional, just helps with NAT traversal)
let gun = Gun::with_options(GunOptions::with_relay(
    "ws://dig-relay-prod.eba-2cmanxbe.us-east-1.elasticbeanstalk.com/gun"
));

// Or connect to multiple relays for redundancy
let gun = Gun::with_options(GunOptions::with_peers(vec![
    "ws://your-relay.com/gun".to_string(),
    "ws://backup-relay.com/gun".to_string(),
]));

// Or run fully P2P without any relay
let gun = Gun::new(); // No relays needed!
```

See [RELAY_SERVERS.md](RELAY_SERVERS.md) for more details.

## Status

This is a work-in-progress port of Gun.js to Rust. The core API is being implemented to match the JavaScript version as closely as possible.

### Networking

**Important**: Gun.js uses a custom **DAM (Directed Acyclic Mesh) protocol** over WebSocket, NOT libp2p. For 1:1 behavioral compatibility, we use:
- `tokio-tungstenite` for WebSocket transport (matches Gun.js)
- Custom DAM protocol implementation (matches mesh.js)
- Message deduplication (matches dup.js)

See [NETWORKING.md](NETWORKING.md) for details.

## License

MIT OR Apache-2.0 OR Zlib (same as Gun.js)

