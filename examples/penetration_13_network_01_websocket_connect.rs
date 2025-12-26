/// Test: WebSocket connect to valid peer
/// 
/// Tests connecting to a valid WebSocket peer (may fail if peer doesn't exist).

use gun::{Gun, GunOptions};
use chia_bls::SecretKey;

#[tokio::main]
async fn main() {
    println!("Test: WebSocket connect to valid peer");
    println!("Description: Connect to valid WebSocket peer");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    println!("\n--- Test: WebSocket connect ---");
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let options = GunOptions {
        peers: vec!["ws://localhost:8765/gun".to_string()],
        ..Default::default()
    };
    
    match Gun::with_options(secret_key, public_key, options).await {
        Ok(_) => {
            println!("✓ WebSocket connect: Instance created (connection may fail if peer doesn't exist)");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ WebSocket connect: Failed - {}", e);
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

