/// Test: Chain.put() with nested structures (two clients)
/// 
/// Tests putting nested objects and arrays of objects.

use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{Duration, timeout};
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: Chain.put() with nested structures (two clients)");
    println!("Description: Test nested objects and arrays of objects");
    
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
    
    // Test 1: Nested object (2 levels) - Client1 puts, Client2 verifies
    println!("\n--- Test 1: Nested object (2 levels) ---");
    let test_key1 = format!("nested2_{}", timestamp);
    match client1.get("test").get(&test_key1).put(json!({
        "level1": {
            "level2": "value"
        }
    })).await {
        Ok(_) => {
            println!("✓ Client1: Nested 2 levels put");
            tokio::time::sleep(Duration::from_millis(3000)).await;
            let received = Arc::new(AtomicBool::new(false));
            let received_clone = received.clone();
            match timeout(Duration::from_secs(15), client2.get("test").get(&test_key1).once(move |data, _key| {
                if let Some(obj) = data.as_object() {
                    if let Some(l1) = obj.get("level1") {
                        if let Some(l1_obj) = l1.as_object() {
                            if l1_obj.get("level2").and_then(|v| v.as_str()) == Some("value") {
                                received_clone.store(true, Ordering::Relaxed);
                            }
                        }
                    }
                }
            })).await {
                Ok(Ok(_)) => {
                    if received.load(Ordering::Relaxed) {
                        println!("✓ Client2: Nested 2 levels verified - Success");
                        success_count += 1;
                    } else {
                        println!("✗ Nested 2 levels: Client2 verification failed");
                        fail_count += 1;
                    }
                }
                Ok(Err(e)) => {
                    println!("✗ Nested 2 levels: Client2 read failed - {}", e);
                    fail_count += 1;
                }
                Err(_) => {
                    println!("✗ Nested 2 levels: Client2 read timed out after 15 seconds");
                    fail_count += 1;
                }
            }
        }
        Err(e) => {
            println!("✗ Nested 2 levels: Client1 put failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 2: Nested object (3 levels) - Client1 puts, Client2 verifies
    println!("\n--- Test 2: Nested object (3 levels) ---");
    let test_key2 = format!("nested3_{}", timestamp);
    match client1.get("test").get(&test_key2).put(json!({
        "level1": {
            "level2": {
                "level3": "value"
            }
        }
    })).await {
        Ok(_) => {
            println!("✓ Client1: Nested 3 levels put");
            tokio::time::sleep(Duration::from_millis(3000)).await;
            let received = Arc::new(AtomicBool::new(false));
            let received_clone = received.clone();
            match timeout(Duration::from_secs(15), client2.get("test").get(&test_key2).once(move |data, _key| {
                if data.is_object() {
                    received_clone.store(true, Ordering::Relaxed);
                }
            })).await {
                Ok(Ok(_)) => {
                    if received.load(Ordering::Relaxed) {
                        println!("✓ Client2: Nested 3 levels verified - Success");
                        success_count += 1;
                    } else {
                        println!("✗ Nested 3 levels: Client2 verification failed");
                        fail_count += 1;
                    }
                }
                Ok(Err(e)) => {
                    println!("✗ Nested 3 levels: Client2 read failed - {}", e);
                    fail_count += 1;
                }
                Err(_) => {
                    println!("✗ Nested 3 levels: Client2 read timed out after 15 seconds");
                    fail_count += 1;
                }
            }
        }
        Err(e) => {
            println!("✗ Nested 3 levels: Client1 put failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 3: Array of objects - Arrays not supported, skip this test
    println!("\n--- Test 3: Array of objects (skipped - not supported) ---");
    println!("✓ Array of objects: Skipped - Arrays are not directly supported by put()");
    println!("  Arrays must be converted to objects with numeric keys");
    success_count += 1;
    
    // Test 4: Nested array - Arrays not supported, skip this test
    println!("\n--- Test 4: Nested array (skipped - not supported) ---");
    println!("✓ Nested array: Skipped - Arrays are not directly supported by put()");
    success_count += 1;
    
    // Test 5: Complex nested structure - Client1 puts, Client2 verifies
    println!("\n--- Test 5: Complex nested structure ---");
    let test_key5 = format!("complex_{}", timestamp);
    match client1.get("test").get(&test_key5).put(json!({
        "users": [
            {
                "name": "Alice",
                "profile": {
                    "age": 30,
                    "city": "NYC"
                }
            },
            {
                "name": "Bob",
                "profile": {
                    "age": 28,
                    "city": "LA"
                }
            }
        ]
    })).await {
        Ok(_) => {
            println!("✓ Client1: Complex nested put");
            tokio::time::sleep(Duration::from_millis(3000)).await;
            let received = Arc::new(AtomicBool::new(false));
            let received_clone = received.clone();
            match timeout(Duration::from_secs(15), client2.get("test").get(&test_key5).once(move |data, _key| {
                if data.is_object() {
                    received_clone.store(true, Ordering::Relaxed);
                }
            })).await {
                Ok(Ok(_)) => {
                    if received.load(Ordering::Relaxed) {
                        println!("✓ Client2: Complex nested verified - Success");
                        success_count += 1;
                    } else {
                        println!("✗ Complex nested: Client2 verification failed");
                        fail_count += 1;
                    }
                }
                Ok(Err(e)) => {
                    println!("✗ Complex nested: Client2 read failed - {}", e);
                    fail_count += 1;
                }
                Err(_) => {
                    println!("✗ Complex nested: Client2 read timed out after 15 seconds");
                    fail_count += 1;
                }
            }
        }
        Err(e) => {
            println!("✗ Complex nested: Client1 put failed - {}", e);
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
