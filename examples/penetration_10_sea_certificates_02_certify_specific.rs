/// Test: SEA.certify() - Create specific certificate
/// 
/// Tests creating specific certificates.

use gun::sea::{pair, certify, Certificants, Policy, CertifyOptions};

#[tokio::main]
async fn main() {
    println!("Test: SEA.certify() - Create specific certificate");
    println!("Description: Create specific certificate");
    
    let mut success_count = 0;
    let mut fail_count = 0;
    
    match pair().await {
        Ok(authority) => {
            match pair().await {
                Ok(holder) => {
                    println!("\n--- Test: Create specific certificate ---");
                    match certify(
                        Certificants::List(vec![holder.pub_key.clone()]),
                        Policy::String("specific.path".to_string()),
                        &authority,
                        CertifyOptions::default()
                    ).await {
                        Ok(cert) => {
                            println!("✓ Specific certificate: Success");
                            println!("  - Certificate: {}...", &cert[..30.min(cert.len())]);
                            success_count += 1;
                        }
                        Err(e) => {
                            println!("✗ Specific certificate: Failed - {}", e);
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

