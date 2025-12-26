/// Test: Chain.back() - Navigate back multiple levels
/// 
/// Tests navigating back multiple levels.

use gun::Gun;

#[tokio::main]
async fn main() {
    println!("Test: Chain.back() - Navigate back multiple levels");
    println!("Description: Navigate back multiple levels");
    
    let gun = Gun::new();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test: Navigate back multiple levels
    println!("\n--- Test: Navigate back multiple levels ---");
    let chain = gun.get("level1").get("level2").get("level3").get("level4");
    
    // Back 2 levels
    match chain.back(Some(2)) {
        Some(_) => {
            println!("✓ Back 2 levels: Success");
            success_count += 1;
        }
        None => {
            println!("✗ Back 2 levels: Returned None");
            fail_count += 1;
        }
    }
    
    // Back 3 levels
    match chain.back(Some(3)) {
        Some(_) => {
            println!("✓ Back 3 levels: Success");
            success_count += 1;
        }
        None => {
            println!("✗ Back 3 levels: Returned None");
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

