//! Comprehensive tests for message deduplication
//! Tests duplicate detection, message tracking, and expiration

use gun::dup::Dup;

#[test]
fn test_dup_new_default() {
    let dup = Dup::new_default();
    // Should not have any messages tracked
    assert!(!dup.check("test_id"));
}

#[test]
fn test_dup_new_custom() {
    let dup = Dup::new(100, 5000);
    assert!(!dup.check("test_id"));
}

#[test]
fn test_dup_check_track() {
    let mut dup = Dup::new_default();

    // New message should not be duplicate
    assert!(!dup.check("msg1"));

    // Track it
    dup.track("msg1");

    // Now should be duplicate
    assert!(dup.check("msg1"));
}

#[test]
fn test_dup_multiple_messages() {
    let mut dup = Dup::new_default();

    dup.track("msg1");
    dup.track("msg2");
    dup.track("msg3");

    assert!(dup.check("msg1"));
    assert!(dup.check("msg2"));
    assert!(dup.check("msg3"));
    assert!(!dup.check("msg4")); // Not tracked
}

#[test]
fn test_dup_track_with_peer() {
    let mut dup = Dup::new_default();

    dup.track_with_peer("msg1", Some("peer1"));

    assert!(dup.check("msg1"));
    assert_eq!(dup.get_via("msg1"), Some("peer1".to_string()));
}

#[test]
fn test_dup_get_via() {
    let mut dup = Dup::new_default();

    dup.track_with_peer("msg1", Some("peer1"));
    dup.track_with_peer("msg2", Some("peer2"));

    assert_eq!(dup.get_via("msg1"), Some("peer1".to_string()));
    assert_eq!(dup.get_via("msg2"), Some("peer2".to_string()));
    assert_eq!(dup.get_via("nonexistent"), None);
}

#[test]
fn test_dup_store_get() {
    let mut dup = Dup::new_default();

    let data = serde_json::json!({"key": "value"});
    dup.store("msg1", data.clone());

    let retrieved = dup.get("msg1");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), data);
}

#[test]
fn test_dup_remove() {
    let mut dup = Dup::new_default();

    dup.track("msg1");
    assert!(dup.check("msg1"));

    dup.remove("msg1");
    assert!(!dup.check("msg1"));
}

#[test]
fn test_dup_expiration() {
    use std::thread;
    use std::time::Duration;

    // Create dup with short expiration (100ms)
    let mut dup = Dup::new(100, 100);

    dup.track("msg1");
    assert!(dup.check("msg1"));

    // Wait for expiration
    thread::sleep(Duration::from_millis(150));

    // Should no longer be duplicate (expired)
    assert!(!dup.check("msg1"));
}

#[test]
fn test_dup_drop_expired_all() {
    use std::thread;
    use std::time::Duration;

    let mut dup = Dup::new(100, 100);

    dup.track("msg1");
    thread::sleep(Duration::from_millis(150));
    dup.track("msg2"); // Still fresh

    dup.drop_expired_all();

    assert!(!dup.check("msg1")); // Expired
    assert!(dup.check("msg2")); // Still valid
}

#[test]
fn test_dup_default() {
    let dup = Dup::default();
    assert!(!dup.check("test"));
}
