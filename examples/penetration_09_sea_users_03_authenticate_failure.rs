/// Test: SEA.authenticate() - Wrong password, non-existent user
/// 
/// Tests authentication failures.

use gun::{Gun, sea::{create_user, authenticate}};
use chia_bls::SecretKey;

#[tokio::main]
async fn main() {
    println!("Test: SEA.authenticate() - Wrong password, non-existent user");
    println!("Description: Test authentication failures");
    
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Gun::new(secret_key, public_key);
    let chain = gun.root();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Setup: Create user
    println!("\n--- Setup: Create user ---");
    match create_user(chain.clone(), Some("failuser".to_string()), "correct_password").await {
        Ok(_) => println!("✓ User created"),
        Err(e) => {
            println!("✗ User creation failed: {}", e);
            fail_count += 1;
        }
    }
    
    // Test 1: Wrong password
    println!("\n--- Test 1: Wrong password ---");
    match authenticate(chain.clone(), "failuser", "wrong_password").await {
        Ok(_) => {
            println!("✗ Wrong password: Authentication succeeded (should have failed)");
            fail_count += 1;
        }
        Err(_) => {
            println!("✓ Wrong password: Authentication correctly failed");
            success_count += 1;
        }
    }
    
    // Test 2: Non-existent user
    println!("\n--- Test 2: Non-existent user ---");
    match authenticate(chain.clone(), "nonexistent", "password").await {
        Ok(_) => {
            println!("✗ Non-existent user: Authentication succeeded (should have failed)");
            fail_count += 1;
        }
        Err(_) => {
            println!("✓ Non-existent user: Authentication correctly failed");
            success_count += 1;
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

