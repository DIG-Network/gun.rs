use gun::{Gun, GunOptions};
use serde_json::json;
use std::sync::Arc;
use tokio::time::{sleep, timeout, Duration};

/// Stress test: Many concurrent connections
#[tokio::test]
#[ignore] // Mark as ignored by default - runs only when explicitly requested
async fn test_stress_concurrent_connections() {
    timeout(Duration::from_secs(120), async {
        const NUM_CONNECTIONS: usize = 10;
        const RELAY_URL: &str =
            "http://dig-relay-prod.eba-2cmanxbe.us-east-1.elasticbeanstalk.com/gun";

        let mut guns = Vec::new();

        // Create multiple Gun instances
        for i in 0..NUM_CONNECTIONS {
            let options = GunOptions {
                peers: vec![RELAY_URL.to_string()],
                ..Default::default()
            };
            let gun = Arc::new(Gun::with_options(options).await.unwrap());
            guns.push(gun);
            println!("Created gun instance {}", i);
        }

        // Wait for all connections
        for (i, gun) in guns.iter().enumerate() {
            sleep(Duration::from_millis(500)).await;
            let count = gun.connected_peer_count().await;
            println!("Gun {} has {} connected peers", i, count);
            assert!(
                count > 0 || gun.wait_for_connection(5000).await,
                "Gun {} should be connected",
                i
            );
        }

        // Test data exchange between all instances
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let test_key = format!("stress_test_{}", timestamp);

        // Write from first instance
        guns[0]
            .get(&test_key)
            .put(json!({
                "source": "gun0",
                "value": "stress_test_data"
            }))
            .await
            .unwrap();

        // Wait and verify other instances receive it
        sleep(Duration::from_secs(5)).await;

        let mut received_count = 0;
        for (i, gun) in guns.iter().enumerate().skip(1) {
            let received = Arc::new(std::sync::Mutex::new(false));
            let received_clone = received.clone();
            let key_clone = test_key.clone();

            gun.get(&key_clone).on(move |data, _key| {
                if !data.is_null() {
                    if let Some(obj) = data.as_object() {
                        if obj.get("source").and_then(|v| v.as_str()) == Some("gun0") {
                            let mut flag = received_clone.lock().unwrap();
                            *flag = true;
                        }
                    }
                }
            });

            sleep(Duration::from_millis(1000)).await;
            if *received.lock().unwrap() {
                received_count += 1;
                println!("Gun {} received data from gun0", i);
            }
        }

        // At least some instances should receive the data
        println!("Received count: {}/{}", received_count, NUM_CONNECTIONS - 1);
        assert!(
            received_count > 0,
            "At least some instances should receive data"
        );
    })
    .await
    .expect("Stress test timed out after 120 seconds");
}

/// Stress test: Lock contention
#[tokio::test]
#[ignore]
async fn test_stress_lock_contention() {
    timeout(Duration::from_secs(60), async {
        const RELAY_URL: &str =
            "http://dig-relay-prod.eba-2cmanxbe.us-east-1.elasticbeanstalk.com/gun";
        const NUM_ITERATIONS: usize = 100;

        let options = GunOptions {
            peers: vec![RELAY_URL.to_string()],
            ..Default::default()
        };
        let gun = Arc::new(Gun::with_options(options).await.unwrap());

        // Wait for connection
        assert!(gun.wait_for_connection(10000).await, "Should connect");

        // Perform many concurrent operations to stress test locks
        let mut handles = Vec::new();

        for i in 0..NUM_ITERATIONS {
            let gun_clone = gun.clone();
            let handle = tokio::spawn(async move {
                let key = format!("stress_lock_{}", i);

                // Concurrent writes
                gun_clone
                    .get(&key)
                    .put(json!({
                        "iteration": i,
                        "data": format!("stress_data_{}", i)
                    }))
                    .await
                    .unwrap();

                // Concurrent reads (should not deadlock)
                let count = gun_clone.connected_peer_count().await;
                // Just verify the call doesn't deadlock (count is usize, always >= 0)
                let _ = count;

                // Concurrent checks
                let _is_connected = gun_clone.is_connected().await;
            });
            handles.push(handle);
        }

        // Wait for all operations to complete
        for handle in handles {
            handle.await.unwrap();
        }

        println!(
            "Completed {} concurrent operations without deadlock",
            NUM_ITERATIONS
        );
    })
    .await
    .expect("Lock contention test timed out");
}

/// Stress test: Rapid connection/disconnection
#[tokio::test]
#[ignore]
async fn test_stress_rapid_reconnect() {
    timeout(Duration::from_secs(120), async {
        const RELAY_URL: &str =
            "http://dig-relay-prod.eba-2cmanxbe.us-east-1.elasticbeanstalk.com/gun";
        const NUM_RECONNECTS: usize = 20;

        for i in 0..NUM_RECONNECTS {
            println!("Reconnect iteration {}", i);

            let options = GunOptions {
                peers: vec![RELAY_URL.to_string()],
                ..Default::default()
            };
            let gun = Arc::new(Gun::with_options(options).await.unwrap());

            // Wait for connection
            assert!(
                gun.wait_for_connection(10000).await,
                "Should connect on iteration {}",
                i
            );

            let count = gun.connected_peer_count().await;
            assert!(count > 0, "Should have connected peers on iteration {}", i);

            // Write some data
            let key = format!("reconnect_test_{}", i);
            gun.get(&key)
                .put(json!({
                    "iteration": i,
                    "connected": true
                }))
                .await
                .unwrap();

            // Drop the gun instance (simulate disconnect)
            drop(gun);

            // Wait a bit before reconnecting
            sleep(Duration::from_millis(500)).await;
        }

        println!("Completed {} reconnect cycles", NUM_RECONNECTS);
    })
    .await
    .expect("Rapid reconnect test timed out");
}
