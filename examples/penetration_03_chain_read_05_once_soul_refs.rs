/// Test: Chain.once() - Read through soul references
/// 
/// Tests reading data through soul references.

use gun::Gun;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain.once() - Read through soul references");
    println!("Description: Read data through soul references");
    
    let gun = Gun::new();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Setup: Create nodes and references
    println!("\n--- Setup: Create nodes and references ---");
    let node_soul = "ref_node_123";
    
    match gun.get(node_soul).put(json!({"name": "Referenced Node", "value": 999})).await {
        Ok(_) => println!("✓ Created referenced node"),
        Err(e) => {
            println!("✗ Failed to create node: {}", e);
            fail_count += 1;
        }
    }
    
    match gun.get("test").get("ref").put(json!({"#": node_soul})).await {
        Ok(_) => println!("✓ Created soul reference"),
        Err(e) => {
            println!("✗ Failed to create reference: {}", e);
            fail_count += 1;
        }
    }
    
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    // Test 1: Read through soul reference
    println!("\n--- Test 1: Read through soul reference ---");
    let received = Arc::new(AtomicBool::new(false));
    let received_clone = received.clone();
    match gun.get("test").get("ref").once(move |data, _key| {
        // Should get the referenced node data
        if let Some(obj) = data.as_object() {
            if obj.get("name").and_then(|v| v.as_str()) == Some("Referenced Node") {
                received_clone.store(true, Ordering::Relaxed);
            }
        }
    }).await {
        Ok(_) => {
            if received.load(Ordering::Relaxed) {
                println!("✓ Read through soul ref: Success");
                success_count += 1;
            } else {
                println!("✗ Data not received through reference");
                fail_count += 1;
            }
        }
        Err(e) => {
            println!("✗ Read through soul ref failed: {}", e);
            fail_count += 1;
        }
    }
    
    // Test 2: Read directly from soul
    println!("\n--- Test 2: Read directly from soul ---");
    let received = Arc::new(AtomicBool::new(false));
    let received_clone = received.clone();
    match gun.get(node_soul).once(move |data, _key| {
        if let Some(obj) = data.as_object() {
            if obj.get("value").and_then(|v| v.as_i64()) == Some(999) {
                received_clone.store(true, Ordering::Relaxed);
            }
        }
    }).await {
        Ok(_) => {
            if received.load(Ordering::Relaxed) {
                println!("✓ Direct soul read: Success");
                success_count += 1;
            } else {
                fail_count += 1;
            }
        }
        Err(e) => {
            println!("✗ Direct soul read failed: {}", e);
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

