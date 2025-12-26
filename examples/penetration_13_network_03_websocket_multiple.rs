/// Test: WebSocket multiple peers
/// 
/// Tests connecting to multiple WebSocket peers.

use gun::{Gun, GunOptions};
use chia_bls::SecretKey;

#[tokio::main]
async fn main() {
    println!("Test: WebSocket multiple peers");
    println!("Description: Connect to multiple WebSocket peers");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    println!("\n--- Test: WebSocket multiple peers ---");
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let options = GunOptions {
        peers: vec![
            "ws://localhost:8765/gun".to_string(),
            "ws://localhost:8766/gun".to_string(),
        ],
        ..Default::default()
    };
    
    match Gun::with_options(secret_key, public_key, options).await {
        Ok(_) => {
            println!("✓ WebSocket multiple peers: Instance created");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ WebSocket multiple peers: Failed - {}", e);
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

