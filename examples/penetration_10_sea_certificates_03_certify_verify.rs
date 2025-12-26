/// Test: SEA.certify() and SEA.verify_certificate() - Create and verify
/// 
/// Tests creating certificates and verifying them.

use gun::sea::{pair, certify, verify_certificate, Certificants, Policy, CertifyOptions};

#[tokio::main]
async fn main() {
    println!("Test: SEA.certify() and SEA.verify_certificate() - Create and verify");
    println!("Description: Create certificate and verify it");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    match pair().await {
        Ok(authority) => {
            match pair().await {
                Ok(holder) => {
                    println!("\n--- Test: Create and verify certificate ---");
                    match certify(
                        Certificants::List(vec![holder.pub_key.clone()]),
                        Policy::String("test.path".to_string()),
                        &authority,
                        CertifyOptions::default()
                    ).await {
                        Ok(cert) => {
                            match verify_certificate(&cert, &authority.pub_key).await {
                                Ok(verified) => {
                                    println!("✓ Certify and verify: Success");
                                    println!("  - Certificants: {:?}", verified.certificants);
                                    success_count += 1;
                                }
                                Err(e) => {
                                    println!("✗ Certify and verify: Verification failed - {}", e);
                                    fail_count += 1;
                                }
                            }
                        }
                        Err(e) => {
                            println!("✗ Certify and verify: Certify failed - {}", e);
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

