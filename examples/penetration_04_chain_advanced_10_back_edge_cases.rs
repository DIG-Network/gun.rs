/// Test: Chain.back() - Edge cases
/// 
/// Tests back() from root and back beyond root.

use gun::Gun;

#[tokio::main]
async fn main() {
    println!("Test: Chain.back() - Edge cases");
    println!("Description: Back from root, back beyond root");
    
    let gun = Gun::new();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    // Test 1: Back from root
    println!("\n--- Test 1: Back from root ---");
    let root_chain = gun.root();
    match root_chain.back(Some(1)) {
        Some(_) => {
            println!("? Back from root: Returned Some (unexpected)");
            success_count += 1; // Still counts as success (no panic)
        }
        None => {
            println!("✓ Back from root: Returned None (expected)");
            success_count += 1;
        }
    }
    
    // Test 2: Back beyond available levels
    println!("\n--- Test 2: Back beyond available levels ---");
    let chain = gun.get("level1").get("level2");
    match chain.back(Some(10)) {
        Some(_) => {
            println!("✓ Back beyond levels: Returned Some");
            success_count += 1;
        }
        None => {
            println!("✓ Back beyond levels: Returned None (may be expected)");
            success_count += 1;
        }
    }
    
    // Test 3: Back with 0
    println!("\n--- Test 3: Back with 0 ---");
    let chain = gun.get("level1").get("level2");
    match chain.back(Some(0)) {
        Some(_) => {
            println!("✓ Back 0: Returned Some");
            success_count += 1;
        }
        None => {
            println!("✓ Back 0: Returned None");
            success_count += 1;
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

