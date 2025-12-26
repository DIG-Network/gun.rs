/// Test: SEA.verify_password() - Password verification
/// 
/// Tests password verification.

use gun::sea::{hash_password, verify_password, generate_salt};

#[tokio::main]
async fn main() {
    println!("Test: SEA.verify_password() - Password verification");
    println!("Description: Test password verification");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    let password = "test_password";
    let salt = generate_salt();
    let hash = hash_password(password, &salt);
    
    // Test 1: Correct password
    println!("\n--- Test 1: Correct password ---");
    if verify_password(password, &salt, &hash) {
        println!("✓ Correct password: Success");
        success_count += 1;
    } else {
        println!("✗ Correct password: Failed");
        fail_count += 1;
    }
    
    // Test 2: Wrong password
    println!("\n--- Test 2: Wrong password ---");
    if !verify_password("wrong_password", &salt, &hash) {
        println!("✓ Wrong password: Correctly rejected");
        success_count += 1;
    } else {
        println!("✗ Wrong password: Incorrectly accepted");
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

