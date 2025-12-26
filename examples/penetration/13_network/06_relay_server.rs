/// Test: Start relay server
/// 
/// Tests starting a relay server.

use gun::{Gun, GunOptions};

#[tokio::main]
async fn main() {
    println!("Test: Start relay server");
    println!("Description: Test starting a relay server");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    println!("\n--- Test: Relay server ---");
    let options = GunOptions {
        super_peer: true,
        port: Some(8765),
        ..Default::default()
    };
    
    match Gun::with_options(options).await {
        Ok(_) => {
            println!("✓ Relay server: Instance created");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Relay server: Failed - {}", e);
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

