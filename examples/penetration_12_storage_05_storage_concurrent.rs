/// Test: Concurrent storage operations
/// 
/// Tests concurrent storage operations.

use gun::{Gun, GunOptions};
use chia_bls::SecretKey;
use tempfile::TempDir;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    println!("Test: Concurrent storage operations");
    println!("Description: Test concurrent storage operations");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    println!("\n--- Test: Concurrent storage operations ---");
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let options = GunOptions {
        radisk: true,
        storage_path: Some(temp_dir.path().to_str().unwrap().to_string()),
        ..Default::default()
    };
    
    match Gun::with_options(secret_key, public_key, options).await {
        Ok(gun) => {
            let gun_arc = Arc::new(gun);
            let mut handles = vec![];
            
            for i in 0..10 {
                let gun_clone = gun_arc.clone();
                let handle = tokio::spawn(async move {
                    gun_clone.get("test").get(&format!("key{}", i)).put(serde_json::json!(i)).await
                });
                handles.push(handle);
            }
            
            let mut concurrent_success = 0;
            for handle in handles {
                if handle.await.is_ok() {
                    concurrent_success += 1;
                }
            }
            
            if concurrent_success == 10 {
                println!("✓ Concurrent storage: All 10 succeeded");
                success_count += 1;
            } else {
                println!("✗ Concurrent storage: Only {}/10 succeeded", concurrent_success);
                fail_count += 1;
            }
        }
        Err(e) => {
            println!("✗ Storage creation failed - {}", e);
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

