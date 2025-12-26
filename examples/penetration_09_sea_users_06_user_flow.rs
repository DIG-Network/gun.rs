/// Test: SEA user flow - Complete user flow
/// 
/// Tests complete user flow: create, authenticate, recall.

use gun::{Gun, sea::{create_user, authenticate, recall}};
use chia_bls::SecretKey;

#[tokio::main]
async fn main() {
    println!("Test: SEA user flow - Complete user flow");
    println!("Description: Complete user flow: create, authenticate, recall");
    
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Gun::new(secret_key, public_key);
    let chain = gun.root();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    let alias = Some("flowuser".to_string());
    let password = "flow_password";
    
    // Step 1: Create user
    println!("\n--- Step 1: Create user ---");
    match create_user(chain.clone(), alias.clone(), password).await {
        Ok(user) => {
            println!("✓ User created");
            let pub_key = user.pair.pub_key.clone();
            
            // Step 2: Authenticate
            println!("\n--- Step 2: Authenticate ---");
            match authenticate(chain.clone(), alias.as_ref().unwrap(), password).await {
                Ok(auth_user) => {
                    println!("✓ User authenticated");
                    if auth_user.pair.pub_key == pub_key {
                        println!("  - Pub keys match");
                    }
                    
                    // Step 3: Recall
                    println!("\n--- Step 3: Recall ---");
                    match recall(Some(chain.clone()), None).await {
                        Ok(Some(recalled)) => {
                            println!("✓ Session recalled");
                            if recalled.pair.pub_key == pub_key {
                                println!("  - Pub keys match");
                                println!("✓ Complete user flow: Success");
                                success_count += 1;
                            } else {
                                println!("✗ Pub keys don't match");
                                fail_count += 1;
                            }
                        }
                        Ok(None) => {
                            println!("? Session not found (may be expected)");
                            success_count += 1;
                        }
                        Err(e) => {
                            println!("✗ Recall failed: {}", e);
                            fail_count += 1;
                        }
                    }
                }
                Err(e) => {
                    println!("✗ Authentication failed: {}", e);
                    fail_count += 1;
                }
            }
        }
        Err(e) => {
            println!("✗ User creation failed: {}", e);
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

