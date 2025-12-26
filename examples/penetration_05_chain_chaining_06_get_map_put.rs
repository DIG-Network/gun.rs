/// Test: Chain method chaining - get().map().put()
/// 
/// Tests chaining get(), map(), and put() methods.

use gun::Gun;
use serde_json::json;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain method chaining - get().map().put()");
    println!("Description: Test get().map().put() chaining");
    
    let gun = Gun::new();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Setup: Put initial data
    match gun.get("test").get("map_chain").put(json!({"a": 1, "b": 2})).await {
        Ok(_) => {}
        Err(e) => {
            println!("✗ Setup failed: {}", e);
            fail_count += 1;
        }
    }
    
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    // Test: get().map().put()
    println!("\n--- Test: get().map().put() ---");
    let properties = Arc::new(Mutex::new(Vec::new()));
    let props_clone = properties.clone();
    
    let chain = gun.get("test").get("map_chain");
    chain.map(move |value, key| {
        props_clone.lock().unwrap().push((key, value.clone()));
    });
    
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    match chain.put(json!({"c": 3})).await {
        Ok(_) => {
            let props = properties.lock().unwrap();
            if props.len() >= 2 {
                println!("✓ get().map().put(): Success");
                success_count += 1;
            } else {
                println!("✗ get().map().put(): Map didn't work");
                fail_count += 1;
            }
        }
        Err(e) => {
            println!("✗ get().map().put(): Put failed - {}", e);
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

