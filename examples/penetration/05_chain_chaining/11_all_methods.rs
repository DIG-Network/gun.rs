/// Test: Chain method chaining - Every method in one chain (two clients)
/// 
/// Tests chaining all methods together in one chain.

use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use tokio::time::Duration;
use chia_bls::{SecretKey, PublicKey};

#[tokio::main]
async fn main() {
    println!("Test: Chain method chaining - Every method in one chain (two clients)");
    println!("Description: Chain all methods together");
    
    const RELAY_URL: &str = "http://dig-relay-prod.eba-2cmanxbe.us-east-1.elasticbeanstalk.com/gun";
    
    // Create Client 1
    println!("\n--- Setup: Creating Client 1 ---");
    let options1 = GunOptions {
        peers: vec![RELAY_URL.to_string()],
        ..Default::default()
    };
    // Generate BLS key pair
    let secret_key1 = SecretKey::from_seed(&[1 u8; 32]);
    let public_key1 = secret_key1.public_key();
    let client1 = match Gun::with_options(secret_key1, public_key1, options1).await {
        Ok(g) => Arc::new(g),
        Err(e) => {
            println!("✗ Failed to create Client 1: {}", e);
            std::process::exit(1);
        }
    };
    tokio::time::sleep(Duration::from_millis(1000)).await;
    
    // Create Client 2
    println!("--- Setup: Creating Client 2 ---");
    let options2 = GunOptions {
        peers: vec![RELAY_URL.to_string()],
        ..Default::default()
    };
    // Generate BLS key pair
    let secret_key2 = SecretKey::from_seed(&[2 u8; 32]);
    let public_key2 = secret_key2.public_key();
    let client2 = match Gun::with_options(secret_key2, public_key2, options2).await {
        Ok(g) => Arc::new(g),
        Err(e) => {
            println!("✗ Failed to create Client 2: {}", e);
            std::process::exit(1);
        }
    };
    tokio::time::sleep(Duration::from_millis(1000)).await;
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let test_key = format!("all_methods_{}", timestamp);
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: Chain all methods
    println!("\n--- Test: Chain all methods ---");
    let chain1 = client1.get("test").get(&test_key);
    let chain2 = client2.get("test").get(&test_key);
    
    // get().put() - Client1
    match chain1.put(json!({"value": 1})).await {
        Ok(_) => {}
        Err(e) => {
            println!("✗ Client1: put() failed: {}", e);
            fail_count += 1;
        }
    }
    
    tokio::time::sleep(Duration::from_millis(2000)).await;
    
    // get().once() - Client2
    let once_worked = Arc::new(AtomicBool::new(false));
    let once_worked_clone = once_worked.clone();
    match chain2.once(move |data, _key| {
        if data.is_object() {
            once_worked_clone.store(true, Ordering::Relaxed);
        }
    }).await {
        Ok(_) => {
            if once_worked.load(Ordering::Relaxed) {
                println!("✓ Client2: once() worked");
            }
        }
        Err(e) => {
            println!("✗ Client2: once() failed: {}", e);
            fail_count += 1;
        }
    }
    
    // get().on() - Client2
    let update_count = Arc::new(AtomicU32::new(0));
    let count_clone = update_count.clone();
    chain2.on(move |data, _key| {
        if data.is_object() {
            count_clone.fetch_add(1, Ordering::Relaxed);
        }
    });
    
    // get().map() - Client2
    let props = Arc::new(std::sync::Mutex::new(Vec::new()));
    let props_clone = props.clone();
    chain2.map(move |value, key| {
        props_clone.lock().unwrap().push((key, value.clone()));
    });
    
    tokio::time::sleep(Duration::from_millis(1000)).await;
    
    // get().set() - Client1
    match chain1.set(json!({"id": 1})).await {
        Ok(_) => {}
        Err(e) => {
            println!("✗ Client1: set() failed: {}", e);
            fail_count += 1;
        }
    }
    
    // get().back() - both clients
    match chain1.back(Some(1)) {
        Some(_) => println!("✓ Client1: back() worked"),
        None => println!("? Client1: back() returned None"),
    }
    
    // get().off() - Client2
    chain2.off();
    
    tokio::time::sleep(Duration::from_millis(1000)).await;
    
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
