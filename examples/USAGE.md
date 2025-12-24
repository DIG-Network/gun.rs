# Running the Two Clients Example

## Quick Start

```bash
cargo run --example two_clients
```

## What This Example Does

This example demonstrates how to:
1. Create two separate Gun instances (Client 1 and Client 2)
2. Connect both to the same relay server
3. Subscribe to data updates
4. Send data from one client to another through the relay
5. Test bidirectional communication

## Expected Output

When run successfully, you should see:
- Both clients connecting to the relay
- Connection verification (connected peer count)
- Client 1 sending data
- Client 2 receiving data from Client 1
- Bidirectional communication test
- Multiple message sending test

## Customization

To use a different relay server, modify the `RELAY_URL` constant in `two_clients.rs`:

```rust
const RELAY_URL: &str = "ws://your-relay-server.com/gun";
```

## Troubleshooting

### Connections not establishing
- Check that the relay server URL is correct and accessible
- Verify network connectivity
- Check firewall settings

### Data not being received
- Ensure both clients show connected peer count > 0
- Wait longer for relay propagation (the example has timeouts)
- Check relay server logs for message routing

### Timeout warnings
- This may indicate network latency or relay routing delays
- Try increasing sleep durations in the example
- Verify the relay server is functioning correctly

