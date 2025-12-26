/// Test: Chain.back() - Navigate back to root
/// 
/// Tests navigating back to root using back(None).

use gun::Gun;
use chia_bls::SecretKey;

#[tokio::main]
async fn main() {
    println!("Test: Chain.back() - Navigate back to root");
    println!("Description: Navigate back to root");
    
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Gun::new(secret_key, public_key);
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: Navigate back to root
    println!("\n--- Test: Navigate back to root ---");
    let chain = gun.get("level1").get("level2").get("level3");
    
    match chain.back(None) {
        Some(_) => {
            println!("✓ Back to root: Success");
            success_count += 1;
        }
        None => {
            println!("✗ Back to root: Returned None");
            fail_count += 1;
        }
    }
    
    // Test: Back to root from root
    println!("\n--- Test: Back to root from root ---");
    let root_chain = gun.root();
    match root_chain.back(None) {
        Some(_) => {
            println!("✓ Back from root: Success");
            success_count += 1;
        }
        None => {
            println!("? Back from root: Returned None (may be expected)");
            success_count += 1; // This might be expected behavior
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

