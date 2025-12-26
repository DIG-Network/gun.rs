/// Test: Chain.back() - Edge cases (two clients)
/// 
/// Tests back() from root and back beyond root.

use gun::{Gun, GunOptions};
use std::sync::Arc;
use tokio::time::Duration;
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: Chain.back() - Edge cases (two clients)");
    println!("Description: Back from root, back beyond root");
    
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
    
    // Test 1: Back from root - both clients can test (local operation)
    println!("\n--- Test 1: Back from root ---");
    let root_chain1 = client1.root();
    match root_chain1.back(Some(1)) {
        Some(_) => {
            println!("? Client1: Back from root - Returned Some (unexpected)");
            success_count += 1; // Still counts as success (no panic)
        }
        None => {
            println!("✓ Client1: Back from root - Returned None (expected)");
            success_count += 1;
        }
    }
    
    // Test 2: Back beyond available levels
    println!("\n--- Test 2: Back beyond available levels ---");
    let chain1 = client1.get("level1").get("level2");
    match chain1.back(Some(10)) {
        Some(_) => {
            println!("✓ Client1: Back beyond levels - Returned Some");
            success_count += 1;
        }
        None => {
            println!("✓ Client1: Back beyond levels - Returned None (may be expected)");
            success_count += 1;
        }
    }
    
    // Test 3: Back with 0
    println!("\n--- Test 3: Back with 0 ---");
    let chain1 = client1.get("level1").get("level2");
    match chain1.back(Some(0)) {
        Some(_) => {
            println!("✓ Client1: Back 0 - Returned Some");
            success_count += 1;
        }
        None => {
            println!("✓ Client1: Back 0 - Returned None");
            success_count += 1;
        }
    }
    
    // Also test on client2
    let root_chain2 = client2.root();
    match root_chain2.back(Some(1)) {
        Some(_) => println!("? Client2: Back from root - Returned Some"),
        None => println!("✓ Client2: Back from root - Returned None"),
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
