/// Test: Gun::with_options() with empty/default options
/// 
/// Tests creating a Gun instance with empty GunOptions (all defaults).
/// Should behave similarly to Gun::new().

use gun::{Gun, GunOptions};
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: Gun::with_options() with empty options");
    println!("Description: Create Gun instance with default GunOptions");
    
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let options = GunOptions::default();
    
    match Gun::with_options(secret_key, public_key, options).await {
        Ok(gun) => {
            println!("✓ Success: Gun instance created with empty options");
            println!("  - Instance type: Gun");
            println!("  - Options: All defaults");
            std::process::exit(0);
        }
        Err(e) => {
            println!("✗ Error: Failed to create Gun instance");
            println!("  Error: {}", e);
            std::process::exit(1);
        }
    }
}

