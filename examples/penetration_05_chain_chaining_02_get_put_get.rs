/// Test: Chain method chaining - get().put().get()
/// 
/// Tests chaining get(), put(), and get() methods.

use gun::Gun;
use chia_bls::SecretKey;
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Test: Chain method chaining - get().put().get()");
    println!("Description: Test get().put().get() chaining");
    
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Gun::new(secret_key, public_key);
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: get().put().get()
    println!("\n--- Test: get().put().get() ---");
    match gun.get("test").get("chained2").put(json!({"value": 100})).await {
        Ok(chain) => {
            // Chain continues with get()
            let _next_chain = chain.get("value");
            println!("✓ get().put().get(): Success");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ get().put().get(): Failed - {}", e);
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

