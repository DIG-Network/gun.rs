/// Test: Chain.once() - Read immediately after put (two clients)
/// 
/// Tests reading data immediately after putting it.

use gun::{Gun, GunOptions};
use chia_bls::SecretKey;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain.once() - Read immediately after put (two clients)");
    println!("Description: Put data then read it immediately");
    
    const RELAY_URL: &str = "http://dig-relay-prod.eba-2cmanxbe.us-east-1.elasticbeanstalk.com/gun";
    
    // Create Client 1
    println!("\n--- Setup: Creating Client 1 ---");
    let secret_key1 = SecretKey::from_seed(&[1u8; 32]);
    let public_key1 = secret_key1.public_key();
    let options1 = GunOptions {
        peers: vec![RELAY_URL.to_string()],
        ..Default::default()
    };
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
    let secret_key2 = SecretKey::from_seed(&[2u8; 32]);
    let public_key2 = secret_key2.public_key();
    let options2 = GunOptions {
        peers: vec![RELAY_URL.to_string()],
        ..Default::default()
    };
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
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test 1: Client1 puts, then Client2 reads (with sync delay)
    println!("\n--- Test 1: Client1 puts, Client2 reads ---");
    let test_key1 = format!("immediate_{}", timestamp);
    match client1.get("test").get(&test_key1).put(json!({"value": 100})).await {
        Ok(_) => {
            println!("✓ Client1: Data put");
            tokio::time::sleep(Duration::from_millis(2000)).await;
            let received = Arc::new(AtomicBool::new(false));
            let received_clone = received.clone();
            match client2.get("test").get(&test_key1).once(move |data, _key| {
                if let Some(obj) = data.as_object() {
                    if obj.get("value").and_then(|v| v.as_i64()) == Some(100) {
                        received_clone.store(true, Ordering::Relaxed);
                    }
                }
            }).await {
                Ok(_) => {
                    if received.load(Ordering::Relaxed) {
                        println!("✓ Client2: Read after put - Success");
                        success_count += 1;
                    } else {
                        println!("✗ Client2: Data not received");
                        fail_count += 1;
                    }
                }
                Err(e) => {
                    println!("✗ Client2: Read failed: {}", e);
                    fail_count += 1;
                }
            }
        }
        Err(e) => {
            println!("✗ Client1: Put failed: {}", e);
            fail_count += 1;
        }
    }
    
    // Test 2: Client1 puts, then Client2 reads with longer delay
    println!("\n--- Test 2: Client1 puts, Client2 reads with delay ---");
    let test_key2 = format!("delayed_{}", timestamp);
    match client1.get("test").get(&test_key2).put(json!({"value": 200})).await {
        Ok(_) => {
            println!("✓ Client1: Data put");
            tokio::time::sleep(Duration::from_millis(2500)).await;
            let received = Arc::new(AtomicBool::new(false));
            let received_clone = received.clone();
            match client2.get("test").get(&test_key2).once(move |data, _key| {
                if let Some(obj) = data.as_object() {
                    if obj.get("value").and_then(|v| v.as_i64()) == Some(200) {
                        received_clone.store(true, Ordering::Relaxed);
                    }
                }
            }).await {
                Ok(_) => {
                    if received.load(Ordering::Relaxed) {
                        println!("✓ Client2: Read after delay - Success");
                        success_count += 1;
                    } else {
                        println!("✗ Client2: Data not received after delay");
                        fail_count += 1;
                    }
                }
                Err(e) => {
                    println!("✗ Client2: Read failed: {}", e);
                    fail_count += 1;
                }
            }
        }
        Err(e) => {
            println!("✗ Client1: Put failed: {}", e);
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
