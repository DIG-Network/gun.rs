/// Test: Chain method chaining - get().back().get() (two clients)
/// 
/// Tests chaining get(), back(), and get() methods.

use gun::{Gun, GunOptions};
use std::sync::Arc;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain method chaining - get().back().get() (two clients)");
    println!("Description: Test get().back().get() chaining");
    
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
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: get().back().get() - this is a local operation, both clients can test
    println!("\n--- Test: get().back().get() ---");
    let chain = client1.get("level1").get("level2").get("level3");
    
    match chain.back(Some(1)) {
        Some(back_chain) => {
            let _next_chain = back_chain.get("new_key");
            println!("✓ Client1: get().back().get() - Success");
            success_count += 1;
        }
        None => {
            println!("✗ Client1: get().back().get() - Back returned None");
            fail_count += 1;
        }
    }
    
    // Also test on client2
    let chain2 = client2.get("level1").get("level2").get("level3");
    match chain2.back(Some(1)) {
        Some(back_chain) => {
            let _next_chain = back_chain.get("new_key");
            println!("✓ Client2: get().back().get() - Success");
        }
        None => {
            println!("✗ Client2: get().back().get() - Back returned None");
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
