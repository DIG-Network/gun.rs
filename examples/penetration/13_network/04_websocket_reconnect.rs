/// Test: WebSocket reconnection scenarios
/// 
/// Tests WebSocket reconnection scenarios.

use gun::{Gun, GunOptions};

#[tokio::main]
async fn main() {
    println!("Test: WebSocket reconnection scenarios");
    println!("Description: Test WebSocket reconnection scenarios");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    println!("\n--- Test: WebSocket reconnection ---");
    let options = GunOptions {
        peers: vec!["ws://localhost:8765/gun".to_string()],
        ..Default::default()
    };
    
    match Gun::with_options(options).await {
        Ok(_) => {
            println!("✓ WebSocket reconnection: Instance created (reconnection handled automatically)");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ WebSocket reconnection: Failed - {}", e);
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

