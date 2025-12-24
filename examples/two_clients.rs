/// Example: Two clients connecting to a relay server and exchanging data
///
/// This example demonstrates:
/// - Creating two separate Gun instances
/// - Both connecting to the same relay server
/// - Client 2 subscribing to updates from Client 1
/// - Client 1 sending data that propagates through the relay to Client 2
/// - Bidirectional communication between clients
/// - Multiple message handling
///
/// Run with: `cargo run --example two_clients`
use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Two Client Relay Test ===\n");
    println!("This example creates two Gun instances that connect to a relay server");
    println!("and demonstrates data synchronization between them.\n");

    // Change this URL to use a different relay server
    const RELAY_URL: &str = "http://dig-relay-prod.eba-2cmanxbe.us-east-1.elasticbeanstalk.com/gun";

    // Create first client
    println!("[Client 1] Creating connection to relay...");
    let options1 = GunOptions {
        peers: vec![RELAY_URL.to_string()],
        ..Default::default()
    };
    let client1 = Arc::new(Gun::with_options(options1).await?);
    println!("[Client 1] Instance created");

    // Wait for client1 to connect
    sleep(Duration::from_millis(2000)).await;
    let count1 = client1.connected_peer_count().await;
    println!("[Client 1] Connected peers: {}", count1);
    assert!(count1 > 0 || client1.wait_for_connection(5000).await, 
           "Client 1 should be connected");

    // Create second client
    println!("\n[Client 2] Creating connection to relay...");
    let options2 = GunOptions {
        peers: vec![RELAY_URL.to_string()],
        ..Default::default()
    };
    let client2 = Arc::new(Gun::with_options(options2).await?);
    println!("[Client 2] Instance created");

    // Wait for client2 to connect
    sleep(Duration::from_millis(2000)).await;
    let count2 = client2.connected_peer_count().await;
    println!("[Client 2] Connected peers: {}", count2);
    assert!(count2 > 0 || client2.wait_for_connection(5000).await,
           "Client 2 should be connected");

    // Generate unique test key
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let test_key = format!("two_clients_test_{}", timestamp);
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
                println!("[Client 2] Received data: source={:?}, message={:?}", source, message);
                if source == Some("client1") {
                    let mut flag = received_clone.lock().unwrap();
                    *flag = true;
                    println!("[Client 2] ✓ Received message from Client 1!");
                }
            }
        }
    });

    // Give subscription time to register
    sleep(Duration::from_millis(500)).await;

    // Client 1 sends data
    println!("[Client 1] Sending data...");
    client1
        .get(&test_key)
        .put(json!({
            "source": "client1",
            "message": "Hello from Client 1!",
            "timestamp": timestamp
        }))
        .await?;
    println!("[Client 1] ✓ Data sent");

    // Wait for data to propagate through relay
    println!("\n[Waiting] Waiting for data to sync through relay...");
    for i in 0..10 {
        sleep(Duration::from_millis(500)).await;
        if *received.lock().unwrap() {
            println!("\n✓ Success! Client 2 received data from Client 1 after {} attempts", i + 1);
            break;
        }
        if i == 9 {
            println!("\n⚠ Warning: Client 2 did not receive data within timeout");
        }
    }

    // Test bidirectional communication
    println!("\n=== Testing bidirectional communication ===\n");

    // Setup Client 1 to listen for data from Client 2
    let received2 = Arc::new(std::sync::Mutex::new(false));
    let received2_clone = received2.clone();
    let test_key2 = format!("{}_bidirectional", test_key);

    client1.get(&test_key2).on(move |data, _key| {
        if !data.is_null() {
            if let Some(obj) = data.as_object() {
                let source = obj.get("source").and_then(|v| v.as_str());
                let message = obj.get("message").and_then(|v| v.as_str());
                println!("[Client 1] Received data: source={:?}, message={:?}", source, message);
                if source == Some("client2") {
                    let mut flag = received2_clone.lock().unwrap();
                    *flag = true;
                    println!("[Client 1] ✓ Received message from Client 2!");
                }
            }
        }
    });

    sleep(Duration::from_millis(500)).await;

    // Client 2 sends data back
    println!("[Client 2] Sending data back to Client 1...");
    client2
        .get(&test_key2)
        .put(json!({
            "source": "client2",
            "message": "Hello back from Client 2!",
            "timestamp": timestamp
        }))
        .await?;
    println!("[Client 2] ✓ Data sent");

    // Wait for data to propagate
    println!("\n[Waiting] Waiting for data to sync...");
    for i in 0..10 {
        sleep(Duration::from_millis(500)).await;
        if *received2.lock().unwrap() {
            println!("\n✓ Success! Client 1 received data from Client 2 after {} attempts", i + 1);
            break;
        }
        if i == 9 {
            println!("\n⚠ Warning: Client 1 did not receive data within timeout");
        }
    }

    // Test multiple messages
    println!("\n=== Testing multiple messages ===\n");

    let message_count = Arc::new(std::sync::Mutex::new(0));
    let message_count_clone = message_count.clone();
    let test_key3 = format!("{}_multiple", test_key);

    client2.get(&test_key3).on(move |data, _key| {
        if !data.is_null() {
            if let Some(obj) = data.as_object() {
                let count = obj.get("count").and_then(|v| v.as_u64());
                if let Some(c) = count {
                    let mut mc = message_count_clone.lock().unwrap();
                    *mc += 1;
                    println!("[Client 2] Received message #{} (count: {})", *mc, c);
                }
            }
        }
    });

    sleep(Duration::from_millis(500)).await;

    // Send multiple messages from Client 1
    for i in 1..=5 {
        println!("[Client 1] Sending message {}...", i);
        client1
            .get(&test_key3)
            .put(json!({
                "source": "client1",
                "count": i,
                "message": format!("Message number {}", i)
            }))
            .await?;
        sleep(Duration::from_millis(200)).await;
    }

    sleep(Duration::from_millis(2000)).await;

    let final_count = *message_count.lock().unwrap();
    println!("\n[Client 2] Received {} messages total", final_count);
    if final_count >= 5 {
        println!("✓ Success! All messages were received");
    } else {
        println!("⚠ Warning: Expected at least 5 messages, got {}", final_count);
    }

    println!("\n=== Test Complete ===");
    println!("Client 1 connected peers: {}", client1.connected_peer_count().await);
    println!("Client 2 connected peers: {}", client2.connected_peer_count().await);

    // Keep connections alive for a moment
    println!("\nKeeping connections alive for 2 seconds...");
    sleep(Duration::from_secs(2)).await;

    Ok(())
}

