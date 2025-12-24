# Quality Assurance Guide

This document outlines the measures in place to ensure gun.rs works as expected.

## Automated Testing

### Unit Tests
Run with: `cargo test --lib`

Tests individual components in isolation.

### Integration Tests
Run with: `cargo test --test integration_tests -- --test-threads=1`

Tests full functionality including network operations with real relay servers.

### Lock Contention Tests
Run with: `cargo test --test lock_tests`

Specifically tests that lock refactoring prevents deadlocks under concurrent access.

### Stress Tests
Run with: `cargo test --test stress_tests -- --ignored`

Tests system behavior under high load and concurrent operations.

## Code Quality

### Formatting
- All code must pass `cargo fmt --all -- --check`
- Configuration: `rustfmt.toml`

### Linting
- All code must pass `cargo clippy --all-targets --all-features -- -D warnings`
- Configuration: `.clippy.toml`
- Addresses common bugs and style issues

### Compiler Warnings
- All warnings are treated as errors in CI
- Code must compile without warnings

## CI/CD Pipeline

The `.github/workflows/CI.yml` pipeline runs on every push/PR and checks:

1. **Format Check**: Ensures code is properly formatted
2. **Clippy**: Lints code for common issues
3. **Build**: Verifies code compiles on multiple platforms
4. **Unit Tests**: Runs all unit tests
5. **Integration Tests**: Runs integration tests (may fail in CI due to network)
6. **Stress Tests**: Runs stress tests (optional)
7. **Security Audit**: Checks for known vulnerabilities

## Lock Safety

The refactored lock usage ensures:

1. **No Deadlocks**: Locks are never held during async operations
2. **Minimal Scope**: Locks are acquired, data cloned, and released immediately
3. **No Lock Upgrades**: Never acquire read lock then write lock
4. **Timeout Protection**: Lock acquisition has timeouts to prevent indefinite blocking

### Lock Patterns Used

```rust
// Good: Clone data then release lock
let data = {
    let lock = self.data.read().await;
    lock.clone()
}; // Lock released here
// Use data without lock

// Bad: Holding lock during async operation
let lock = self.data.read().await;
some_async_operation().await; // Lock still held!
```

## Connection Verification

- All integration tests verify connections are established before proceeding
- `connected_peer_count()` works without deadlocking
- Connection timeouts prevent tests from hanging indefinitely

## Error Handling

- All network operations have proper error handling
- Timeouts prevent indefinite waits
- Graceful degradation when connections fail

## Performance Considerations

- Locks held for minimal time
- Data cloned instead of holding locks
- Concurrent operations don't block each other unnecessarily

## Running All Checks Locally

```bash
make ci  # Runs format check, clippy, build, and tests
```

Or manually:
```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-features
cargo test --all-features
```

## Manual Testing

### Connection Test
```bash
cargo test --test integration_tests test_relay_connection -- --nocapture
```

### Lock Safety Test
```bash
cargo test --test lock_tests -- --nocapture
```

### Stress Test
```bash
cargo test --test stress_tests -- --ignored --nocapture
```

## Monitoring

- Connection status is tracked and verifiable
- Lock contention is minimized
- Errors are properly logged

## Best Practices

1. Always test locally before pushing
2. Run `make ci` to ensure all checks pass
3. Write tests for new functionality
4. Keep lock scopes minimal
5. Never hold locks during async operations
6. Use timeouts for network operations

