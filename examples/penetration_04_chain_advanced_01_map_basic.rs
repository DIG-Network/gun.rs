/// Test: Chain.map() - Map over properties
/// 
/// Tests mapping over node properties.

use gun::Gun;
use serde_json::json;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain.map() - Map over properties");
    println!("Description: Map over node properties");
    
    let gun = Gun::new();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Setup: Put data with multiple properties
    println!("\n--- Setup: Put data ---");
    match gun.get("test").get("map_test").put(json!({
        "a": 1,
        "b": 2,
        "c": 3
    })).await {
        Ok(_) => println!("✓ Data put"),
        Err(e) => {
            println!("✗ Failed to put: {}", e);
            fail_count += 1;
        }
    }
    
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    // Test: Map over properties
    println!("\n--- Test: Map over properties ---");
    let properties = Arc::new(Mutex::new(Vec::new()));
    let props_clone = properties.clone();
    
    gun.get("test").get("map_test").map(move |value, key| {
        props_clone.lock().unwrap().push((key, value.clone()));
    });
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let props = properties.lock().unwrap();
    println!("  Mapped properties: {}", props.len());
    
    if props.len() >= 3 {
        println!("✓ Map: Success (found {} properties)", props.len());
        success_count += 1;
    } else {
        println!("✗ Map: Failed (found {} properties, expected >= 3)", props.len());
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

