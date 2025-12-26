/// Test: SEA.create_user() - Create user with/without alias
/// 
/// Tests creating users with and without aliases.

use gun::{Gun, sea::create_user};

#[tokio::main]
async fn main() {
    println!("Test: SEA.create_user() - Create user with/without alias");
    println!("Description: Create users with and without aliases");
    
    let gun = Gun::new();
    let chain = gun.root();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test 1: Create user with alias
    println!("\n--- Test 1: Create user with alias ---");
    match create_user(chain.clone(), Some("testuser".to_string()), "password123").await {
        Ok(user) => {
            println!("✓ Create user with alias: Success");
            println!("  - Pub key: {}...", &user.pair.pub_key[..20.min(user.pair.pub_key.len())]);
            if let Some(ref alias) = user.alias {
                println!("  - Alias: {}", alias);
            }
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Create user with alias: Failed - {}", e);
            fail_count += 1;
        }
    }
    
    // Test 2: Create user without alias
    println!("\n--- Test 2: Create user without alias ---");
    match create_user(chain.clone(), None, "password456").await {
        Ok(user) => {
            println!("✓ Create user without alias: Success");
            println!("  - Pub key: {}...", &user.pair.pub_key[..20.min(user.pair.pub_key.len())]);
            if user.alias.is_none() {
                println!("  - Alias: None (as expected)");
            }
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Create user without alias: Failed - {}", e);
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

