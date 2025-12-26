/// Test: Persistence across restarts
/// 
/// Tests persistence across restarts (if supported).

use gun::{Gun, GunOptions};
use tempfile::TempDir;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Persistence across restarts");
    println!("Description: Test persistence across restarts");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    println!("\n--- Test: Persistence ---");
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let storage_path = temp_dir.path().to_str().unwrap().to_string();
    
    // Create first instance and write data
    let options1 = GunOptions {
        radisk: true,
        storage_path: Some(storage_path.clone()),
        ..Default::default()
    };
    
    match Gun::with_options(options1).await {
        Ok(gun1) => {
            match gun1.get("test").get("persist").put(json!({"value": 42})).await {
                Ok(_) => {
                    println!("✓ Data written");
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    
                    // Create second instance and read data
                    let options2 = GunOptions {
                        radisk: true,
                        storage_path: Some(storage_path),
                        ..Default::default()
                    };
                    
                    match Gun::with_options(options2).await {
                        Ok(gun2) => {
                            let received = Arc::new(AtomicBool::new(false));
                            let received_clone = received.clone();
                            match gun2.get("test").get("persist").once(move |data, _key| {
                                if let Some(obj) = data.as_object() {
                                    if obj.get("value").and_then(|v| v.as_i64()) == Some(42) {
                                        received_clone.store(true, Ordering::Relaxed);
                                    }
                                }
                            }).await {
                                Ok(_) => {
                                    if received.load(Ordering::Relaxed) {
                                        println!("✓ Persistence: Success");
                                        success_count += 1;
                                    } else {
                                        println!("✗ Persistence: Data not received");
                                        fail_count += 1;
                                    }
                                }
                                Err(e) => {
                                    println!("✗ Persistence: Read failed - {}", e);
                                    fail_count += 1;
                                }
                            }
                        }
                        Err(e) => {
                            println!("✗ Second instance creation failed - {}", e);
                            fail_count += 1;
                        }
                    }
                }
                Err(e) => {
                    println!("✗ Write failed - {}", e);
                    fail_count += 1;
                }
            }
        }
        Err(e) => {
            println!("✗ First instance creation failed - {}", e);
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

