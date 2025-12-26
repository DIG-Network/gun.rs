/// Test: Chain.on() then Chain.off() - Subscribe then unsubscribe
/// 
/// Tests subscribing and then unsubscribing.

use gun::Gun;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::time::Duration;

#[tokio::main]
async fn main() {
    println!("Test: Chain.on() then Chain.off() - Subscribe then unsubscribe");
    println!("Description: Subscribe then unsubscribe");
    
    let gun = Gun::new();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: Subscribe, receive updates, then unsubscribe
    println!("\n--- Test: Subscribe then unsubscribe ---");
    let update_count = Arc::new(AtomicU32::new(0));
    let count_clone = update_count.clone();
    
    let chain = gun.get("test").get("onoff");
    chain.on(move |data, _key| {
        if data.is_i64() {
            count_clone.fetch_add(1, Ordering::Relaxed);
        }
    });
    
    // Put data - should trigger subscription
    match chain.put(json!(1)).await {
        Ok(_) => {}
        Err(e) => {
            println!("✗ Put failed: {}", e);
            fail_count += 1;
        }
    }
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let count_before = update_count.load(Ordering::Relaxed);
    println!("  Updates before unsubscribe: {}", count_before);
    
    // Unsubscribe
    chain.off();
    
    // Put more data - should NOT trigger subscription
    match chain.put(json!(2)).await {
        Ok(_) => {}
        Err(e) => {
            println!("✗ Put failed: {}", e);
            fail_count += 1;
        }
    }
    
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let count_after = update_count.load(Ordering::Relaxed);
    println!("  Updates after unsubscribe: {}", count_after);
    
    if count_before > 0 && count_after == count_before {
        println!("✓ Subscribe/unsubscribe: Working correctly");
        success_count += 1;
    } else {
        println!("✗ Subscribe/unsubscribe: Not working correctly");
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

