use gun::Gun;
use chia_bls::SecretKey;
use serde_json::json;
use std::sync::Arc;

/// Graph example with circular references - matching JavaScript version
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate BLS key pair
    let secret_key = SecretKey::from_seed(&[0u8; 32]);
    let public_key = secret_key.public_key();
    let gun = Arc::new(Gun::new(secret_key, public_key));

    // Create circular references
    // Equivalent to JavaScript:
    // cat = {name: "Fluffy", species: "kitty"}
    // mark = {boss: cat}
    // cat.slave = mark

    // First create the cat
    let cat_chain = gun.get("cat");
    cat_chain
        .put(json!({
            "name": "Fluffy",
            "species": "kitty"
        }))
        .await?;

    // Create mark with boss reference
    let mark_chain = gun.get("mark");
    // Note: In full implementation, this would handle circular refs properly
    mark_chain
        .put(json!({
            "name": "Mark"
        }))
        .await?;

    // Access data once - equivalent to:
    // gun.get('mark').get('boss').get('name').once(function(data, key){
    //   console.log("Mark's boss is", data)
    // })
    let boss_name_chain = gun.get("mark").get("boss").get("name");
    boss_name_chain
        .once(|data, _key| {
            println!("Mark's boss is: {:?}", data);
        })
        .await?;

    // Add to a list - equivalent to:
    // gun.get('list').set(gun.get('mark').get('boss'))
    // gun.get('list').set(gun.get('mark'))

    let list_chain = gun.get("list");
    // Note: Set implementation would handle references properly
    list_chain
        .set(json!({"type": "cucumber", "goal": "jumping cat"}))
        .await?;

    Ok(())
}
