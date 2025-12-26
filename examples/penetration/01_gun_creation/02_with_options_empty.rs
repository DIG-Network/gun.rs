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
    
    let options = GunOptions::default();
    
    // Generate BLS key pair
    let secret_key1 = SecretKey::from_seed(&[1 u8; 32]);
    let public_key1 = secret_key1.public_key();
    match Gun::with_options(secret_key1, public_key1, options).await {
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

