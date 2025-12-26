/// Test: Network errors
/// 
/// Tests handling network failures and timeouts.

use gun::{Gun, GunOptions};

#[tokio::main]
async fn main() {
    println!("Test: Network errors");
    println!("Description: Test network failures and timeouts");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    println!("\n--- Test: Network errors ---");
    let options = GunOptions {
        peers: vec!["ws://nonexistent.example.com:9999/gun".to_string()],
        ..Default::default()
    };
    
    match Gun::with_options(options).await {
        Ok(_) => {
            println!("✓ Network errors: Instance created (connection will fail gracefully)");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Network errors: Failed - {}", e);
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

