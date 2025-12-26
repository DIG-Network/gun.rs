/// Test: Super peer mode
/// 
/// Tests super peer mode.

use gun::{Gun, GunOptions};
use chia_bls::SecretKey;

#[tokio::main]
async fn main() {
    println!("Test: Super peer mode");
    println!("Description: Test super peer mode");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    println!("\n--- Test: Super peer mode ---");
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let options = GunOptions {
        super_peer: true,
        port: Some(8765),
        ..Default::default()
    };
    
    match Gun::with_options(secret_key, public_key, options).await {
        Ok(_) => {
            println!("✓ Super peer mode: Instance created");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Super peer mode: Failed - {}", e);
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

