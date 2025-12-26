/// Test: Chain.back() - Navigate back one level
/// 
/// Tests navigating back one level in the chain.

use gun::Gun;
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: Chain.back() - Navigate back one level");
    println!("Description: Navigate back one level");
    
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Gun::new(secret_key, public_key);
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: Navigate back one level
    println!("\n--- Test: Navigate back one level ---");
    let chain = gun.get("level1").get("level2").get("level3");
    
    match chain.back(Some(1)) {
        Some(_back_chain) => {
            println!("✓ Back one level: Success");
            success_count += 1;
        }
        None => {
            println!("✗ Back one level: Returned None");
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

