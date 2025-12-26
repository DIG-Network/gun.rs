/// Test: Chain method chaining - get().back().get()
/// 
/// Tests chaining get(), back(), and get() methods.

use gun::Gun;
use chia_bls::SecretKey;

#[tokio::main]
async fn main() {
    println!("Test: Chain method chaining - get().back().get()");
    println!("Description: Test get().back().get() chaining");
    
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Gun::new(secret_key, public_key);
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: get().back().get()
    println!("\n--- Test: get().back().get() ---");
    let chain = gun.get("level1").get("level2").get("level3");
    
    match chain.back(Some(1)) {
        Some(back_chain) => {
            let _next_chain = back_chain.get("new_key");
            println!("✓ get().back().get(): Success");
            success_count += 1;
        }
        None => {
            println!("✗ get().back().get(): Back returned None");
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

