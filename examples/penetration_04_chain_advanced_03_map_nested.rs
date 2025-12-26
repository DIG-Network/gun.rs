/// Test: Chain.map() - Map with nested objects
/// 
/// Tests mapping with nested objects.

use gun::Gun;
use serde_json::json;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain.map() - Map with nested objects");
    println!("Description: Map with nested objects");
    
    let gun = Gun::new();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Setup: Put nested data
    println!("\n--- Setup: Put nested data ---");
    match gun.get("test").get("map_nested").put(json!({
        "user1": {"name": "Alice", "age": 30},
        "user2": {"name": "Bob", "age": 28}
    })).await {
        Ok(_) => println!("✓ Nested data put"),
        Err(e) => {
            println!("✗ Failed to put: {}", e);
            fail_count += 1;
        }
    }
    
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    // Test: Map over nested properties
    println!("\n--- Test: Map over nested properties ---");
    let properties = Arc::new(Mutex::new(Vec::new()));
    let props_clone = properties.clone();
    
    gun.get("test").get("map_nested").map(move |value, key| {
        props_clone.lock().unwrap().push((key, value.clone()));
    });
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let props = properties.lock().unwrap();
    println!("  Mapped properties: {}", props.len());
    
    if props.len() >= 2 {
        println!("✓ Map nested: Success (found {} properties)", props.len());
        success_count += 1;
    } else {
        println!("✗ Map nested: Failed (found {} properties)", props.len());
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

