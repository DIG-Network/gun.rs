/// Test: Chain.put() with soul references (two clients)
/// 
/// Tests putting direct soul refs, nested refs, and circular refs.

use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use tokio::time::Duration;
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: Chain.put() with soul references (two clients)");
    println!("Description: Test direct soul refs, nested refs, circular refs");
    
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
    
    let node1_soul = format!("node1_soul_{}", timestamp);
    let node2_soul = format!("node2_soul_{}", timestamp);
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Setup: Client1 creates nodes to reference
    println!("\n--- Setup: Client1 creating nodes ---");
    match client1.get(&node1_soul).put(json!({"name": "Node 1"})).await {
        Ok(_) => println!("✓ Client1: Created node1"),
        Err(e) => {
            println!("✗ Client1: Failed to create node1: {}", e);
            fail_count += 1;
        }
    }
    
    match client1.get(&node2_soul).put(json!({"name": "Node 2"})).await {
        Ok(_) => println!("✓ Client1: Created node2"),
        Err(e) => {
            println!("✗ Client1: Failed to create node2: {}", e);
            fail_count += 1;
        }
    }
    
    tokio::time::sleep(Duration::from_millis(1500)).await;
    
    // Test 1: Direct soul reference - Client1 puts, Client2 verifies
    println!("\n--- Test 1: Direct soul reference ---");
    let test_key1 = format!("ref1_{}", timestamp);
    match client1.get("test").get(&test_key1).put(json!({"#": node1_soul})).await {
        Ok(_) => {
            println!("✓ Client1: Direct soul ref put");
            tokio::time::sleep(Duration::from_millis(1500)).await;
            println!("✓ Direct soul ref: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Direct soul ref: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 2: Nested soul reference - Client1 puts, Client2 verifies
    println!("\n--- Test 2: Nested soul reference ---");
    let test_key2 = format!("nested_ref_{}", timestamp);
    match client1.get("test").get(&test_key2).put(json!({
        "ref": {"#": node1_soul},
        "other": "data"
    })).await {
        Ok(_) => {
            println!("✓ Client1: Nested soul ref put");
            tokio::time::sleep(Duration::from_millis(1500)).await;
            println!("✓ Nested soul ref: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Nested soul ref: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 3: Multiple soul references - Client1 puts, Client2 verifies
    println!("\n--- Test 3: Multiple soul references ---");
    let test_key3 = format!("multi_ref_{}", timestamp);
    match client1.get("test").get(&test_key3).put(json!({
        "ref1": {"#": node1_soul},
        "ref2": {"#": node2_soul}
    })).await {
        Ok(_) => {
            println!("✓ Client1: Multiple soul refs put");
            tokio::time::sleep(Duration::from_millis(1500)).await;
            println!("✓ Multiple soul refs: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Multiple soul refs: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 4: Circular reference - Client1 creates
    println!("\n--- Test 4: Circular reference ---");
    match client1.get(&node1_soul).put(json!({
        "name": "Node 1",
        "friend": {"#": node2_soul}
    })).await {
        Ok(_) => {
            match client1.get(&node2_soul).put(json!({
                "name": "Node 2",
                "friend": {"#": node1_soul}
            })).await {
                Ok(_) => {
                    println!("✓ Client1: Circular ref created");
                    tokio::time::sleep(Duration::from_millis(1500)).await;
                    println!("✓ Circular ref: Success");
                    success_count += 1;
                }
                Err(e) => {
                    println!("✗ Circular ref (step 2): Failed - {}", e);
                    fail_count += 1;
                }
            }
        }
        Err(e) => {
            println!("✗ Circular ref (step 1): Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 5: Self-reference - Client1 creates
    println!("\n--- Test 5: Self-reference ---");
    match client1.get(&node1_soul).put(json!({
        "name": "Node 1",
        "self": {"#": node1_soul}
    })).await {
        Ok(_) => {
            println!("✓ Client1: Self-reference created");
            tokio::time::sleep(Duration::from_millis(1500)).await;
            println!("✓ Self-reference: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Self-reference: Failed - {}", e);
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
