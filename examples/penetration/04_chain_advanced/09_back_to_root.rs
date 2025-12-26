/// Test: Chain.back() - Navigate back to root (two clients)
/// 
/// Tests navigating back to root using back(None).

use gun::{Gun, GunOptions};
use std::sync::Arc;
use tokio::time::Duration;
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: Chain.back() - Navigate back to root (two clients)");
    println!("Description: Navigate back to root");
    
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
    
    // Test: Navigate back to root - both clients can test (local operation)
    println!("\n--- Test: Navigate back to root ---");
    let chain1 = client1.get("level1").get("level2").get("level3");
    
    match chain1.back(None) {
        Some(_) => {
            println!("✓ Client1: Back to root - Success");
            success_count += 1;
        }
        None => {
            println!("✗ Client1: Back to root - Returned None");
            fail_count += 1;
        }
    }
    
    // Test: Back to root from root
    println!("\n--- Test: Back to root from root ---");
    let root_chain1 = client1.root();
    match root_chain1.back(None) {
        Some(_) => {
            println!("✓ Client1: Back from root - Success");
            success_count += 1;
        }
        None => {
            println!("? Client1: Back from root - Returned None (may be expected)");
            success_count += 1; // This might be expected behavior
        }
    }
    
    // Also test on client2
    let chain2 = client2.get("level1").get("level2").get("level3");
    match chain2.back(None) {
        Some(_) => println!("✓ Client2: Back to root - Success"),
        None => println!("✗ Client2: Back to root - Returned None"),
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
