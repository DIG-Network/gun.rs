/// Test: SEA.certify() - Create wildcard certificate
/// 
/// Tests creating wildcard certificates.

use gun::sea::{pair, certify, Certificants, Policy, CertifyOptions};

#[tokio::main]
async fn main() {
    println!("Test: SEA.certify() - Create wildcard certificate");
    println!("Description: Create wildcard certificate");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Generate authority key pair
    match pair().await {
        Ok(authority) => {
            // Generate certificate holder key pair
            match pair().await {
                Ok(holder) => {
                    // Create wildcard certificate
                    println!("\n--- Test: Create wildcard certificate ---");
                    match certify(
                        Certificants::Wildcard,
                        Policy::String("*".to_string()),
                        &authority,
                        CertifyOptions::default()
                    ).await {
                        Ok(cert) => {
                            println!("✓ Wildcard certificate: Success");
                            println!("  - Certificate: {}...", &cert[..30.min(cert.len())]);
                            success_count += 1;
                        }
                        Err(e) => {
                            println!("✗ Wildcard certificate: Failed - {}", e);
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

