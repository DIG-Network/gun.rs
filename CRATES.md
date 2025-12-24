# Crate Dependencies - 1:1 Compatibility Analysis

This document explains why each crate was chosen to maintain 1:1 behavioral compatibility with Gun.js.

## Core Crates

### Async Runtime
- **`tokio`** - Async runtime for Rust
  - Matches: Node.js async I/O model
  - Needed for: WebSocket, storage I/O, async chain operations

### Serialization
- **`serde` + `serde_json`** - JSON serialization
  - Matches: JavaScript's native JSON (Gun.js uses JSON.stringify/parse)
  - Critical for: Message format, data storage, WebSocket messages
  - 1:1: ✅ Uses same JSON format as Gun.js

## Network Layer

### WebSocket
- **`tokio-tungstenite`** - WebSocket implementation
  - Matches: Gun.js uses native WebSocket API
  - Needed for: DAM protocol transport
  - Why NOT libp2p: Gun.js uses custom DAM protocol over WebSocket, not libp2p
  - 1:1: ✅ WebSocket is the correct transport for Gun.js compatibility

### URL Parsing
- **`url`** - URL parsing and manipulation
  - Matches: JavaScript URL parsing
  - Needed for: Peer URLs, WebSocket connections

## Crypto (for SEA module)

- **`sha2`** - SHA-256 hashing
  - Matches: Gun.js SEA uses SHA-256
  - 1:1: ✅ Same algorithm

- **`sha1`** - SHA-1 hashing  
  - Matches: Gun.js uses SHA-1 in some contexts
  - 1:1: ✅ Same algorithm

- **`aes-gcm`** - AES-GCM encryption
  - Matches: Gun.js SEA uses AES-GCM
  - 1:1: ✅ Same algorithm

- **`rand`** - Random number generation
  - Matches: JavaScript Math.random() / crypto.getRandomValues()
  - 1:1: ✅ Cryptographically secure random (matches WebCrypto)

- **`hex`** - Hex encoding/decoding
  - Matches: JavaScript hex string handling
  - 1:1: ✅ Same format

- **`base64`** - Base64 encoding/decoding
  - Matches: JavaScript btoa()/atob()
  - 1:1: ✅ Same format

## Storage

- **`sled`** - Embedded database
  - Matches: Gun.js uses various storage adapters (localStorage, Radisk, etc.)
  - Alternative to: LevelDB, IndexedDB, localStorage
  - Note: Pure Rust, fast, transactional
  - 1:1: ✅ Provides persistent storage (actual storage format may differ but API matches)

- **`parking_lot`** - Fast synchronization primitives
  - Needed for: Graph locking, state management
  - 1:1: ✅ Internal implementation detail (doesn't affect API)

## Utilities

- **`uuid`** - UUID generation
  - Used for: Peer IDs, temporary IDs
  - Matches: Gun.js generates random IDs
  - 1:1: ⚠️ Gun.js uses custom random string generation, but UUIDs are compatible

- **`chrono`** - Date/time handling
  - Matches: JavaScript Date objects
  - Needed for: Timestamps, state timestamps
  - 1:1: ✅ Unix timestamps are compatible

- **`thiserror`** - Error handling
  - Internal: Error types for better error messages
  - 1:1: ✅ Doesn't affect API, internal only

- **`tracing`** - Structured logging
  - Matches: console.log, but better structured
  - 1:1: ✅ Optional, doesn't affect behavior

## What We're NOT Using (and why)

### libp2p
- **Why NOT**: Gun.js uses custom DAM protocol over WebSocket, not libp2p
- **Impact**: Using libp2p would break compatibility with Gun.js clients
- **Decision**: ✅ Correct to NOT use libp2p for 1:1 compatibility

### Other P2P Libraries
- **Why NOT**: Gun.js has its own mesh protocol (DAM)
- **Decision**: ✅ Must implement DAM protocol directly for compatibility

## Compatibility Matrix

| Component | Gun.js | Gun.rs | 1:1 Match |
|-----------|--------|--------|-----------|
| JSON Format | JSON.stringify/parse | serde_json | ✅ Yes |
| WebSocket | Native WebSocket | tokio-tungstenite | ✅ Yes |
| State Algorithm | Custom timestamp | Same algorithm | ✅ Yes |
| Message Dedup | dup.js | dup.rs | ✅ Yes |
| Mesh Protocol | DAM (mesh.js) | dam.rs | 🔄 In progress |
| Encryption | SEA (WebCrypto) | aes-gcm, sha2 | ✅ Yes |
| Storage | Various adapters | sled (default) | ⚠️ Compatible API |

## Summary

All crate choices prioritize **1:1 behavioral compatibility** with Gun.js:

1. ✅ **WebSocket over libp2p** - Matches Gun.js protocol
2. ✅ **JSON serialization** - Same format
3. ✅ **Same crypto algorithms** - SHA-256, AES-GCM, etc.
4. ✅ **Same state algorithm** - Timestamp-based conflict resolution
5. ✅ **Same deduplication** - Message ID tracking

The main differences are:
- Internal implementation (Rust vs JavaScript)
- Storage backend (sled vs various JS adapters) - but API is compatible
- UUID format (standard UUID vs custom random strings) - but compatible

These choices ensure that Gun.rs can:
- ✅ Communicate with Gun.js peers
- ✅ Use the same message format
- ✅ Have the same conflict resolution behavior
- ✅ Support the same encryption (when SEA is implemented)

