/// Test: SEA.certify() - Certificates with expiry
/// 
/// Tests creating certificates with expiry.

use gun::sea::{pair, certify, Certificants, Policy, CertifyOptions};
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() {
    println!("Test: SEA.certify() - Certificates with expiry");
    println!("Description: Create certificates with expiry");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    match pair().await {
        Ok(authority) => {
            match pair().await {
                Ok(holder) => {
                    println!("\n--- Test: Create certificate with expiry ---");
                    let expiry = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as f64 + 3600.0; // 1 hour from now
                    
                    match certify(
                        Certificants::List(vec![holder.pub_key.clone()]),
                        Policy::String("expiry.path".to_string()),
                        &authority,
                        CertifyOptions {
                            expiry: Some(expiry),
                            ..Default::default()
                        }
                    ).await {
                        Ok(cert) => {
                            println!("✓ Certificate with expiry: Success");
                            println!("  - Certificate: {}...", &cert[..30.min(cert.len())]);
                            success_count += 1;
                        }
                        Err(e) => {
                            println!("✗ Certificate with expiry: Failed - {}", e);
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

