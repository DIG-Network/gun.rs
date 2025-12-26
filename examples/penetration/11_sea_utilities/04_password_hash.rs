/// Test: SEA.hash_password() - Password hashing
/// 
/// Tests password hashing.

use gun::sea::{hash_password, generate_salt};

#[tokio::main]
async fn main() {
    println!("Test: SEA.hash_password() - Password hashing");
    println!("Description: Test password hashing");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    println!("\n--- Test: Password hashing ---");
    let password = "test_password";
    let salt = generate_salt();
    
    let hash = hash_password(password, &salt);
    println!("  - Hash: {}...", &hash[..20.min(hash.len())]);
    
    if !hash.is_empty() {
        println!("✓ Password hashing: Success");
        success_count += 1;
    } else {
        println!("✗ Password hashing: Failed (empty hash)");
        fail_count += 1;
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

