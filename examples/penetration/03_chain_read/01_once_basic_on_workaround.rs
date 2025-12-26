/// Test: Chain.once() - Read existing data (using on() workaround)
/// 
/// This is a workaround version that uses on() instead of once() to handle
/// network sync timing issues. It subscribes, waits for first update, then unsubscribes.

use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{Duration, timeout};
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: Chain.once() - Read existing data (on() workaround)");
    println!("Description: Read data using on() workaround for network sync");
    
    const RELAY_URL: &str = "http://dig-relay-prod.eba-2cmanxbe.us-east-1.elasticbeanstalk.com/gun";
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
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
    
    // Generate unique test key
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let test_key = format!("test_read_test_{}", timestamp);
    
    // Client 1 puts data
    println!("\n--- Setup: Client 1 putting data ---");
    match client1.get("test").get(&test_key).put(json!({"value": 42})).await {
        Ok(_) => println!("✓ Data put successfully by Client 1"),
        Err(e) => {
            println!("✗ Failed to put data: {}", e);
            fail_count += 1;
        }
    }
    
    // Wait for data to sync (longer delay for network sync)
    tokio::time::sleep(Duration::from_millis(5000)).await;
    
    // Client 2 reads the data using on() workaround
    println!("\n--- Test 1: Client 2 reading existing data (on() workaround) ---");
    let received = Arc::new(AtomicBool::new(false));
    let received_clone = received.clone();
    let test_key_clone = test_key.clone();
    
    // Subscribe with on() and store the chain for later unsubscribe
    let chain = client2.get("test").get(&test_key_clone);
    let listener_id = chain.on(move |data, _key| {
        if let Some(obj) = data.as_object() {
            if obj.get("value").and_then(|v| v.as_i64()) == Some(42) {
                received_clone.store(true, Ordering::Relaxed);
                println!("✓ Data received correctly by Client 2");
            }
        }
    });
    
    // Wait for data with timeout
    match timeout(Duration::from_secs(15), async {
        let start = std::time::Instant::now();
        while start.elapsed() < Duration::from_secs(15) {
            if received.load(Ordering::Relaxed) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }).await {
        Ok(_) => {
            if received.load(Ordering::Relaxed) {
                success_count += 1;
                // Unsubscribe - note: off() doesn't take listener_id, it removes all
                chain.off();
            } else {
                println!("✗ Data not received or incorrect by Client 2");
                fail_count += 1;
            }
        }
        Err(_) => {
            println!("✗ Read timed out after 15 seconds");
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

