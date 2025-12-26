/// Test: Chain.once() - Read nested properties (two clients)
/// 
/// Tests reading nested properties from the graph.

use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{Duration, timeout};

#[tokio::main]
async fn main() {
    println!("Test: Chain.once() - Read nested properties (two clients)");
    println!("Description: Read nested properties from graph");
    
    const RELAY_URL: &str = "http://dig-relay-prod.eba-2cmanxbe.us-east-1.elasticbeanstalk.com/gun";
    
    // Create Client 1
    println!("\n--- Setup: Creating Client 1 ---");
    let options1 = GunOptions {
        peers: vec![RELAY_URL.to_string()],
        ..Default::default()
    };
    let client1 = match Gun::with_options(options1).await {
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
    let client2 = match Gun::with_options(options2).await {
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
    let test_key = format!("nested_{}", timestamp);
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Setup: Client1 puts nested data
    println!("\n--- Setup: Client1 putting nested data ---");
    match client1.get("test").get(&test_key).put(json!({
        "level1": {
            "level2": {
                "level3": "deep value"
            }
        }
    })).await {
        Ok(_) => println!("✓ Client1: Nested data put"),
        Err(e) => {
            println!("✗ Client1: Failed to put: {}", e);
            fail_count += 1;
        }
    }
    
    tokio::time::sleep(Duration::from_millis(2000)).await;
    
    // Test 1: Client2 reads top level
    println!("\n--- Test 1: Client2 reading top level ---");
    let received = Arc::new(AtomicBool::new(false));
    let received_clone = received.clone();
    match timeout(Duration::from_secs(10), client2.get("test").get(&test_key).once(move |data, _key| {
        if data.is_object() {
            received_clone.store(true, Ordering::Relaxed);
        }
    })).await {
        Ok(Ok(_)) => {
            if received.load(Ordering::Relaxed) {
                println!("✓ Client2: Top level read - Success");
                success_count += 1;
            } else {
                fail_count += 1;
            }
        }
        Ok(Err(e)) => {
            println!("✗ Client2: Top level read failed: {}", e);
            fail_count += 1;
        }
        Err(_) => {
            println!("✗ Client2: Top level read timed out after 10 seconds");
            fail_count += 1;
        }
    }
    
    // Test 2: Client2 reads nested property
    println!("\n--- Test 2: Client2 reading nested property ---");
    let received = Arc::new(AtomicBool::new(false));
    let received_clone = received.clone();
    match timeout(Duration::from_secs(10), client2.get("test").get(&test_key).get("level1").once(move |data, _key| {
        if data.is_object() {
            received_clone.store(true, Ordering::Relaxed);
        }
    })).await {
        Ok(Ok(_)) => {
            if received.load(Ordering::Relaxed) {
                println!("✓ Client2: Nested property read - Success");
                success_count += 1;
            } else {
                fail_count += 1;
            }
        }
        Ok(Err(e)) => {
            println!("✗ Client2: Nested property read failed: {}", e);
            fail_count += 1;
        }
        Err(_) => {
            println!("✗ Client2: Nested property read timed out after 10 seconds");
            fail_count += 1;
        }
    }
    
    // Test 3: Client2 reads deeply nested property
    println!("\n--- Test 3: Client2 reading deeply nested property ---");
    let received = Arc::new(AtomicBool::new(false));
    let received_clone = received.clone();
    match timeout(Duration::from_secs(10), client2.get("test").get(&test_key).get("level1").get("level2").get("level3").once(move |data, _key| {
        if data.as_str() == Some("deep value") {
            received_clone.store(true, Ordering::Relaxed);
        }
    })).await {
        Ok(Ok(_)) => {
            if received.load(Ordering::Relaxed) {
                println!("✓ Client2: Deeply nested read - Success");
                success_count += 1;
            } else {
                fail_count += 1;
            }
        }
        Ok(Err(e)) => {
            println!("✗ Client2: Deeply nested read failed: {}", e);
            fail_count += 1;
        }
        Err(_) => {
            println!("✗ Client2: Deeply nested read timed out after 10 seconds");
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
