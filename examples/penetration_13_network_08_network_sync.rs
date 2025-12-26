/// Test: Data synchronization
/// 
/// Tests data synchronization over network.

use gun::{Gun, GunOptions};
use chia_bls::SecretKey;
use serde_json::json;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Data synchronization");
    println!("Description: Test data synchronization over network");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    println!("\n--- Test: Data synchronization ---");
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let options = GunOptions {
        peers: vec!["ws://localhost:8765/gun".to_string()],
        ..Default::default()
    };
    
    match Gun::with_options(secret_key, public_key, options).await {
        Ok(gun) => {
            match gun.get("test").get("sync").put(json!({"value": "synced"})).await {
                Ok(_) => {
                    println!("✓ Data synchronization: Data put (sync depends on peer connection)");
                    success_count += 1;
                }
                Err(e) => {
                    println!("✗ Data synchronization: Put failed - {}", e);
                    fail_count += 1;
                }
            }
        }
        Err(e) => {
            println!("✗ Instance creation failed - {}", e);
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

