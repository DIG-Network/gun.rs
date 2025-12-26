/// Test: Gun::new() - Default instance creation with two clients
/// 
/// Tests creating two Gun instances with default settings (no options).
/// This should always succeed and create local-only instances.

use gun::{Gun, GunOptions};
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: Gun::new() - Default instance creation (two clients)");
    println!("Description: Create two Gun instances with default settings");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test 1: Create Client 1
    println!("\n--- Test 1: Creating Client 1 ---");
    let secret_key1 = SecretKey::from_seed(&[1u8; 32]);
    let public_key1 = secret_key1.public_key();
    match Gun::with_options(secret_key1, public_key1, GunOptions::default()).await {
        Ok(_client1) => {
            println!("✓ Client 1: Gun instance created");
            println!("  - Instance type: Gun");
            println!("  - Storage: None (in-memory only)");
            println!("  - Peers: None");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Client 1: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 2: Create Client 2
    println!("\n--- Test 2: Creating Client 2 ---");
    let secret_key2 = SecretKey::from_seed(&[2u8; 32]);
    let public_key2 = secret_key2.public_key();
    match Gun::with_options(secret_key2, public_key2, GunOptions::default()).await {
        Ok(_client2) => {
            println!("✓ Client 2: Gun instance created");
            println!("  - Instance type: Gun");
            println!("  - Storage: None (in-memory only)");
            println!("  - Peers: None");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Client 2: Failed - {}", e);
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
