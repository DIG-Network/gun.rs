/// Test: Chain.put() concurrent operations (two clients)
/// 
/// Tests concurrent puts to same and different keys.

use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use tokio::time::Duration;
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: Chain.put() concurrent operations (two clients)");
    println!("Description: Test concurrent puts to same/different keys");
    
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
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test 1: Client1 concurrent puts to different keys
    println!("\n--- Test 1: Client1 concurrent puts to different keys ---");
    let mut handles = vec![];
    for i in 0..10 {
        let client1_clone = client1.clone();
        let key = format!("key{}_{}", i, timestamp);
        let handle = tokio::spawn(async move {
            client1_clone.get(&key).put(json!(i)).await
        });
        handles.push(handle);
    }
    
    let mut concurrent_success = 0;
    for handle in handles {
        match handle.await {
            Ok(Ok(_)) => concurrent_success += 1,
            Ok(Err(e)) => {
                println!("✗ Concurrent different keys: Error - {}", e);
                fail_count += 1;
            }
            Err(e) => {
                println!("✗ Concurrent different keys: Join error - {:?}", e);
                fail_count += 1;
            }
        }
    }
    
    if concurrent_success == 10 {
        println!("✓ Client1: Concurrent different keys - All 10 succeeded");
        tokio::time::sleep(Duration::from_millis(2000)).await;
        success_count += 1;
    }
    
    // Test 2: Client1 concurrent puts to same key
    println!("\n--- Test 2: Client1 concurrent puts to same key ---");
    let same_key = format!("same_key_{}", timestamp);
    let mut handles = vec![];
    for i in 0..10 {
        let client1_clone = client1.clone();
        let key = same_key.clone();
        let handle = tokio::spawn(async move {
            client1_clone.get(&key).put(json!(i)).await
        });
        handles.push(handle);
    }
    
    let mut concurrent_same_success = 0;
    for handle in handles {
        match handle.await {
            Ok(Ok(_)) => concurrent_same_success += 1,
            Ok(Err(e)) => {
                println!("✗ Concurrent same key: Error - {}", e);
                fail_count += 1;
            }
            Err(e) => {
                println!("✗ Concurrent same key: Join error - {:?}", e);
                fail_count += 1;
            }
        }
    }
    
    if concurrent_same_success == 10 {
        println!("✓ Client1: Concurrent same key - All 10 succeeded");
        tokio::time::sleep(Duration::from_millis(2000)).await;
        success_count += 1;
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
