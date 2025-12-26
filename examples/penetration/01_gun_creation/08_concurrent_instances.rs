/// Test: Multiple Gun instances created concurrently (two clients approach)
/// 
/// Tests creating multiple Gun instances simultaneously.

use gun::{Gun, GunOptions};
use std::sync::Arc;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Multiple Gun instances created concurrently (two clients)");
    println!("Description: Create multiple instances simultaneously");
    
    const RELAY_URL: &str = "http://dig-relay-prod.eba-2cmanxbe.us-east-1.elasticbeanstalk.com/gun";
    
    let mut handles = vec![];
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Create multiple instances concurrently
    for i in 0..10 {
        let handle = tokio::spawn(async move {
            let options = GunOptions {
                peers: vec![RELAY_URL.to_string()],
                ..Default::default()
            };
            match Gun::with_options(options).await {
                Ok(gun) => {
                    println!("✓ Instance {}: Created successfully", i);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    Ok(gun)
                }
                Err(e) => {
                    println!("✗ Instance {}: Failed - {}", i, e);
                    Err(format!("Failed to create instance {}", i))
                }
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        match handle.await {
            Ok(Ok(_)) => success_count += 1,
            Ok(Err(e)) => {
                println!("Error: {}", e);
                fail_count += 1;
            }
            Err(e) => {
                println!("Join error: {:?}", e);
                fail_count += 1;
            }
        }
    }
    
    // Give time for any async operations
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    println!("\n--- Summary ---");
    println!("Success: {}", success_count);
    println!("Failed: {}", fail_count);
    
    if fail_count == 0 {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}
