/// Test: SEA.certify() - String and Radix policies
/// 
/// Tests creating certificates with different policies.

use gun::sea::{pair, certify, Certificants, Policy, CertifyOptions};

#[tokio::main]
async fn main() {
    println!("Test: SEA.certify() - String and Radix policies");
    println!("Description: Create certificates with different policies");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    match pair().await {
        Ok(authority) => {
            match pair().await {
                Ok(holder) => {
                    // Test: String policy
                    println!("\n--- Test 1: String policy ---");
                    match certify(
                        Certificants::List(vec![holder.pub_key.clone()]),
                        Policy::String("string.policy".to_string()),
                        &authority,
                        CertifyOptions::default()
                    ).await {
                        Ok(_) => {
                            println!("✓ String policy: Success");
                            success_count += 1;
                        }
                        Err(e) => {
                            println!("✗ String policy: Failed - {}", e);
                            fail_count += 1;
                        }
                    }
                    
                    // Test: Radix policy (wildcard)
                    println!("\n--- Test 2: Radix policy (wildcard) ---");
                    match certify(
                        Certificants::Wildcard,
                        Policy::String("*".to_string()),
                        &authority,
                        CertifyOptions::default()
                    ).await {
                        Ok(_) => {
                            println!("✓ Radix policy: Success");
                            success_count += 1;
                        }
                        Err(e) => {
                            println!("✗ Radix policy: Failed - {}", e);
                            fail_count += 1;
                        }
                    }
                }
                Err(e) => {
                    println!("✗ Holder pair generation failed - {}", e);
                    fail_count += 1;
                }
            }
        }
        Err(e) => {
            println!("✗ Authority pair generation failed - {}", e);
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

