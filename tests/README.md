# Test Suite

This directory contains the test suite for Gun.rs.

## Test Files

- `basic_tests.rs` - Unit tests for core functionality (no network required)
- `integration_tests.rs` - Integration tests that connect to a real Gun.js relay server

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

### Run Integration Tests with Output
```bash
cargo test --test integration_tests -- --nocapture
```

### Run Specific Test
```bash
cargo test --test integration_tests test_relay_put_get
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

## Test Coverage

- ✅ Core database operations
- ✅ Chain API
- ✅ Event system
- ✅ Network synchronization
- ✅ Data persistence
- ✅ Error handling
- ✅ Real-time updates

