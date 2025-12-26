/// Test: Chain.once() - Read non-existent data (two clients)
/// 
/// Tests reading data that doesn't exist in the graph.

use gun::{Gun, GunOptions};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{Duration, timeout};

#[tokio::main]
async fn main() {
    println!("Test: Chain.once() - Read non-existent data (two clients)");
    println!("Description: Read data that doesn't exist");
    
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
    let test_key = format!("nonexistent_{}", timestamp);
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test 1: Client2 reads non-existent data (Client1 doesn't put anything) - with timeout
    println!("\n--- Test 1: Client2 reading non-existent data ---");
    let called = Arc::new(AtomicBool::new(false));
    let called_clone = called.clone();
    match timeout(Duration::from_secs(10), client2.get("nonexistent").get(&test_key).once(move |data, _key| {
        called_clone.store(true, Ordering::Relaxed);
        // Data should be null or not found
        if data.is_null() {
            println!("✓ Client2: Received null for missing data (expected)");
        } else {
            println!("? Client2: Received data: {:?} (unexpected)", data);
        }
    })).await {
        Ok(Ok(_)) => {
            if called.load(Ordering::Relaxed) {
                println!("✓ Client2: Callback was called");
                success_count += 1;
            } else {
                println!("✗ Client2: Callback was not called");
                fail_count += 1;
            }
        }
        Ok(Err(e)) => {
            println!("✗ Client2: Read failed: {}", e);
            fail_count += 1;
        }
        Err(_) => {
            println!("✗ Client2: Read timed out after 10 seconds");
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
