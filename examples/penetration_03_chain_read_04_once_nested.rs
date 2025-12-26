/// Test: Chain.once() - Read nested properties
/// 
/// Tests reading nested properties from the graph.

use gun::Gun;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain.once() - Read nested properties");
    println!("Description: Read nested properties from graph");
    
    let gun = Gun::new();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Setup: Put nested data
    println!("\n--- Setup: Put nested data ---");
    match gun.get("test").get("nested").put(json!({
        "level1": {
            "level2": {
                "level3": "deep value"
            }
        }
    })).await {
        Ok(_) => println!("✓ Nested data put"),
        Err(e) => {
            println!("✗ Failed to put: {}", e);
            fail_count += 1;
        }
    }
    
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    // Test 1: Read top level
    println!("\n--- Test 1: Read top level ---");
    let received = Arc::new(AtomicBool::new(false));
    let received_clone = received.clone();
    match gun.get("test").get("nested").once(move |data, _key| {
        if data.is_object() {
            received_clone.store(true, Ordering::Relaxed);
        }
    }).await {
        Ok(_) => {
            if received.load(Ordering::Relaxed) {
                println!("✓ Top level read: Success");
                success_count += 1;
            } else {
                fail_count += 1;
            }
        }
        Err(e) => {
            println!("✗ Top level read failed: {}", e);
            fail_count += 1;
        }
    }
    
    // Test 2: Read nested property
    println!("\n--- Test 2: Read nested property ---");
    let received = Arc::new(AtomicBool::new(false));
    let received_clone = received.clone();
    match gun.get("test").get("nested").get("level1").once(move |data, _key| {
        if data.is_object() {
            received_clone.store(true, Ordering::Relaxed);
        }
    }).await {
        Ok(_) => {
            if received.load(Ordering::Relaxed) {
                println!("✓ Nested property read: Success");
                success_count += 1;
            } else {
                fail_count += 1;
            }
        }
        Err(e) => {
            println!("✗ Nested property read failed: {}", e);
            fail_count += 1;
        }
    }
    
    // Test 3: Read deeply nested property
    println!("\n--- Test 3: Read deeply nested property ---");
    let received = Arc::new(AtomicBool::new(false));
    let received_clone = received.clone();
    match gun.get("test").get("nested").get("level1").get("level2").get("level3").once(move |data, _key| {
        if data.as_str() == Some("deep value") {
            received_clone.store(true, Ordering::Relaxed);
        }
    }).await {
        Ok(_) => {
            if received.load(Ordering::Relaxed) {
                println!("✓ Deeply nested read: Success");
                success_count += 1;
            } else {
                fail_count += 1;
            }
        }
        Err(e) => {
            println!("✗ Deeply nested read failed: {}", e);
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

