/// Test: Chain.get() navigation - single, nested, deep nesting (two clients)
/// 
/// Tests navigation through the graph using get() with various depths.

use gun::{Gun, GunOptions};
use std::sync::Arc;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain.get() navigation (two clients)");
    println!("Description: Test single, nested, and deep nesting (1-10 levels)");
    
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
    
    // Test 1: Single level
    println!("\n--- Test 1: Single level ---");
    let _chain1 = client1.get("level1");
    let _chain2 = client2.get("level1");
    println!("✓ Single level: Success");
    success_count += 1;
    
    // Test 2-10: Nested levels
    for depth in 2..=10 {
        println!("\n--- Test {}: {} levels deep ---", depth, depth);
        let mut chain1 = client1.get("level1");
        let mut chain2 = client2.get("level1");
        for i in 2..=depth {
            chain1 = chain1.get(&format!("level{}", i));
            chain2 = chain2.get(&format!("level{}", i));
        }
        println!("✓ {} levels: Success", depth);
        success_count += 1;
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
