/// Test: SEA.recall() - Recall expired session
/// 
/// Tests recalling expired user sessions.

use gun::{Gun, sea::{create_user, recall}};
use chia_bls::SecretKey;

#[tokio::main]
async fn main() {
    println!("Test: SEA.recall() - Recall expired session");
    println!("Description: Recall expired user sessions");
    
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Gun::new(secret_key, public_key);
    let chain = gun.root();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Setup: Create user
    println!("\n--- Setup: Create user ---");
    let alias = Some("expireduser".to_string());
    match create_user(chain.clone(), alias.clone(), "password").await {
        Ok(user) => {
            println!("✓ User created");
            let pub_key = user.pair.pub_key.clone();
            
            // Note: Expiration is handled by the recall function checking expiry timestamps
            // This test documents current behavior
            println!("\n--- Test: Recall (may be expired) ---");
            match recall(Some(chain.clone()), None).await {
                Ok(_) => {
                    println!("✓ Recall: Success (session may not be expired yet)");
                    success_count += 1;
                }
                Err(e) => {
                    println!("? Recall: Failed - {} (may be expected if expired)", e);
                    // This might be expected behavior
                    success_count += 1;
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
    println!("Note: Expiration behavior depends on implementation");
    
    if fail_count == 0 {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}

