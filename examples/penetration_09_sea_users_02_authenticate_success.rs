/// Test: SEA.authenticate() - Successful authentication
/// 
/// Tests successful user authentication.

use gun::{Gun, sea::{create_user, authenticate}};
use chia_bls::SecretKey;

#[tokio::main]
async fn main() {
    println!("Test: SEA.authenticate() - Successful authentication");
    println!("Description: Test successful user authentication");
    
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Gun::new(secret_key, public_key);
    let chain = gun.root();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Setup: Create user
    println!("\n--- Setup: Create user ---");
    let password = "secure_password";
    match create_user(chain.clone(), Some("authuser".to_string()), password).await {
        Ok(_) => println!("✓ User created"),
        Err(e) => {
            println!("✗ User creation failed: {}", e);
            fail_count += 1;
        }
    }
    
    // Test: Authenticate with correct password
    println!("\n--- Test: Authenticate with correct password ---");
    match authenticate(chain.clone(), "authuser", password).await {
        Ok(user) => {
            println!("✓ Authentication: Success");
            println!("  - Pub key: {}...", &user.pair.pub_key[..20.min(user.pair.pub_key.len())]);
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Authentication: Failed - {}", e);
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

