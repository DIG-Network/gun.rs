/// Test: Chain.map() - Map over empty node (two clients)
/// 
/// Tests mapping over an empty node.

use gun::{Gun, GunOptions};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain.map() - Map over empty node (two clients)");
    println!("Description: Map over empty node");
    
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
    let test_key = format!("empty_map_{}", timestamp);
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: Client2 maps over empty node (Client1 doesn't put anything)
    println!("\n--- Test: Client2 mapping over empty node ---");
    let properties = Arc::new(Mutex::new(Vec::new()));
    let props_clone = properties.clone();
    let key_clone = test_key.clone();
    
    client2.get("test").get(&key_clone).map(move |value, key| {
        props_clone.lock().unwrap().push((key, value.clone()));
    });
    
    tokio::time::sleep(Duration::from_millis(2000)).await;
    
    let props = properties.lock().unwrap();
    println!("  Client2: Mapped properties: {}", props.len());
    
    // Empty node should have 0 properties
    if props.len() == 0 {
        println!("✓ Client2: Map empty - Success (0 properties as expected)");
        success_count += 1;
    } else {
        println!("✗ Client2: Map empty - Unexpected properties found");
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
