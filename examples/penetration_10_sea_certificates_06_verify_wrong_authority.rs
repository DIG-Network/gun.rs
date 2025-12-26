/// Test: SEA.verify_certificate() - Verify with wrong authority
/// 
/// Tests verifying certificates with wrong authority (should fail).

use gun::sea::{pair, certify, verify_certificate, Certificants, Policy, CertifyOptions};

#[tokio::main]
async fn main() {
    println!("Test: SEA.verify_certificate() - Verify with wrong authority");
    println!("Description: Verify certificate with wrong authority (should fail)");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    match pair().await {
        Ok(authority1) => {
            match pair().await {
                Ok(authority2) => {
                    match pair().await {
                        Ok(holder) => {
                            println!("\n--- Test: Verify with wrong authority ---");
                            match certify(
                                Certificants::List(vec![holder.pub_key.clone()]),
                                Policy::String("test.path".to_string()),
                                &authority1,
                                CertifyOptions::default()
                            ).await {
                                Ok(cert) => {
                                    // Try to verify with wrong authority
                                    match verify_certificate(&cert, &authority2.pub_key).await {
                                        Ok(_) => {
                                            println!("✗ Wrong authority: Verification succeeded (should have failed)");
                                            fail_count += 1;
                                        }
                                        Err(_) => {
                                            println!("✓ Wrong authority: Verification correctly failed");
                                            success_count += 1;
                                        }
                                    }
                                }
                                Err(e) => {
                                    println!("✗ Certify failed - {}", e);
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
                    println!("✗ Authority2 pair generation failed - {}", e);
                    fail_count += 1;
                }
            }
        }
        Err(e) => {
            println!("✗ Authority1 pair generation failed - {}", e);
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

