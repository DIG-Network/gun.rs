/// Test: SEA.work() - PBKDF2/SHA hashing
/// 
/// Tests PBKDF2/SHA hashing with work().

use gun::sea::{work, WorkOptions};

#[tokio::main]
async fn main() {
    println!("Test: SEA.work() - PBKDF2/SHA hashing");
    println!("Description: Test PBKDF2/SHA hashing");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    println!("\n--- Test: PBKDF2/SHA hashing ---");
    let data = b"test data";
    let options = WorkOptions {
        name: Some("PBKDF2".to_string()),
        iterations: Some(1000),
        salt: Some(b"salt".to_vec()),
        hash: Some("SHA-256".to_string()),
        length: Some(32),
        encode: Some("base64".to_string()),
    };
    
    match work(data, Some(b"salt".to_vec()), options).await {
        Ok(result) => {
            println!("✓ Work: Success");
            println!("  - Result: {}...", &result[..20.min(result.len())]);
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Work: Failed - {}", e);
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

