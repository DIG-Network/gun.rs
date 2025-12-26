/// Test: WebSocket connect to invalid URL
/// 
/// Tests connecting to an invalid WebSocket URL.

use gun::{Gun, GunOptions};

#[tokio::main]
async fn main() {
    println!("Test: WebSocket connect to invalid URL");
    println!("Description: Connect to invalid WebSocket URL");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    println!("\n--- Test: WebSocket invalid URL ---");
    let options = GunOptions {
        peers: vec!["not-a-valid-url".to_string()],
        ..Default::default()
    };
    
    match Gun::with_options(options).await {
        Ok(_) => {
            println!("✓ WebSocket invalid URL: Instance created (connection will fail)");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ WebSocket invalid URL: Failed - {}", e);
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

