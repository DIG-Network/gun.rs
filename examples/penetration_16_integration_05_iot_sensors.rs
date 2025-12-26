/// Test: IoT sensor data collection
/// 
/// Tests IoT sensor data collection.

use gun::Gun;
use serde_json::json;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    println!("Test: IoT sensor data collection");
    println!("Description: Test IoT sensor data collection");
    
    let gun = Arc::new(Gun::new());
    let mut success_count = 0;
    let mut fail_count = 0;
    
    println!("\n--- Test: IoT sensor data collection ---");
    
    // Simulate multiple sensors sending data
    let mut handles = vec![];
    for i in 0..10 {
        let gun_clone = gun.clone();
        let handle = tokio::spawn(async move {
            gun_clone.get("sensors").get(&format!("sensor{}", i)).put(json!({
                "temperature": 20.0 + (i as f64),
                "humidity": 50.0 + (i as f64),
                "timestamp": i
            })).await
        });
        handles.push(handle);
    }
    
    let mut sensor_success = 0;
    for handle in handles {
        if handle.await.is_ok() {
            sensor_success += 1;
        }
    }
    
    if sensor_success == 10 {
        println!("✓ IoT sensor data: All 10 sensors sent data");
        success_count += 1;
    } else {
        println!("✗ IoT sensor data: Only {}/10 sensors sent data", sensor_success);
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

