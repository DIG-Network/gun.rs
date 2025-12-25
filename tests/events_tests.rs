//! Comprehensive tests for event system
//! Tests event emission, listener registration, and removal

use gun::events::{Event, EventEmitter};
use serde_json::json;
use std::sync::{Arc, Mutex};

#[test]
fn test_event_emitter_new() {
    let emitter = EventEmitter::new();
    assert_eq!(emitter.listener_count("test"), 0);
}

#[test]
fn test_event_emitter_on_emit() {
    let emitter = EventEmitter::new();
    let count = Arc::new(Mutex::new(0));
    let count_clone = count.clone();

    emitter.on(
        "test",
        Box::new(move |_event| {
            *count_clone.lock().unwrap() += 1;
        }),
    );

    let event = Event {
        event_type: "test".to_string(),
        data: json!({"value": 42}),
    };

    emitter.emit(&event);

    assert_eq!(*count.lock().unwrap(), 1);
}

#[test]
fn test_event_emitter_multiple_listeners() {
    let emitter = EventEmitter::new();
    let count = Arc::new(Mutex::new(0));

    // Register multiple listeners
    for _ in 0..5 {
        let count_clone = count.clone();
        emitter.on(
            "test",
            Box::new(move |_event| {
                *count_clone.lock().unwrap() += 1;
            }),
        );
    }

    let event = Event {
        event_type: "test".to_string(),
        data: json!(null),
    };

    emitter.emit(&event);

    assert_eq!(*count.lock().unwrap(), 5);
}

#[test]
fn test_event_emitter_off() {
    let emitter = EventEmitter::new();
    let count = Arc::new(Mutex::new(0));
    let count_clone = count.clone();

    let id = emitter.on(
        "test",
        Box::new(move |_event| {
            *count_clone.lock().unwrap() += 1;
        }),
    );

    let event = Event {
        event_type: "test".to_string(),
        data: json!(null),
    };

    emitter.emit(&event);
    assert_eq!(*count.lock().unwrap(), 1);

    // Remove listener
    emitter.off("test", id);

    emitter.emit(&event);
    // Count should not increase
    assert_eq!(*count.lock().unwrap(), 1);
}

#[test]
fn test_event_emitter_off_all() {
    let emitter = EventEmitter::new();
    let count = Arc::new(Mutex::new(0));

    for _ in 0..3 {
        let count_clone = count.clone();
        emitter.on(
            "test",
            Box::new(move |_event| {
                *count_clone.lock().unwrap() += 1;
            }),
        );
    }

    emitter.off_all("test");

    let event = Event {
        event_type: "test".to_string(),
        data: json!(null),
    };

    emitter.emit(&event);
    assert_eq!(*count.lock().unwrap(), 0);
}

#[test]
fn test_event_emitter_listener_count() {
    let emitter = EventEmitter::new();

    assert_eq!(emitter.listener_count("test"), 0);

    emitter.on("test", Box::new(|_event| {}));
    assert_eq!(emitter.listener_count("test"), 1);

    emitter.on("test", Box::new(|_event| {}));
    assert_eq!(emitter.listener_count("test"), 2);

    emitter.on("other", Box::new(|_event| {}));
    assert_eq!(emitter.listener_count("test"), 2);
    assert_eq!(emitter.listener_count("other"), 1);
}

#[test]
fn test_event_emitter_different_event_types() {
    let emitter = EventEmitter::new();
    let count1 = Arc::new(Mutex::new(0));
    let count2 = Arc::new(Mutex::new(0));

    let c1 = count1.clone();
    emitter.on(
        "event1",
        Box::new(move |_event| {
            *c1.lock().unwrap() += 1;
        }),
    );

    let c2 = count2.clone();
    emitter.on(
        "event2",
        Box::new(move |_event| {
            *c2.lock().unwrap() += 1;
        }),
    );

    let event1 = Event {
        event_type: "event1".to_string(),
        data: json!(null),
    };

    let event2 = Event {
        event_type: "event2".to_string(),
        data: json!(null),
    };

    emitter.emit(&event1);
    assert_eq!(*count1.lock().unwrap(), 1);
    assert_eq!(*count2.lock().unwrap(), 0);

    emitter.emit(&event2);
    assert_eq!(*count1.lock().unwrap(), 1);
    assert_eq!(*count2.lock().unwrap(), 1);
}

#[test]
fn test_event_emitter_event_data() {
    let emitter = EventEmitter::new();
    let received_data = Arc::new(Mutex::new(None));
    let data_clone = received_data.clone();

    emitter.on(
        "test",
        Box::new(move |event| {
            *data_clone.lock().unwrap() = Some(event.data.clone());
        }),
    );

    let event = Event {
        event_type: "test".to_string(),
        data: json!({"key": "value", "number": 42}),
    };

    emitter.emit(&event);

    let data = received_data.lock().unwrap().take();
    assert!(data.is_some());
    let data_value = data.unwrap();
    assert_eq!(
        data_value.get("key").and_then(|v| v.as_str()),
        Some("value")
    );
    assert_eq!(data_value.get("number").and_then(|v| v.as_i64()), Some(42));
}

#[test]
fn test_event_emitter_default() {
    let emitter = EventEmitter::default();
    assert_eq!(emitter.listener_count("test"), 0);
}
