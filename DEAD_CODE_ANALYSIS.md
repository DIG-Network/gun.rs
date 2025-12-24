# Dead Code Analysis

This document analyzes the dead code warnings in the codebase to identify missing functionality vs. truly unused code.

## Summary

Most "dead code" falls into these categories:
1. **Unused duplicate implementations** - Code that was superseded but never removed
2. **Placeholder/empty stubs** - Methods with empty implementations that need completion
3. **Internal accessors** - Methods that might be useful but aren't currently needed
4. **Server lifecycle** - Code stored for potential cleanup but not actively used

## Dead Code Breakdown

### 1. `src/network.rs` - Entire Module is Unused ❌

**Status: Should be removed or completed**

The `network.rs` module contains duplicate/unused implementations:

- **`MeshNetwork`** - Has empty stub implementations for `send()` and `broadcast()`
- **`WebSocketServer`** in `network.rs` - Different from the real one in `websocket.rs`
- **`Peer`** - Different structure from `dam::Peer`

**Why it's dead:**
- The actual networking is handled by `dam::Mesh` and `websocket::WebSocketClient/Server`
- `MeshNetwork` is never instantiated or used anywhere
- The `network.rs` module appears to be an early implementation attempt that was superseded

**Recommendation:** 
- **Option A**: Remove `src/network.rs` entirely (cleanest)
- **Option B**: Complete the implementations if there's a reason to have an alternative API

### 2. `dam.rs::send_to_peer()` - Convenience Wrapper

**Status: Might be useful, but currently unused**

```rust
async fn send_to_peer(&self, raw: &str, peer: &Peer) -> GunResult<()> {
    self.send_to_peer_by_id(raw, &peer.id).await
}
```

**Why it's dead:**
- Currently all code uses `send_to_peer_by_id()` directly
- This is just a convenience wrapper

**Recommendation:** 
- Keep it for API completeness (might be useful for external code)
- Or remove if we want to keep the API minimal

### 3. `Gun::core()` and `Gun::mesh()` - Internal Accessors

**Status: Internal API methods**

```rust
pub(crate) fn core(&self) -> &Arc<GunCore> { ... }
pub(crate) fn mesh(&self) -> Option<&Arc<Mesh>> { ... }
```

**Why they're "dead":**
- Marked as `pub(crate)` for internal module access
- Currently no other modules need direct access to core/mesh
- Direct field access could be used instead

**Recommendation:**
- Keep them if planning to add internal modules that need access
- Remove if direct field access is acceptable

### 4. `Gun::ws_server` - Server Handle

**Status: Used for spawn, but not for shutdown**

The `ws_server` handle is stored when the server is started:

```rust
let handle = tokio::spawn(async move { ... });
ws_server = Some(handle);
```

**Why it's "dead":**
- The handle is stored but never read back
- Could be used for graceful shutdown

**Recommendation:**
- Add a `shutdown()` method to `Gun` that aborts the server handle
- This would enable clean server shutdown

### 5. `WebSocketServer::port` - Stored but Unused

**Status: Stored for potential future use**

The port is stored in the struct but never read after initialization.

**Recommendation:**
- Keep it for debugging/logging purposes
- Or remove if truly not needed

## Missing Functionality Identified

### 1. MeshNetwork Implementation (if kept)

If we decide to keep `MeshNetwork` as an alternative API:

- ✅ `add_peer()` - Implemented
- ❌ `send()` - Empty stub: `Ok(())`
- ❌ `broadcast()` - Empty stub: `Ok(())`
- ❌ `tx` channel - Stored but never used

**Required work:**
```rust
pub async fn send(&self, peer_id: &str, message: NetworkMessage) -> GunResult<()> {
    // Need to:
    // 1. Look up peer by ID
    // 2. Send message over WebSocket connection
    // 3. Handle errors appropriately
}

pub async fn broadcast(&self, message: NetworkMessage) -> GunResult<()> {
    // Need to:
    // 1. Iterate over all peers
    // 2. Send message to each peer
    // 3. Handle errors (continue on individual failures)
}
```

### 2. Graceful Server Shutdown

The `ws_server` handle should be used for cleanup:

```rust
impl Gun {
    pub async fn shutdown(&mut self) -> GunResult<()> {
        if let Some(handle) = self.ws_server.take() {
            handle.abort();
        }
        // Optionally: close all peer connections
        Ok(())
    }
}
```

## Recommendations

### High Priority (Should Fix)

1. **Remove `src/network.rs`** - Entire module is unused duplicate code
   - Cleaner codebase
   - Reduces confusion about which implementation to use
   - Eliminates false sense of available functionality

2. **Add graceful shutdown** - Use `ws_server` handle
   - Better resource management
   - Cleaner test cleanup
   - Production-ready behavior

### Medium Priority (Nice to Have)

3. **Keep `send_to_peer` wrapper** - Useful for API completeness
   - OR remove if we want minimal API surface

4. **Remove `core()` and `mesh()` accessors** - Use direct field access
   - OR keep if planning internal modules that need them

### Low Priority (Optional)

5. **Keep `port` field** - Might be useful for debugging
   - OR remove if truly not needed

## Action Items

- [ ] Decide: Remove or complete `MeshNetwork`?
- [ ] Implement graceful shutdown using `ws_server` handle
- [ ] Remove `src/network.rs` if not needed
- [ ] Add tests for graceful shutdown
- [ ] Update documentation to reflect final API decisions

