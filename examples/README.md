# Gun.rs Examples

This directory contains example applications demonstrating how to use gun.rs.

## Examples

### `two_clients.rs`
Demonstrates two Gun instances connecting to a relay server and exchanging data bidirectionally.

**Features:**
- Two separate client instances
- Connection verification
- Unidirectional data flow (Client 1 → Client 2)
- Bidirectional data flow (Client 1 ↔ Client 2)
- Multiple message sending

**Run:**
```bash
cargo run --example two_clients
```

**What it does:**
1. Creates two Gun instances (Client 1 and Client 2)
2. Both connect to the same relay server
3. Client 2 subscribes to updates
4. Client 1 sends data to Client 2 through the relay
5. Tests bidirectional communication
6. Tests multiple message sending

### `basic.rs`
Basic example showing single instance usage.

### `graph.rs`
Example demonstrating graph operations.

### `relay.rs`
Example showing how to run a relay server.

## Running Examples

All examples can be run with:
```bash
cargo run --example <example_name>
```

For example:
```bash
cargo run --example two_clients
```

## Relay Server

The examples use the production relay server:
- URL: `http://dig-relay-prod.eba-2cmanxbe.us-east-1.elasticbeanstalk.com/gun`

You can change this by modifying the `RELAY_URL` constant in each example.

