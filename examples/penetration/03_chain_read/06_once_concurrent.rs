/// Test: Chain.once() - Concurrent reads (two clients)
/// 
/// Tests concurrent read operations.

use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{Duration, timeout};

#[tokio::main]
async fn main() {
    println!("Test: Chain.once() - Concurrent reads (two clients)");
    println!("Description: Test concurrent read operations");
    
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
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Setup: Client1 puts data
    println!("\n--- Setup: Client1 putting data ---");
    for i in 0..10 {
        let key = format!("key{}_{}", i, timestamp);
        match client1.get("test").get(&key).put(json!(i)).await {
            Ok(_) => {}
            Err(e) => {
                println!("✗ Client1: Failed to put key{}: {}", i, e);
                fail_count += 1;
            }
        }
    }
    
    tokio::time::sleep(Duration::from_millis(2000)).await;
    
    // Test: Client2 reads concurrently
    println!("\n--- Test: Client2 reading concurrently ---");
    let mut handles = vec![];
    for i in 0..10 {
        let client2_clone = client2.clone();
        let key = format!("key{}_{}", i, timestamp);
        let handle = tokio::spawn(async move {
            let received = Arc::new(AtomicBool::new(false));
            let received_clone = received.clone();
            match timeout(Duration::from_secs(10), client2_clone.get("test").get(&key).once(move |data, _key| {
                if data.as_i64() == Some(i) {
                    received_clone.store(true, Ordering::Relaxed);
                }
            })).await {
                Ok(Ok(_)) => received.load(Ordering::Relaxed),
                Ok(Err(_)) => false,
                Err(_) => false, // timeout
            }
        });
        handles.push(handle);
    }
    
    let mut concurrent_success = 0;
    for handle in handles {
        match handle.await {
            Ok(true) => concurrent_success += 1,
            Ok(false) => fail_count += 1,
            Err(e) => {
                println!("✗ Join error: {:?}", e);
                fail_count += 1;
            }
        }
    }
    
    if concurrent_success == 10 {
        println!("✓ Client2: Concurrent reads - All 10 succeeded");
        success_count += 1;
    } else {
        println!("✗ Client2: Concurrent reads - Only {}/10 succeeded", concurrent_success);
        fail_count += 1;
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
