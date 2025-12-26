/// Test: Chain.once() - Read existing data
/// 
/// Tests reading data that exists in the graph using two clients.

use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain.once() - Read existing data");
    println!("Description: Read data that exists in the graph using two clients");
    
    const RELAY_URL: &str = "http://dig-relay-prod.eba-2cmanxbe.us-east-1.elasticbeanstalk.com/gun";
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
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
    
    // Generate unique test key
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let test_key = format!("test_read_test_{}", timestamp);
    
    // Client 1 puts data
    println!("\n--- Setup: Client 1 putting data ---");
    match client1.get("test").get(&test_key).put(json!({"value": 42})).await {
        Ok(_) => println!("✓ Data put successfully by Client 1"),
        Err(e) => {
            println!("✗ Failed to put data: {}", e);
            fail_count += 1;
        }
    }
    
    // Wait for data to sync
    tokio::time::sleep(Duration::from_millis(2000)).await;
    
    // Client 2 reads the data
    println!("\n--- Test 1: Client 2 reading existing data ---");
    let received = Arc::new(AtomicBool::new(false));
    let received_clone = received.clone();
    match client2.get("test").get(&test_key).once(move |data, _key| {
        if let Some(obj) = data.as_object() {
            if obj.get("value").and_then(|v| v.as_i64()) == Some(42) {
                received_clone.store(true, Ordering::Relaxed);
                println!("✓ Data received correctly by Client 2");
            }
        }
    }).await {
        Ok(_) => {
            if received.load(Ordering::Relaxed) {
                success_count += 1;
            } else {
                println!("✗ Data not received or incorrect by Client 2");
                fail_count += 1;
            }
        }
        Err(e) => {
            println!("✗ Read failed: {}", e);
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
