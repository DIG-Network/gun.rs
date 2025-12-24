# Networking Architecture - 1:1 with Gun.js

## Why NOT libp2p?

Gun.js uses a **custom DAM (Directed Acyclic Mesh) protocol** over WebSocket, NOT libp2p. For 1:1 behavioral compatibility, we must match this exactly.

### Gun.js Networking Stack

1. **WebSocket** - Transport layer (not libp2p transports)
2. **DAM Protocol** - Custom mesh routing with message deduplication
3. **dup.js** - Message ID deduplication (prevents message loops)
4. **mesh.js** - Peer management and message routing

### Current Rust Implementation

- ✅ `tokio-tungstenite` - WebSocket implementation (matches Gun.js WebSocket)
- ✅ `dup.rs` - Message deduplication (matches dup.js)
- ✅ `dam.rs` - DAM protocol implementation (matches mesh.js)
- ❌ NOT using libp2p (would break compatibility)

## DAM Protocol Details

### Message Format

```json
{
  "#": "message_id",
  "##": "hash_optional",
  "@": "ack_id",
  "put": {...},  // or "get": {...}
  "dam": "?",    // special DAM control messages
  "><": "peer1,peer2"  // peers already sent to (dedup)
}
```

### Key Features

1. **Message Deduplication** - Uses `#` (message ID) to prevent loops
2. **Content Hashing** - Uses `##` (hash) for content-based dedup
3. **ACK Routing** - Uses `@` (ack ID) to route responses
4. **Peer Tracking** - Uses `><` to track which peers already received message
5. **Batching** - Batches multiple messages in JSON array `[...]`

### Peer Discovery

Gun.js uses:
- Static peer list from `opt.peers`
- WebSocket handshake with peer ID exchange (`dam: '?'`)
- No DHT or libp2p discovery

## Implementation Status

### ✅ Completed
- Message deduplication (dup.rs)
- DAM message structure
- Basic mesh framework

### 🔄 In Progress
- WebSocket server/client integration
- Full DAM message routing
- Peer management (hi/bye)
- Message batching

### ❌ Not Started
- Full message routing with `><` tracking
- Content hashing (`##`)
- ACK routing optimization
- Reconnection logic

## Why WebSocket and NOT libp2p?

1. **1:1 Compatibility** - Gun.js clients expect WebSocket
2. **Protocol Matching** - DAM protocol is WebSocket-based
3. **Simplicity** - WebSocket is simpler than libp2p's abstraction layers
4. **Browser Support** - WebSocket is native in browsers

## Future Considerations

If we wanted to add libp2p support:
- Could create a separate transport layer
- Would need protocol translation layer
- Would break compatibility with standard Gun.js
- Could be an optional feature behind a feature flag

But for now, **WebSocket + DAM is the correct choice** for 1:1 behavior.

