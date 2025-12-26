/// Test: Chain.map() - Map over empty node
/// 
/// Tests mapping over an empty node.

use gun::Gun;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain.map() - Map over empty node");
    println!("Description: Map over empty node");
    
    let gun = Gun::new();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: Map over empty node
    println!("\n--- Test: Map over empty node ---");
    let properties = Arc::new(Mutex::new(Vec::new()));
    let props_clone = properties.clone();
    
    gun.get("test").get("empty_map").map(move |value, key| {
        props_clone.lock().unwrap().push((key, value.clone()));
    });
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let props = properties.lock().unwrap();
    println!("  Mapped properties: {}", props.len());
    
    // Empty node should have 0 properties
    if props.len() == 0 {
        println!("✓ Map empty: Success (0 properties as expected)");
        success_count += 1;
    } else {
        println!("✗ Map empty: Unexpected properties found");
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

