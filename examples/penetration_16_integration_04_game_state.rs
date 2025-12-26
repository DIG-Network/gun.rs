/// Test: Game state synchronization
/// 
/// Tests game state synchronization.

use gun::Gun;
use serde_json::json;

#[tokio::main]
async fn main() {
    println!("Test: Game state synchronization");
    println!("Description: Test game state synchronization");
    
    let gun = Gun::new();
    let mut success_count = 0;
    let mut fail_count = 0;
    
    println!("\n--- Test: Game state synchronization ---");
    match gun.get("game").get("state").put(json!({
        "players": [
            {"id": 1, "x": 10, "y": 20},
            {"id": 2, "x": 30, "y": 40}
        ],
        "score": 100
    })).await {
        Ok(_) => {
            println!("✓ Game state synchronization: State created");
            success_count += 1;
        }
        Err(e) => {
            println!("✗ Game state synchronization: Failed - {}", e);
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

