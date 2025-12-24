# Gun.js to Gun.rs Porting Notes

This document tracks the progress of porting Gun.js to Rust and notes on design decisions.

## Completed

### Core Structure
- ✅ Project setup (Cargo.toml, basic structure)
- ✅ Core data structures (Graph, Node, State)
- ✅ State management (timestamp-based conflict resolution)
- ✅ Chain API skeleton (get, put, on, once, map, set, back)
- ✅ Storage trait and implementations (MemoryStorage, SledStorage)
- ✅ Event system foundation
- ✅ Error handling

### Modules

#### `src/core.rs`
- GunCore structure with graph, state, events
- UUID generation (matching Gun.js format)
- Chain ID generation

#### `src/state.rs`
- State timestamp generation (matches Gun.js algorithm)
- Node structure with data and meta
- State.ify() for merging state into nodes
- State.is() for reading state

#### `src/graph.rs`
- Graph storage with HashMap
- Node merging with conflict resolution (HAM-based)
- Put/get operations

#### `src/chain.rs`
- Chain API matching Gun.js interface
- get(), put(), on(), once(), map(), set(), back(), off()
- Fluent API design

#### `src/storage.rs`
- Storage trait for pluggable backends
- MemoryStorage implementation
- SledStorage implementation (persistent)

#### `src/events.rs`
- EventEmitter for hooks and events
- Event listener registration

#### `src/network.rs`
- Basic mesh network structure
- Peer management
- Network message format

## In Progress / Needs Work

### Chain API
- [ ] Complete put() implementation with proper graph traversal
- [ ] Complete on() implementation with real-time updates
- [ ] Complete once() with proper async handling
- [ ] Complete map() for iterating over node properties
- [ ] Proper set() implementation (unordered list/set)

### Network Layer
- [ ] Complete WebSocket server implementation
- [ ] Complete WebSocket client implementation
- [ ] Message routing and synchronization
- [ ] Peer discovery and mesh networking
- [ ] Message deduplication

### Graph Operations
- [ ] Proper HAM (Hypothetical Amnesia Machine) conflict resolution
- [ ] Graph traversal for circular references
- [ ] Link resolution (soul references)
- [ ] State-based merge logic

### Storage
- [ ] LocalStorage adapter (browser equivalent)
- [ ] RAD (Radix) storage format
- [ ] Persistence hooks

### SEA (Security, Encryption, Authorization)
- [ ] User authentication
- [ ] Encryption/decryption
- [ ] Signature verification
- [ ] Key pair generation

### API Completeness
- [ ] Full compatibility with Gun.js API
- [ ] TypeScript-style type safety
- [ ] Better error messages
- [ ] Performance optimization

## Design Decisions

### Async/Await vs Callbacks
- Using Rust's async/await for async operations
- Callbacks for event listeners (on, once)
- This matches Rust idioms while maintaining API similarity

### Type System
- Using serde_json::Value for data (flexible like JavaScript)
- Could add generics later for type safety
- Chain API uses Arc for sharing

### State Management
- State timestamps use same algorithm as Gun.js
- Floating point timestamps with fractional increments
- Conflict resolution based on state comparison

### Storage
- Trait-based design allows pluggable storage
- Default in-memory storage
- Sled for persistent storage (Rust equivalent of LevelDB)

## Testing

- [ ] Unit tests for core modules
- [ ] Integration tests matching Gun.js behavior
- [ ] Performance benchmarks
- [ ] Compatibility tests with Gun.js

## Notes

- Gun.js uses a lot of dynamic typing and prototype chains
- Rust's static typing requires some trade-offs
- Event system needs careful design for real-time updates
- Network layer needs proper async message handling
- State resolution algorithm is critical for correctness

## Next Steps

1. Complete chain.put() with full graph traversal
2. Implement real-time synchronization
3. Add WebSocket networking
4. Implement SEA module
5. Add comprehensive tests
6. Performance optimization

