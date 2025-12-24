use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use tokio::time::{sleep, Duration, timeout};

// Note: These tests require network connectivity to the relay server
// They may take longer to run due to network delays
// Run with: cargo test --test integration_tests -- --nocapture

const RELAY_URL: &str = "http://dig-relay-prod.eba-2cmanxbe.us-east-1.elasticbeanstalk.com/gun";

/// Test basic put and get operations with relay server
#[tokio::test]
async fn test_relay_put_get() {
    let options = GunOptions {
        peers: vec![RELAY_URL.to_string()],
        ..Default::default()
    };
    
    let gun = Arc::new(Gun::with_options(options).await.unwrap());
    
    // Put data
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let test_key = format!("test_put_get_{}", timestamp);
    let test_data = json!({
        "message": "Hello from Rust!",
        "timestamp": timestamp,
        "test": true
    });
    
    println!("Putting data with key: {}", test_key);
    gun.get(&test_key).put(test_data.clone()).await.unwrap();
    
    // Wait a bit for sync
    sleep(Duration::from_millis(500)).await;
    
    // Get data
    let mut received = false;
    gun.get(&test_key).once(|data, _key| {
        println!("Received data: {:?}", data);
        if let Some(obj) = data.as_object() {
            assert_eq!(obj.get("message").and_then(|v| v.as_str()), Some("Hello from Rust!"));
            assert_eq!(obj.get("test").and_then(|v| v.as_bool()), Some(true));
            received = true;
        }
    }).await.unwrap();
    
    assert!(received, "Data should be received from relay");
}

/// Test real-time updates with relay server
#[tokio::test]
async fn test_relay_realtime_updates() {
    let options = GunOptions {
        peers: vec![RELAY_URL.to_string()],
        ..Default::default()
    };
    
    let gun = Arc::new(Gun::with_options(options).await.unwrap());
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let test_key = format!("test_realtime_{}", timestamp);
    let update_count = Arc::new(std::sync::Mutex::new(0));
    
    // Subscribe to updates
    let update_count_clone = update_count.clone();
    gun.get(&test_key).on(move |data, _key| {
        if !data.is_null() {
            let mut count = update_count_clone.lock().unwrap();
            *count += 1;
            println!("Update received #{}: {:?}", *count, data);
        }
    });
    
    // Wait for subscription to be set up
    sleep(Duration::from_millis(200)).await;
    
    // Send multiple updates
    for i in 0..5 {
        let data = json!({
            "counter": i,
            "message": format!("Update {}", i)
        });
        gun.get(&test_key).put(data).await.unwrap();
        sleep(Duration::from_millis(200)).await;
    }
    
    // Wait for all updates to propagate
    sleep(Duration::from_millis(1000)).await;
    
    let count = update_count.lock().unwrap();
    assert!(*count >= 5, "Should receive at least 5 updates, got {}", *count);
}

/// Test nested data structure with relay
#[tokio::test]
async fn test_relay_nested_data() {
    let options = GunOptions {
        peers: vec![RELAY_URL.to_string()],
        ..Default::default()
    };
    
    let gun = Arc::new(Gun::with_options(options).await.unwrap());
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let test_key = format!("test_nested_{}", timestamp);
    
    // Put nested data
    let nested_data = json!({
        "user": {
            "name": "Test User",
            "email": "test@example.com"
        },
        "metadata": {
            "created": timestamp,
            "version": 1
        }
    });
    
    gun.get(&test_key).put(nested_data).await.unwrap();
    sleep(Duration::from_millis(500)).await;
    
    // Get nested data
    let mut received = false;
    gun.get(&test_key).once(|data, _key| {
        if let Some(obj) = data.as_object() {
            assert!(obj.contains_key("user"));
            assert!(obj.contains_key("metadata"));
            received = true;
        }
    }).await.unwrap();
    
    assert!(received, "Nested data should be received");
}

/// Test chain operations with relay
#[tokio::test]
async fn test_relay_chain_operations() {
    let options = GunOptions {
        peers: vec![RELAY_URL.to_string()],
        ..Default::default()
    };
    
    let gun = Arc::new(Gun::with_options(options).await.unwrap());
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let user_key = format!("user_{}", timestamp);
    
    // Use chain to set nested properties
    gun.get(&user_key)
        .get("profile")
        .get("name")
        .put(json!("Alice"))
        .await
        .unwrap();
    
    gun.get(&user_key)
        .get("profile")
        .get("age")
        .put(json!(30))
        .await
        .unwrap();
    
    sleep(Duration::from_millis(500)).await;
    
    // Read back using chain
    let mut name_received = false;
    gun.get(&user_key)
        .get("profile")
        .get("name")
        .once(|data, _key| {
            if let Some(name) = data.as_str() {
                assert_eq!(name, "Alice");
                name_received = true;
            }
        })
        .await
        .unwrap();
    
    assert!(name_received, "Name should be received via chain");
}

/// Test multiple concurrent connections to relay
#[tokio::test]
async fn test_relay_concurrent_connections() {
    let options1 = GunOptions {
        peers: vec![RELAY_URL.to_string()],
        ..Default::default()
    };
    let gun1 = Arc::new(Gun::with_options(options1).await.unwrap());
    
    let options2 = GunOptions {
        peers: vec![RELAY_URL.to_string()],
        ..Default::default()
    };
    let gun2 = Arc::new(Gun::with_options(options2).await.unwrap());
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let test_key = format!("test_concurrent_{}", timestamp);
    
    // Write from gun1
    gun1.get(&test_key).put(json!({"source": "gun1", "value": 100})).await.unwrap();
    
    sleep(Duration::from_millis(1000)).await;
    
    // Read from gun2 (should sync via relay)
    let mut received = false;
    gun2.get(&test_key).once(|data, _key| {
        if let Some(obj) = data.as_object() {
            assert_eq!(obj.get("source").and_then(|v| v.as_str()), Some("gun1"));
            received = true;
        }
    }).await.unwrap();
    
    assert!(received, "gun2 should receive data written by gun1 via relay");
}

