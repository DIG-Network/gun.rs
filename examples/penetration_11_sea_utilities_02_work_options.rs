/// Test: SEA.work() - All work() options
/// 
/// Tests work() with different options.

use gun::sea::{work, WorkOptions};

#[tokio::main]
async fn main() {
    println!("Test: SEA.work() - All work() options");
    println!("Description: Test work() with different options");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    let data = b"test data";
    
    // Test 1: Different iterations
    println!("\n--- Test 1: Different iterations ---");
    let options1 = WorkOptions {
        name: Some("PBKDF2".to_string()),
        iterations: Some(100),
        salt: Some(b"salt1".to_vec()),
        hash: Some("SHA-256".to_string()),
        length: Some(32),
        encode: Some("base64".to_string()),
    };
    if test_work(data, Some(b"salt1".to_vec()), options1, "iterations 100").await {
        success_count += 1;
    } else {
        fail_count += 1;
    }
    
    // Test 2: Different hash
    println!("\n--- Test 2: Different hash ---");
    let options2 = WorkOptions {
        name: Some("PBKDF2".to_string()),
        iterations: Some(1000),
        salt: Some(b"salt2".to_vec()),
        hash: Some("SHA-256".to_string()),
        length: Some(32),
        encode: Some("base64".to_string()),
    };
    if test_work(data, Some(b"salt2".to_vec()), options2, "SHA-256").await {
        success_count += 1;
    } else {
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

async fn test_work(data: &[u8], salt: Option<Vec<u8>>, options: WorkOptions, name: &str) -> bool {
    match work(data, salt, options).await {
        Ok(_) => {
            println!("✓ Work {}: Success", name);
            true
        }
        Err(e) => {
            println!("✗ Work {}: Failed - {}", name, e);
            false
        }
    }
}

