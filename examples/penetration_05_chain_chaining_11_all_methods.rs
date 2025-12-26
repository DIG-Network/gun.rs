/// Test: Chain method chaining - Every method in one chain
/// 
/// Tests chaining all methods together in one chain.

use gun::Gun;
use chia_bls::SecretKey;
use serde_json::json;
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain method chaining - Every method in one chain");
    println!("Description: Chain all methods together");
    
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Gun::new(secret_key, public_key);
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: Chain all methods
    println!("\n--- Test: Chain all methods ---");
    let chain = gun.get("test").get("all_methods");
    
    // get().put()
    match chain.put(json!({"value": 1})).await {
        Ok(_) => {}
        Err(e) => {
            println!("✗ put() failed: {}", e);
            fail_count += 1;
        }
    }
    
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    // get().once()
    let mut once_worked = false;
    match chain.once(|data, _key| {
        if data.is_object() {
            once_worked = true;
        }
    }).await {
        Ok(_) => {
            if once_worked {
                println!("✓ once() worked");
            }
        }
        Err(e) => {
            println!("✗ once() failed: {}", e);
            fail_count += 1;
        }
    }
    
    // get().on()
    let update_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let count_clone = update_count.clone();
    chain.on(move |data, _key| {
        if data.is_object() {
            count_clone.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    });
    
    // get().map()
    let props = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let props_clone = props.clone();
    chain.map(move |value, key| {
        props_clone.lock().unwrap().push((key, value.clone()));
    });
    
    // get().set()
    match chain.set(json!({"id": 1})).await {
        Ok(_) => {}
        Err(e) => {
            println!("✗ set() failed: {}", e);
            fail_count += 1;
        }
    }
    
    // get().back()
    match chain.back(Some(1)) {
        Some(_) => println!("✓ back() worked"),
        None => println!("? back() returned None"),
    }
    
    // get().off()
    chain.off();
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    if fail_count == 0 {
        println!("✓ All methods chained: Success");
        success_count += 1;
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

