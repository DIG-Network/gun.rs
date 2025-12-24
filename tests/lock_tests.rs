use gun::{Gun, GunOptions};
use std::sync::Arc;
use tokio::time::{timeout, Duration};

/// Test: Verify connected_peer_count doesn't deadlock under contention
#[tokio::test]
async fn test_lock_no_deadlock_on_contention() {
    timeout(Duration::from_secs(30), async {
        const RELAY_URL: &str =
            "http://dig-relay-prod.eba-2cmanxbe.us-east-1.elasticbeanstalk.com/gun";

        let options = GunOptions {
            peers: vec![RELAY_URL.to_string()],
            ..Default::default()
        };
        let gun = Arc::new(Gun::with_options(options).await.unwrap());

        // Wait for connection
        assert!(gun.wait_for_connection(10000).await);

        // Spawn many concurrent calls to connected_peer_count
        let mut handles = Vec::new();
        const NUM_CONCURRENT: usize = 50;

        for _ in 0..NUM_CONCURRENT {
            let gun_clone = gun.clone();
            let handle = tokio::spawn(async move {
                // This should complete quickly without deadlocking
                let count = gun_clone.connected_peer_count().await;
                count
            });
            handles.push(handle);
        }

        // Wait for all to complete with timeout
        let results: Vec<_> = futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();

        // All should have returned successfully
        assert_eq!(results.len(), NUM_CONCURRENT);

        // All should have returned the same count (or 0 if timing issues)
        let first_count = results[0];
        for count in &results {
            // Allow some variance due to timing, but should be consistent
            assert!(
                (*count as i32 - first_count as i32).abs() <= 1,
                "Counts should be consistent"
            );
        }

        println!(
            "Completed {} concurrent connected_peer_count() calls without deadlock",
            NUM_CONCURRENT
        );
    })
    .await
    .expect("Lock contention test timed out");
}

/// Test: Verify is_connected doesn't deadlock
#[tokio::test]
async fn test_lock_is_connected_no_deadlock() {
    timeout(Duration::from_secs(30), async {
        const RELAY_URL: &str =
            "http://dig-relay-prod.eba-2cmanxbe.us-east-1.elasticbeanstalk.com/gun";

        let options = GunOptions {
            peers: vec![RELAY_URL.to_string()],
            ..Default::default()
        };
        let gun = Arc::new(Gun::with_options(options).await.unwrap());

        // Wait for connection
        assert!(gun.wait_for_connection(10000).await);

        // Make many concurrent calls to is_connected
        let mut handles = Vec::new();
        const NUM_CONCURRENT: usize = 50;

        for _ in 0..NUM_CONCURRENT {
            let gun_clone = gun.clone();
            let handle = tokio::spawn(async move { gun_clone.is_connected().await });
            handles.push(handle);
        }

        let results: Vec<_> = futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();

        assert_eq!(results.len(), NUM_CONCURRENT);

        // All should return true (we waited for connection)
        for result in &results {
            assert!(*result, "Should be connected");
        }

        println!(
            "Completed {} concurrent is_connected() calls without deadlock",
            NUM_CONCURRENT
        );
    })
    .await
    .expect("is_connected test timed out");
}

/// Test: Verify wait_for_connection doesn't deadlock
#[tokio::test]
async fn test_lock_wait_for_connection_no_deadlock() {
    timeout(Duration::from_secs(30), async {
        const RELAY_URL: &str =
            "http://dig-relay-prod.eba-2cmanxbe.us-east-1.elasticbeanstalk.com/gun";

        let options = GunOptions {
            peers: vec![RELAY_URL.to_string()],
            ..Default::default()
        };
        let gun = Arc::new(Gun::with_options(options).await.unwrap());

        // Call wait_for_connection from multiple tasks
        let mut handles = Vec::new();
        const NUM_CONCURRENT: usize = 10;

        for _ in 0..NUM_CONCURRENT {
            let gun_clone = gun.clone();
            let handle = tokio::spawn(async move { gun_clone.wait_for_connection(5000).await });
            handles.push(handle);
        }

        let results: Vec<_> = futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();

        assert_eq!(results.len(), NUM_CONCURRENT);

        // All should succeed
        for result in &results {
            assert!(*result, "Should successfully wait for connection");
        }

        println!(
            "Completed {} concurrent wait_for_connection() calls without deadlock",
            NUM_CONCURRENT
        );
    })
    .await
    .expect("wait_for_connection test timed out");
}
