/// Test: Chain method chaining - 10+ method chains (two clients)
/// 
/// Tests very deep method chaining (10+ methods).

use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use tokio::time::Duration;
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: Chain method chaining - 10+ method chains (two clients)");
    println!("Description: Test very deep method chaining");
    
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
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: 10+ method chain - this is a local operation, both clients can test
    println!("\n--- Test: 10+ method chain ---");
    let mut chain = client1.get("a");
    for i in 0..10 {
        chain = chain.get(&format!("level{}", i));
    }
    println!("✓ Client1: 10+ method chain - Success (no panic)");
    success_count += 1;
    
    // Also test on client2
    let mut chain2 = client2.get("a");
    for i in 0..10 {
        chain2 = chain2.get(&format!("level{}", i));
    }
    println!("✓ Client2: 10+ method chain - Success (no panic)");
    
    // Test: 15 method chain with put - Client1 puts, Client2 can verify
    println!("\n--- Test: 15 method chain with put ---");
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let mut chain = client1.get("deep");
    for i in 0..14 {
        chain = chain.get(&format!("l{}_{}", i, timestamp));
    }
    
    match chain.put(json!({"value": "deep"})).await {
        Ok(_) => {
            println!("✓ Client1: 15 method chain with put - Success");
            tokio::time::sleep(Duration::from_millis(1500)).await;
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Client1: 15 method chain with put - Failed - {}", e);
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
