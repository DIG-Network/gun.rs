/// Test: Gun::with_options() with relay server mode
/// 
/// Tests creating Gun instances in relay server (super peer) mode.

use gun::{Gun, GunOptions};
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: Gun::with_options() with relay server mode");
    println!("Description: Test super peer mode with port");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test 1: Super peer mode with port
    println!("\n--- Test 1: Super peer with port ---");
    let options = GunOptions {
        super_peer: true,
        port: Some(8765),
        ..Default::default()
    };
    // Generate BLS key pair
    let secret_key1 = SecretKey::from_seed(&[1 u8; 32]);
    let public_key1 = secret_key1.public_key();
    match Gun::with_options(secret_key1, public_key1, options).await {
        Ok(_) => {
            println!("✓ Super peer with port: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Super peer with port: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 2: Super peer mode without port
    println!("\n--- Test 2: Super peer without port ---");
    let options = GunOptions {
        super_peer: true,
        port: None,
        ..Default::default()
    };
    // Generate BLS key pair
    let secret_key2 = SecretKey::from_seed(&[2 u8; 32]);
    let public_key2 = secret_key2.public_key();
    match Gun::with_options(secret_key2, public_key2, options).await {
        Ok(_) => {
            println!("✓ Super peer without port: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Super peer without port: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 3: Not super peer (normal mode)
    println!("\n--- Test 3: Normal mode (not super peer) ---");
    let options = GunOptions {
        super_peer: false,
        ..Default::default()
    };
    // Generate BLS key pair
    let secret_key3 = SecretKey::from_seed(&[3 u8; 32]);
    let public_key3 = secret_key3.public_key();
    match Gun::with_options(secret_key3, public_key3, options).await {
        Ok(_) => {
            println!("✓ Normal mode: Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Normal mode: Failed - {}", e);
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

