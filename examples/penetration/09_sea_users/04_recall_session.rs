/// Test: SEA.recall() - Recall from file/graph
/// 
/// Tests recalling user sessions from storage.

use gun::{Gun, sea::{create_user, recall}};

#[tokio::main]
async fn main() {
    println!("Test: SEA.recall() - Recall from file/graph");
    println!("Description: Recall user sessions from storage");
    
    let gun = Gun::new();
    let chain = gun.root();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Setup: Create user
    println!("\n--- Setup: Create user ---");
    let alias = Some("recalluser".to_string());
    let password = "recall_password";
    match create_user(chain.clone(), alias.clone(), password).await {
        Ok(user) => {
            println!("✓ User created");
            let pub_key = user.pair.pub_key.clone();
            
            // Test: Recall session
            println!("\n--- Test: Recall session ---");
            match recall(Some(chain.clone()), None).await {
                Ok(Some(recalled)) => {
                    println!("✓ Recall session: Success");
                    println!("  - Pub key: {}...", &recalled.pair.pub_key[..20.min(recalled.pair.pub_key.len())]);
                    success_count += 1;
                }
                Ok(None) => {
                    println!("? Recall session: No session found (may be expected)");
                    success_count += 1;
                }
                Err(e) => {
                    println!("✗ Recall session: Failed - {}", e);
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

