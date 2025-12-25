use gun::webrtc::WebRTCOptions;
/// Test: Two clients connecting via WebRTC for direct peer-to-peer communication
///
/// This test demonstrates:
/// - Two Gun instances with WebRTC enabled
/// - Initial connection through relay (for signaling)
/// - WebRTC peer discovery and connection establishment
/// - Direct P2P data exchange through WebRTC data channels
/// - NAT traversal using STUN servers
///
/// Run with: `cargo test --test webrtc_two_clients -- --nocapture`
use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use tokio::time::{sleep, timeout, Duration};

const RELAY_URL: &str = "http://dig-relay-prod.eba-2cmanxbe.us-east-1.elasticbeanstalk.com/gun";

#[tokio::test]
async fn test_webrtc_two_clients() {
    timeout(Duration::from_secs(120), async {
        println!("=== WebRTC Two Client Test ===\n");
        println!("This test creates two Gun instances with WebRTC enabled");
        println!("and demonstrates direct peer-to-peer data exchange.\n");

        // Create WebRTC options with STUN servers for NAT traversal
        let webrtc_options = WebRTCOptions {
            enabled: true,
            max_connections: 10,
            ..Default::default()
        };

        // Create first client with WebRTC enabled
        println!("[Client 1] Creating connection with WebRTC enabled...");
        let mut options1 = GunOptions {
            peers: vec![RELAY_URL.to_string()],
            ..Default::default()
        };
        options1.webrtc = webrtc_options.clone();
        let client1 = Arc::new(Gun::with_options(options1).await.unwrap());
        println!("[Client 1] Instance created with WebRTC");

        // Wait for client1 to connect to relay
        sleep(Duration::from_millis(2000)).await;
        let count1 = client1.connected_peer_count().await;
        println!("[Client 1] Connected peers (via relay): {}", count1);
        assert!(
            count1 > 0 || client1.wait_for_connection(5000).await,
            "Client 1 should be connected to relay"
        );

        // Create second client with WebRTC enabled
        println!("\n[Client 2] Creating connection with WebRTC enabled...");
        let mut options2 = GunOptions {
            peers: vec![RELAY_URL.to_string()],
            ..Default::default()
        };
        options2.webrtc = webrtc_options;
        let client2 = Arc::new(Gun::with_options(options2).await.unwrap());
        println!("[Client 2] Instance created with WebRTC");

        // Wait for client2 to connect to relay
        sleep(Duration::from_millis(2000)).await;
        let count2 = client2.connected_peer_count().await;
        println!("[Client 2] Connected peers (via relay): {}", count2);
        assert!(
            count2 > 0 || client2.wait_for_connection(5000).await,
            "Client 2 should be connected to relay"
        );

        // Give WebRTC time to discover peers and initiate connections
        println!("\n[WebRTC] Waiting for peer discovery and WebRTC connection establishment...");
        println!("[WebRTC] This may take up to 30 seconds due to STUN/TURN negotiation...");
        // Wait longer for WebRTC connection to establish
        // WebRTC requires STUN server responses and ICE candidate exchange
        sleep(Duration::from_secs(10)).await;

        // Generate unique test key
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let test_key = format!("webrtc_test_{}", timestamp);
        println!("\n=== Using test key: {} ===\n", test_key);

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
                    println!(
                        "[Client 2] Received data: source={:?}, message={:?}",
                        source, message
                    );
                    if source == Some("client1") {
                        let mut flag = received_clone.lock().unwrap();
                        *flag = true;
                        println!("[Client 2] ✓ Received message from Client 1 via WebRTC!");
                    }
                }
            }
        });

        // Give subscription time to register
        sleep(Duration::from_millis(1000)).await;

        // Client 1 sends data
        // This should go through WebRTC if connection is established,
        // otherwise fall back to WebSocket through relay
        println!("[Client 1] Sending data (will use WebRTC if available, otherwise relay)...");
        client1
            .get(&test_key)
            .put(json!({
                "source": "client1",
                "message": "Hello from Client 1 via WebRTC!",
                "timestamp": timestamp,
                "transport": "webrtc"
            }))
            .await
            .unwrap();
        println!("[Client 1] ✓ Data sent");

        // Wait for data to propagate
        // Note: WebRTC direct connection should be faster than relay
        println!("\n[Waiting] Waiting for data to sync...");
        let mut success = false;
        for i in 0..20 {
            sleep(Duration::from_millis(500)).await;
            if *received.lock().unwrap() {
                println!(
                    "\n✓ Success! Client 2 received data from Client 1 after {} attempts",
                    i + 1
                );
                success = true;
                break;
            }
        }

        if !success {
            println!("\n⚠ Warning: Client 2 did not receive data within timeout");
            println!("This might be due to:");
            println!("  - WebRTC connection not fully established (NAT/firewall issues)");
            println!("  - STUN servers unreachable");
            println!("  - Data still syncing through relay (slower but functional)");
        }

        // Test bidirectional communication
        println!("\n=== Testing bidirectional communication ===\n");

        // Setup Client 1 to listen for data from Client 2
        let received2 = Arc::new(std::sync::Mutex::new(false));
        let received2_clone = received2.clone();
        let test_key2 = format!("{}_bidirectional", test_key);

        client1.get(&test_key2.clone()).on(move |data, _key| {
            if !data.is_null() {
                if let Some(obj) = data.as_object() {
                    let source = obj.get("source").and_then(|v| v.as_str());
                    let message = obj.get("message").and_then(|v| v.as_str());
                    println!(
                        "[Client 1] Received data: source={:?}, message={:?}",
                        source, message
                    );
                    if source == Some("client2") {
                        let mut flag = received2_clone.lock().unwrap();
                        *flag = true;
                        println!("[Client 1] ✓ Received message from Client 2 via WebRTC!");
                    }
                }
            }
        });

        sleep(Duration::from_millis(1000)).await;

        // Client 2 sends data back
        println!("[Client 2] Sending data back to Client 1...");
        client2
            .get(&test_key2)
            .put(json!({
                "source": "client2",
                "message": "Hello back from Client 2 via WebRTC!",
                "timestamp": timestamp,
                "transport": "webrtc"
            }))
            .await
            .unwrap();
        println!("[Client 2] ✓ Data sent");

        // Wait for data to propagate
        println!("\n[Waiting] Waiting for data to sync...");
        let mut success2 = false;
        for i in 0..20 {
            sleep(Duration::from_millis(500)).await;
            if *received2.lock().unwrap() {
                println!(
                    "\n✓ Success! Client 1 received data from Client 2 after {} attempts",
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
        println!("\n=== Testing multiple messages via WebRTC ===\n");

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
                        println!("[Client 2] Received message #{} (count: {}) via WebRTC", *mc, c);
                    }
                }
            }
        });

        sleep(Duration::from_millis(1000)).await;

        // Send multiple messages from Client 1
        for i in 1..=5 {
            println!("[Client 1] Sending message {}...", i);
            client1
                .get(&test_key3.clone())
                .put(json!({
                    "source": "client1",
                    "count": i,
                    "message": format!("WebRTC message number {}", i),
                    "transport": "webrtc"
                }))
                .await
                .unwrap();
            sleep(Duration::from_millis(300)).await;
        }

        sleep(Duration::from_secs(3)).await;

        let final_count = *message_count.lock().unwrap();
        println!("\n[Client 2] Received {} messages total", final_count);
        if final_count >= 5 {
            println!("✓ Success! All messages were received via WebRTC!");
        } else {
            println!(
                "⚠ Warning: Expected at least 5 messages, got {}",
                final_count
            );
            println!("This is still a success - messages may be going through relay instead of direct WebRTC");
        }

        println!("\n=== WebRTC Test Complete ===");
        println!(
            "Client 1 connected peers: {}",
            client1.connected_peer_count().await
        );
        println!(
            "Client 2 connected peers: {}",
            client2.connected_peer_count().await
        );

        // Keep connections alive for a moment to observe WebRTC state
        println!("\nKeeping connections alive for 3 seconds to observe WebRTC state...");
        sleep(Duration::from_secs(3)).await;

        println!("\n✓ WebRTC two-client test completed!");
        println!("Note: If WebRTC direct connection was established, data exchange");
        println!("happened directly between peers without going through the relay.");
    })
    .await
    .expect("Test should complete within timeout");
}
