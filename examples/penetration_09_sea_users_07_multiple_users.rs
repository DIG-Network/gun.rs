/// Test: SEA - Multiple users simultaneously
/// 
/// Tests creating and using multiple users simultaneously.

use gun::{Gun, sea::create_user};
use chia_bls::SecretKey;

#[tokio::main]
async fn main() {
    println!("Test: SEA - Multiple users simultaneously");
    println!("Description: Create and use multiple users simultaneously");
    
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Gun::new(secret_key, public_key);
    let chain = gun.root();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: Create multiple users
    println!("\n--- Test: Create multiple users ---");
    let mut handles = vec![];
    for i in 0..5 {
        let chain_clone = chain.clone();
        let alias = format!("user{}", i);
        let password = format!("password{}", i);
        let handle = tokio::spawn(async move {
            create_user(chain_clone, Some(alias.clone()), &password).await
        });
        handles.push(handle);
    }
    
    let mut created = 0;
    for handle in handles {
        match handle.await {
            Ok(Ok(_)) => created += 1,
            Ok(Err(e)) => {
                println!("✗ User creation failed: {}", e);
                fail_count += 1;
            }
            Err(e) => {
                println!("✗ Join error: {:?}", e);
                fail_count += 1;
            }
        }
    }
    
    if created == 5 {
        println!("✓ Multiple users: All 5 created");
        success_count += 1;
    } else {
        println!("✗ Multiple users: Only {}/5 created", created);
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

