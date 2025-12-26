/// Test: Chain method chaining - get().on().put() (two clients)
/// 
/// Tests subscribing then writing (get().on().put()).

use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::time::Duration;
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: Chain method chaining - get().on().put() (two clients)");
    println!("Description: Subscribe then write (get().on().put())");
    
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
    let test_key = format!("chained5_{}", timestamp);
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: Client2 get().on(), Client1 put()
    println!("\n--- Test: Client2 get().on(), Client1 put() ---");
    let update_count = Arc::new(AtomicU32::new(0));
    let count_clone = update_count.clone();
    let key_clone = test_key.clone();
    
    let chain = client2.get("test").get(&key_clone);
    chain.on(move |data, _key| {
        if data.is_object() {
            count_clone.fetch_add(1, Ordering::Relaxed);
        }
    });
    
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    match client1.get("test").get(&test_key).put(json!({"value": 400})).await {
        Ok(_) => {
            println!("✓ Client1: put() - Success");
            tokio::time::sleep(Duration::from_millis(2000)).await;
            
            let count = update_count.load(Ordering::Relaxed);
            if count > 0 {
                println!("✓ Client2: get().on().put() - Success");
                success_count += 1;
            } else {
                println!("✗ Client2: get().on().put() - No updates received");
                fail_count += 1;
            }
        }
        Err(e) => {
            println!("✗ Client1: get().on().put() - Put failed - {}", e);
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
