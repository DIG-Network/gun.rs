/// Test: Chain.once() - Read through soul references (two clients)
/// 
/// Tests reading data through soul references.

use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{Duration, timeout};
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: Chain.once() - Read through soul references (two clients)");
    println!("Description: Read data through soul references");
    
    const RELAY_URL: &str = "http://dig-relay-prod.eba-2cmanxbe.us-east-1.elasticbeanstalk.com/gun";
    
    // Create Client 1
    println!("\n--- Setup: Creating Client 1 ---");
    let options1 = GunOptions {
        peers: vec![RELAY_URL.to_string()],
        ..Default::default()
    };
    // Generate BLS key pair
    let secret_key1 = SecretKey::from_seed(&[1 u8; 32]);
    let public_key1 = secret_key1.public_key();
    let client1 = match Gun::with_options(secret_key1, public_key1, options1).await {
        Ok(g) => Arc::new(g),
        Err(e) => {
            println!("✗ Failed to create Client 1: {}", e);
            std::process::exit(1);
        }
    };
    tokio::time::sleep(Duration::from_millis(1000)).await;
    
    // Create Client 2
    println!("--- Setup: Creating Client 2 ---");
    let options2 = GunOptions {
        peers: vec![RELAY_URL.to_string()],
        ..Default::default()
    };
    // Generate BLS key pair
    let secret_key2 = SecretKey::from_seed(&[2 u8; 32]);
    let public_key2 = secret_key2.public_key();
    let client2 = match Gun::with_options(secret_key2, public_key2, options2).await {
        Ok(g) => Arc::new(g),
        Err(e) => {
            println!("✗ Failed to create Client 2: {}", e);
            std::process::exit(1);
        }
    };
    tokio::time::sleep(Duration::from_millis(1000)).await;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let node_soul = format!("ref_node_{}", timestamp);
    let test_key = format!("ref_{}", timestamp);
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Setup: Client1 creates nodes and references
    println!("\n--- Setup: Client1 creating nodes and references ---");
    match client1.get(&node_soul).put(json!({"name": "Referenced Node", "value": 999})).await {
        Ok(_) => println!("✓ Client1: Created referenced node"),
        Err(e) => {
            println!("✗ Client1: Failed to create node: {}", e);
            fail_count += 1;
        }
    }
    
    match client1.get("test").get(&test_key).put(json!({"#": node_soul})).await {
        Ok(_) => println!("✓ Client1: Created soul reference"),
        Err(e) => {
            println!("✗ Client1: Failed to create reference: {}", e);
            fail_count += 1;
        }
    }
    
    tokio::time::sleep(Duration::from_millis(3000)).await;
    
    // Test 1: Client2 reads through soul reference
    println!("\n--- Test 1: Client2 reading through soul reference ---");
    let received = Arc::new(AtomicBool::new(false));
    let received_clone = received.clone();
    match timeout(Duration::from_secs(15), client2.get("test").get(&test_key).once(move |data, _key| {
        // Should get the referenced node data
        if let Some(obj) = data.as_object() {
            if obj.get("name").and_then(|v| v.as_str()) == Some("Referenced Node") {
                received_clone.store(true, Ordering::Relaxed);
            }
        }
    })).await {
        Ok(Ok(_)) => {
            if received.load(Ordering::Relaxed) {
                println!("✓ Client2: Read through soul ref - Success");
                success_count += 1;
            } else {
                println!("✗ Client2: Data not received through reference");
                fail_count += 1;
            }
        }
        Ok(Err(e)) => {
            println!("✗ Client2: Read through soul ref failed: {}", e);
            fail_count += 1;
        }
        Err(_) => {
            println!("✗ Client2: Read through soul ref timed out after 15 seconds");
            fail_count += 1;
        }
    }
    
    // Test 2: Client2 reads directly from soul
    println!("\n--- Test 2: Client2 reading directly from soul ---");
    let received = Arc::new(AtomicBool::new(false));
    let received_clone = received.clone();
    match timeout(Duration::from_secs(15), client2.get(&node_soul).once(move |data, _key| {
        if let Some(obj) = data.as_object() {
            if obj.get("value").and_then(|v| v.as_i64()) == Some(999) {
                received_clone.store(true, Ordering::Relaxed);
            }
        }
    })).await {
        Ok(Ok(_)) => {
            if received.load(Ordering::Relaxed) {
                println!("✓ Client2: Direct soul read - Success");
                success_count += 1;
            } else {
                fail_count += 1;
            }
        }
        Ok(Err(e)) => {
            println!("✗ Client2: Direct soul read failed: {}", e);
            fail_count += 1;
        }
        Err(_) => {
            println!("✗ Client2: Direct soul read timed out after 15 seconds");
            fail_count += 1;
        }
    }
    
    println!("\n--- Summary ---");
    println!("Success: {}", success_count);
    println!("Failed: {}", fail_count);
    
    if fail_count == 0 {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}