/// Test that data persists across connections
#[tokio::test]
async fn test_relay_persistence() {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let test_key = format!("test_persist_{}", timestamp);
    let test_value = format!("persistent_value_{}", timestamp);
    
    // Write with first connection
    {
        let options = GunOptions {
            peers: vec![RELAY_URL.to_string()],
            ..Default::default()
        };
        let gun = Arc::new(Gun::with_options(options).await.unwrap());
        gun.get(&test_key).put(json!({"value": test_value.clone()})).await.unwrap();
        sleep(Duration::from_millis(500)).await;
    }
    
    // Wait a bit
    sleep(Duration::from_millis(500)).await;
    
    // Read with new connection
    {
        let options = GunOptions {
            peers: vec![RELAY_URL.to_string()],
            ..Default::default()
        };
        let gun = Arc::new(Gun::with_options(options).await.unwrap());
        
        // Wait for sync
        sleep(Duration::from_millis(1000)).await;
        
        let mut received = false;
        gun.get(&test_key).once(|data, _key| {
            if let Some(obj) = data.as_object() {
                if let Some(val) = obj.get("value").and_then(|v| v.as_str()) {
                    assert_eq!(val, test_value);
                    received = true;
                }
            }
        }).await.unwrap();
        
        assert!(received, "Data should persist and be retrievable with new connection");
    }
}

/// Test map operation with relay
#[tokio::test]
async fn test_relay_map_operation() {
    let options = GunOptions {
        peers: vec![RELAY_URL.to_string()],
        ..Default::default()
    };
    
    let gun = Arc::new(Gun::with_options(options).await.unwrap());
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let test_key = format!("test_map_{}", timestamp);
    
    // Put data with multiple properties
    gun.get(&test_key).put(json!({
        "prop1": "value1",
        "prop2": "value2",
        "prop3": "value3"
    })).await.unwrap();
    
    sleep(Duration::from_millis(500)).await;
    
    // Map over properties - subscribe to updates first
    let properties_found = Arc::new(std::sync::Mutex::new(Vec::new()));
    let properties_clone = properties_found.clone();
    
    // Subscribe to node to get current data
    gun.get(&test_key).on(move |data, _key| {
        if let Some(obj) = data.as_object() {
            let mut props = properties_clone.lock().unwrap();
            for (key, _value) in obj {
                if !props.contains(key) {
                    props.push(key.clone());
                    println!("Mapped property: {}", key);
                }
            }
        }
    });
    
    sleep(Duration::from_millis(500)).await;
    
    let props = properties_found.lock().unwrap();
    assert!(props.len() >= 3, "Should find at least 3 properties, got {}", props.len());
}

/// Test error handling with invalid relay URL
#[tokio::test]
async fn test_invalid_relay_url() {
    let options = GunOptions {
        peers: vec!["ws://invalid-relay-that-does-not-exist.com/gun".to_string()],
        ..Default::default()
    };
    
    // Should still create Gun instance (connections fail gracefully)
    let gun = Arc::new(Gun::with_options(options).await.unwrap());
    
    // Local operations should still work
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let test_key = format!("test_local_{}", timestamp);
    gun.get(&test_key).put(json!({"local": true})).await.unwrap();
    
    let mut received = false;
    gun.get(&test_key).once(|data, _key| {
        if let Some(obj) = data.as_object() {
            assert_eq!(obj.get("local").and_then(|v| v.as_bool()), Some(true));
            received = true;
        }
    }).await.unwrap();
    
    assert!(received, "Local operations should work even if relay is unreachable");
}

/// Test timeout handling for network operations
#[tokio::test]
async fn test_relay_timeout_handling() {
    let options = GunOptions {
        peers: vec![RELAY_URL.to_string()],
        ..Default::default()
    };
    
    let gun = Arc::new(Gun::with_options(options).await.unwrap());
    
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let test_key = format!("test_timeout_{}", timestamp);
    
    // Try to get non-existent key with timeout
    let result = timeout(
        Duration::from_secs(5),
        gun.get(&test_key).once(|_data, _key| {})
    ).await;
    
    // Should complete (either with data or without, but not hang)
    assert!(result.is_ok(), "Operation should complete within timeout");
}

/// Test that we can connect to the relay server
#[tokio::test]
async fn test_relay_connection() {
    let options = GunOptions {
        peers: vec![RELAY_URL.to_string()],
        ..Default::default()
    };
    
    let gun = Arc::new(Gun::with_options(options).await.unwrap());
    
    // Give it time to connect
    sleep(Duration::from_secs(2)).await;
    
    // Try a simple operation to verify connection works
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let test_key = format!("test_connection_{}", timestamp);
    gun.get(&test_key).put(json!({"connected": true})).await.unwrap();
    
    sleep(Duration::from_millis(500)).await;
    
    let mut verified = false;
    gun.get(&test_key).once(|data, _key| {
        if let Some(obj) = data.as_object() {
            if obj.get("connected").and_then(|v| v.as_bool()) == Some(true) {
                verified = true;
            }
        }
    }).await.unwrap();
    
    assert!(verified, "Should be able to connect and sync with relay");
}

