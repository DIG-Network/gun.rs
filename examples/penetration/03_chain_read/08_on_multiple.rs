/// Test: Chain.on() - Multiple subscriptions (two clients)
/// 
/// Tests multiple subscriptions to the same data.

use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::time::Duration;
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: Chain.on() - Multiple subscriptions (two clients)");
    println!("Description: Multiple subscriptions to same data");
    
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
    let test_key = format!("multi_{}", timestamp);
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: Client2 multiple subscriptions, Client1 puts data
    println!("\n--- Test: Client2 multiple subscriptions, Client1 puts ---");
    let count1 = Arc::new(AtomicU32::new(0));
    let count2 = Arc::new(AtomicU32::new(0));
    let count3 = Arc::new(AtomicU32::new(0));
    
    let c1 = count1.clone();
    let key1 = test_key.clone();
    client2.get("test").get(&key1).on(move |data, _key| {
        if data.is_i64() {
            c1.fetch_add(1, Ordering::Relaxed);
        }
    });
    
    let c2 = count2.clone();
    let key2 = test_key.clone();
    client2.get("test").get(&key2).on(move |data, _key| {
        if data.is_i64() {
            c2.fetch_add(1, Ordering::Relaxed);
        }
    });
    
    let c3 = count3.clone();
    let key3 = test_key.clone();
    client2.get("test").get(&key3).on(move |data, _key| {
        if data.is_i64() {
            c3.fetch_add(1, Ordering::Relaxed);
        }
    });
    
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Client1 puts data multiple times
    for i in 0..5 {
        match client1.get("test").get(&test_key).put(json!(i)).await {
            Ok(_) => {}
            Err(e) => {
                println!("✗ Client1: Put failed: {}", e);
                fail_count += 1;
            }
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
    
    tokio::time::sleep(Duration::from_millis(2000)).await;
    
    let c1_val = count1.load(Ordering::Relaxed);
    let c2_val = count2.load(Ordering::Relaxed);
    let c3_val = count3.load(Ordering::Relaxed);
    
    println!("  Client2 Listener 1: {} updates", c1_val);
    println!("  Client2 Listener 2: {} updates", c2_val);
    println!("  Client2 Listener 3: {} updates", c3_val);
    
    if c1_val > 0 && c2_val > 0 && c3_val > 0 {
        println!("✓ Multiple subscriptions: All listeners received updates");
        success_count += 1;
    } else {
        println!("✗ Multiple subscriptions: Some listeners didn't receive updates");
        fail_count += 1;
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
