# Test Suite

This directory contains the test suite for Gun.rs.

## Test Files

- `basic_tests.rs` - Unit tests for core functionality (no network required)
- `integration_tests.rs` - Integration tests that connect to a real Gun.js relay server
- `lock_tests.rs` - Tests for lock contention and deadlock prevention
- `stress_tests.rs` - Stress tests for concurrent operations and network resilience
- `webrtc_tests.rs` - Comprehensive tests for WebRTC functionality (25+ tests)

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Only Unit Tests (Fast, No Network Required)
```bash
cargo test --lib
```

### Run Integration Tests (Requires Network Connectivity)
```bash
cargo test --test integration_tests
```

### Run WebRTC Tests
```bash
# Unit tests for WebRTC components
cargo test --test webrtc_tests

# Integration test: Two clients via WebRTC
cargo test --test webrtc_two_clients -- --nocapture
```

### Run Lock Contention Tests
```bash
cargo test --test lock_tests
```

### Run Stress Tests
```bash
cargo test --test stress_tests
```

### Run Tests with Output
```bash
cargo test --test integration_tests -- --nocapture
cargo test --test webrtc_tests -- --nocapture
```

### Run Specific Test
```bash
cargo test --test integration_tests test_relay_put_get
cargo test --test webrtc_tests test_webrtc_peer_creation
```

## Integration Tests

The integration tests (`integration_tests.rs`) connect to a real Gun.js relay server to verify:
- Basic put/get operations
- Real-time updates
- Data persistence
- Concurrent connections
- Chain operations
- Map operations
- Error handling

**Note**: Integration tests require:
1. Network connectivity
2. Access to the relay server: `http://dig-relay-prod.eba-2cmanxbe.us-east-1.elasticbeanstalk.com/gun`
3. May take longer to run due to network delays

The tests use unique keys (with timestamps) to avoid conflicts with other tests or users.

## WebRTC Tests

The `webrtc_tests.rs` file contains **25+ comprehensive tests** covering:

### Configuration Tests
- Default WebRTC options
- Custom ICE servers (STUN/TURN)
- Data channel configuration
- Options cloning

### WebRTCPeer Tests
- Peer connection creation
- SDP offer/answer generation
- Message sending through data channels
- Connection state tracking
- Peer cleanup and closing
- Concurrent peer creation

### WebRTCManager Tests
- Manager initialization
- RTC signaling message handling (offer, answer, ICE candidate)
- Peer discovery and connection initiation
- Self-message filtering
- Invalid message handling
- Connection limit enforcement
- Message forwarding to mesh

### Integration Tests
- Gun instance with WebRTC enabled/disabled
- Invalid SDP handling
- Connection timeouts
- Error scenarios

**Note**: Some WebRTC tests may fail if STUN servers are unreachable (expected in restricted network environments). This is normal and doesn't indicate a code issue.

## Test Coverage

- ✅ Core database operations
- ✅ Chain API
- ✅ Event system
- ✅ Network synchronization
- ✅ Data persistence
- ✅ Error handling
- ✅ Real-time updates
- ✅ WebRTC peer-to-peer connections (25+ tests)
- ✅ NAT traversal (STUN/TURN)
- ✅ SDP offer/answer exchange
- ✅ ICE candidate handling
- ✅ Lock contention and deadlock prevention
- ✅ Stress testing for concurrent operations

