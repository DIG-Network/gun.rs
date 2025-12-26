/// Test: Chain.put() with content addressing (hash-prefixed souls) (two clients)
/// 
/// Tests putting data with #hash souls for content addressing.

use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain.put() with content addressing (two clients)");
    println!("Description: Test hash-prefixed souls (#hash)");
    
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
    
    // Test 1: Content addressing with hash - Client1 puts
    println!("\n--- Test 1: Client1 content addressing with hash ---");
    let data = json!({"content": "test data"});
    let data_string = serde_json::to_string(&data).unwrap();
    
    // Generate hash (simplified - in practice, this would use SEA.work)
    use sha2::{Sha256, Digest};
    use base64::{engine::general_purpose, Engine as _};
use chia_bls::{SecretKey, PublicKey};
    let mut hasher = Sha256::new();
    hasher.update(data_string.as_bytes());
    let hash = general_purpose::STANDARD_NO_PAD.encode(hasher.finalize());
    let hash_soul = format!("#{}", hash);
    
    match client1.get(&hash_soul).put(data.clone()).await {
        Ok(_) => {
            println!("✓ Client1: Content addressing - Success");
            tokio::time::sleep(Duration::from_millis(3000)).await;
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Client1: Content addressing - Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 2: Invalid hash (should still work, but hash won't match) - Client1 puts
    println!("\n--- Test 2: Client1 invalid hash format ---");
    let invalid_hash_soul = format!("#invalid_hash_{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis());
    match client1.get(&invalid_hash_soul).put(json!({"data": "test"})).await {
        Ok(_) => {
            println!("✓ Client1: Invalid hash format - Success (hash won't verify)");
            tokio::time::sleep(Duration::from_millis(3000)).await;
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Client1: Invalid hash format - Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 3: Hash with different data (should fail verification) - Client1 puts
    println!("\n--- Test 3: Client1 hash mismatch ---");
    let wrong_data = json!({"content": "different data"});
    match client1.get(&hash_soul).put(wrong_data).await {
        Ok(_) => {
            println!("✓ Client1: Hash mismatch - Put succeeded (verification may fail)");
            tokio::time::sleep(Duration::from_millis(3000)).await;
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Client1: Hash mismatch - Failed - {}", e);
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
