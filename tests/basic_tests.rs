use gun::Gun;
use std::sync::Arc;
use serde_json::json;
use tokio;

#[tokio::test]
async fn test_basic_put_get() {
    let gun = Gun::new();
    let chain = gun.get("test");

    // Put data
    chain
        .put(json!({"name": "test", "value": 42}))
        .await
        .unwrap();

    // Get data
    let mut received = false;
    chain
        .once(|data, _key| {
            assert!(data.is_object());
            if let Some(obj) = data.as_object() {
                assert_eq!(obj.get("name").and_then(|v| v.as_str()), Some("test"));
                assert_eq!(obj.get("value").and_then(|v| v.as_i64()), Some(42));
            }
            received = true;
        })
        .await
        .unwrap();

    assert!(received);
}

#[tokio::test]
async fn test_chain_get() {
    let gun = Gun::new();
    let chain = gun.get("user").get("name");

    chain.put(json!("Alice")).await.unwrap();

    let mut received = false;
    chain
        .once(|data, _key| {
            assert_eq!(data.as_str(), Some("Alice"));
            received = true;
        })
        .await
        .unwrap();

    assert!(received);
}

#[tokio::test]
async fn test_chain_back() {
    let gun = Gun::new();
    let root = gun.root();
    let user = root.get("user");
    let name = user.get("name");

    // Go back to user
    let back_to_user = name.back(None);
    assert!(back_to_user.is_some());

    // Go back to root
    let back_to_root = user.back(None);
    assert!(back_to_root.is_some());
}

#[tokio::test]
async fn test_event_subscription() {
    let gun = Gun::new();
    let chain = gun.get("counter");

    let count = Arc::new(std::sync::Mutex::new(0));
    let count_clone = count.clone();
    chain.on(move |_data, _key| {
        *count_clone.lock().unwrap() += 1;
    });

    // Put multiple times
    for i in 0..5 {
        chain.put(json!(i)).await.unwrap();
    }

    // Give events time to process
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Events should have been triggered
    assert!(*count.lock().unwrap() >= 1);
}
