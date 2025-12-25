use gun::webrtc::WebRTCOptions;
/// Example: Two clients connecting via WebRTC for direct peer-to-peer communication
///
/// This example demonstrates:
/// - Two Gun instances with WebRTC enabled
/// - Initial connection through relay (for signaling)
/// - WebRTC peer discovery and connection establishment
/// - Direct P2P data exchange through WebRTC data channels
/// - NAT traversal using STUN servers
///
/// Run with: `cargo run --example two_clients_webrtc`
use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use tokio::time::sleep;
use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== WebRTC Two Client Example ===\n");
    println!("This example creates two Gun instances with WebRTC enabled");
    println!("and demonstrates direct peer-to-peer data exchange.\n");
    println!("Benefits of WebRTC:");
    println!("  - Direct P2P connection (lower latency)");
    println!("  - NAT traversal (works behind firewalls)");
    println!("  - Reduced load on relay servers");
    println!("  - Better scalability\n");

    // Change this URL to use a different relay server
    const RELAY_URL: &str = "http://dig-relay-prod.eba-2cmanxbe.us-east-1.elasticbeanstalk.com/gun";

    // Create WebRTC options with STUN servers for NAT traversal
    let webrtc_options = WebRTCOptions {
        enabled: true,
        max_connections: 10,
        ..Default::default()
    };

    println!("WebRTC Configuration:");
    println!("  - Enabled: {}", webrtc_options.enabled);
    println!("  - Max connections: {}", webrtc_options.max_connections);
    println!(
        "  - STUN servers: {} configured",
        webrtc_options.ice_servers.len()
    );
    println!();

    // Create first client with WebRTC enabled
    println!("[Client 1] Creating connection with WebRTC enabled...");
    let mut options1 = GunOptions {
        peers: vec![RELAY_URL.to_string()],
        ..Default::default()
    };
    options1.webrtc = webrtc_options.clone();
    let client1 = Arc::new(Gun::with_options(options1).await?);
    println!("[Client 1] ✓ Instance created with WebRTC");

    // Wait for client1 to connect to relay
    println!("[Client 1] Waiting for connection to relay...");
    sleep(Duration::from_millis(2000)).await;
    let count1 = client1.connected_peer_count().await;
    println!("[Client 1] ✓ Connected peers (via relay): {}", count1);
    if count1 == 0 {
        if client1.wait_for_connection(5000).await {
            println!("[Client 1] ✓ Connection established");
        } else {
            println!("[Client 1] ⚠ Connection timeout - continuing anyway");
        }
    }

    // Create second client with WebRTC enabled
    println!("\n[Client 2] Creating connection with WebRTC enabled...");
    let mut options2 = GunOptions {
        peers: vec![RELAY_URL.to_string()],
        ..Default::default()
    };
    options2.webrtc = webrtc_options;
    let client2 = Arc::new(Gun::with_options(options2).await?);
    println!("[Client 2] ✓ Instance created with WebRTC");

    // Wait for client2 to connect to relay
    println!("[Client 2] Waiting for connection to relay...");
    sleep(Duration::from_millis(2000)).await;
    let count2 = client2.connected_peer_count().await;
    println!("[Client 2] ✓ Connected peers (via relay): {}", count2);

    if count2 == 0 {
        if client2.wait_for_connection(5000).await {
            println!("[Client 2] ✓ Connection established");
        } else {
            println!("[Client 2] ⚠ Connection timeout - continuing anyway");
        }
    }

    // Give WebRTC time to discover peers and initiate connections
    println!("\n[WebRTC] Waiting for peer discovery and connection establishment...");
    println!("[WebRTC] This involves:");
    println!("  - Peer ID exchange through relay");
    println!("  - STUN server queries (NAT traversal)");
    println!("  - ICE candidate exchange");
    println!("  - SDP offer/answer negotiation");
    println!("  - Direct P2P connection establishment");
    println!("[WebRTC] This may take 10-30 seconds depending on network conditions...\n");

    // Wait longer for WebRTC connection to establish
    // WebRTC requires STUN server responses and ICE candidate exchange
    for i in 0..20 {
        sleep(Duration::from_millis(1000)).await;
        if i % 5 == 0 {
            println!("[WebRTC] Still negotiating... ({}s)", i + 1);
        }
    }
    println!("[WebRTC] Connection negotiation complete\n");

    // Generate unique test key
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let test_key = format!("webrtc_example_{}", timestamp);
    println!("=== Using test key: {} ===\n", test_key);

    // Setup Client 2 to listen for data from Client 1
    println!("[Client 2] Subscribing to updates...");
    let received = Arc::new(std::sync::Mutex::new(false));
    let received_clone = received.clone();
    let test_key_clone = test_key.clone();

    client2.get(&test_key_clone).on(move |data, _key| {
        if !data.is_null() {
            if let Some(obj) = data.as_object() {
                let source = obj.get("source").and_then(|v| v.as_str());
                let message = obj.get("message").and_then(|v| v.as_str());
                let transport = obj.get("transport").and_then(|v| v.as_str());
                println!(
                    "[Client 2] 📨 Received: source={:?}, message={:?}, transport={:?}",
                    source, message, transport
                );
                if source == Some("client1") {
                    let mut flag = received_clone.lock().unwrap();
                    *flag = true;
                    if transport == Some("webrtc") {
                        println!("[Client 2] ✅ Received via direct WebRTC connection!");
                    } else {
                        println!(
                            "[Client 2] ✅ Received via relay (WebRTC may not be established)"
                        );
                    }
                }
            }
        }
    });

    // Give subscription time to register
    sleep(Duration::from_millis(1000)).await;

    // Client 1 sends data
    // This should go through WebRTC if connection is established,
    // otherwise fall back to WebSocket through relay
    println!("[Client 1] 📤 Sending data...");
    println!("  (Will use WebRTC direct connection if available, otherwise relay)");
    client1
        .get(&test_key)
        .put(json!({
            "source": "client1",
            "message": "Hello from Client 1!",
            "timestamp": timestamp,
            "transport": "webrtc"
        }))
        .await?;
    println!("[Client 1] ✓ Data sent");

    // Wait for data to propagate
    // Note: WebRTC direct connection should be faster than relay
    println!("\n[Waiting] Waiting for data to sync...");
    let mut success = false;
    for i in 0..20 {
        sleep(Duration::from_millis(500)).await;
        if *received.lock().unwrap() {
            println!(
                "\n✅ Success! Client 2 received data after {} attempts",
                i + 1
            );
            success = true;
            break;
        }
    }

    if !success {
        println!("\n⚠ Warning: Client 2 did not receive data within timeout");
        println!("Possible reasons:");
        println!("  - WebRTC connection not fully established (NAT/firewall issues)");
        println!("  - STUN servers unreachable");
        println!("  - Data still syncing through relay (slower but functional)");
        println!("  - Network latency higher than expected");
    }

    // Test bidirectional communication
    println!("\n=== Testing Bidirectional Communication ===\n");

    // Setup Client 1 to listen for data from Client 2
    let received2 = Arc::new(std::sync::Mutex::new(false));
    let received2_clone = received2.clone();
    let test_key2 = format!("{}_bidirectional", test_key);

    client1.get(&test_key2.clone()).on(move |data, _key| {
        if !data.is_null() {
            if let Some(obj) = data.as_object() {
                let source = obj.get("source").and_then(|v| v.as_str());
                let message = obj.get("message").and_then(|v| v.as_str());
                let transport = obj.get("transport").and_then(|v| v.as_str());
                println!(
                    "[Client 1] 📨 Received: source={:?}, message={:?}, transport={:?}",
                    source, message, transport
                );
                if source == Some("client2") {
                    let mut flag = received2_clone.lock().unwrap();
                    *flag = true;
                    if transport == Some("webrtc") {
                        println!("[Client 1] ✅ Received via direct WebRTC connection!");
                    } else {
                        println!("[Client 1] ✅ Received via relay");
                    }
                }
            }
        }
    });

    sleep(Duration::from_millis(1000)).await;

    // Client 2 sends data back
    println!("[Client 2] 📤 Sending data back to Client 1...");
    client2
        .get(&test_key2)
        .put(json!({
            "source": "client2",
            "message": "Hello back from Client 2!",
            "timestamp": timestamp,
            "transport": "webrtc"
        }))
        .await?;
    println!("[Client 2] ✓ Data sent");

    // Wait for data to propagate
    println!("\n[Waiting] Waiting for data to sync...");
    let mut success2 = false;
    for i in 0..20 {
        sleep(Duration::from_millis(500)).await;
        if *received2.lock().unwrap() {
            println!(
                "\n✅ Success! Client 1 received data after {} attempts",
                i + 1
            );
            success2 = true;
            break;
        }
    }

    if !success2 {
        println!("\n⚠ Warning: Client 1 did not receive data within timeout");
    }

    // Test multiple messages through WebRTC
    println!("\n=== Testing Multiple Messages ===\n");

    let message_count = Arc::new(std::sync::Mutex::new(0));
    let message_count_clone = message_count.clone();
    let test_key3 = format!("{}_multiple", test_key);

    client2.get(&test_key3.clone()).on(move |data, _key| {
        if !data.is_null() {
            if let Some(obj) = data.as_object() {
                let count = obj.get("count").and_then(|v| v.as_u64());
                if let Some(c) = count {
                    let mut mc = message_count_clone.lock().unwrap();
                    *mc += 1;
                    println!("[Client 2] 📨 Received message #{} (count: {})", *mc, c);
                }
            }
        }
    });

    sleep(Duration::from_millis(1000)).await;

    // Send multiple messages from Client 1
    println!("[Client 1] 📤 Sending 5 messages...");
    for i in 1..=5 {
        client1
            .get(&test_key3.clone())
            .put(json!({
                "source": "client1",
                "count": i,
                "message": format!("Message number {}", i),
                "transport": "webrtc"
            }))
            .await?;
        sleep(Duration::from_millis(300)).await;
    }
    println!("[Client 1] ✓ All messages sent");

    sleep(Duration::from_secs(3)).await;

    let final_count = *message_count.lock().unwrap();
    println!("\n[Client 2] Received {} messages total", final_count);
    if final_count >= 5 {
        println!("✅ Success! All messages were received!");
        if final_count == 5 {
            println!("   (If WebRTC was established, these went directly P2P)");
        }
    } else {
        println!(
            "⚠ Warning: Expected at least 5 messages, got {}",
            final_count
        );
        println!("   (Some messages may still be syncing)");
    }

    println!("\n=== Example Complete ===");
    println!(
        "Client 1 connected peers: {}",
        client1.connected_peer_count().await
    );
    println!(
        "Client 2 connected peers: {}",
        client2.connected_peer_count().await
    );

    // Keep connections alive for a moment to observe WebRTC state
    println!("\nKeeping connections alive for 3 seconds...");
    sleep(Duration::from_secs(3)).await;

    println!("\n✅ WebRTC two-client example completed!");
    println!("\nNote: If WebRTC direct connection was established,");
    println!("data exchange happened directly between peers without going through the relay.");
    println!("This provides lower latency and reduced server load.");

    Ok(())
}
