//! Integration tests for Tincan

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tincan::{create_effect, create_memo, create_signal, Store};

#[test]
fn signal_integration() {
    let (count, set_count) = create_signal(0);

    // Test read
    assert_eq!(count.get(), 0);

    // Test write
    set_count.set(42);
    assert_eq!(count.get(), 42);

    // Test update
    set_count.update(|n| *n += 10);
    assert_eq!(count.get(), 52);
}

#[test]
fn memo_integration() {
    let (a, set_a) = create_signal(5);
    let (b, set_b) = create_signal(10);

    let sum = create_memo({
        let a = a.clone();
        let b = b.clone();
        move || a.get() + b.get()
    });

    assert_eq!(sum.get(), 15);

    set_a.set(20);
    assert_eq!(sum.get(), 30);

    set_b.set(5);
    assert_eq!(sum.get(), 25);
}

#[test]
fn effect_integration() {
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let (signal, set_signal) = create_signal(0);

    create_effect({
        let signal = signal.clone();
        move || {
            let _ = signal.get();
            counter_clone.fetch_add(1, Ordering::SeqCst);
        }
    });

    // Effect runs immediately
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn store_integration() {
    #[derive(Clone, PartialEq, Debug)]
    struct State {
        count: i32,
        name: String,
    }

    let store = Store::new(State {
        count: 0,
        name: "test".to_string(),
    });

    // Test get
    assert_eq!(store.get().count, 0);

    // Test update
    store.update(|state| {
        state.count = 42;
        state.name = "updated".to_string();
    });

    assert_eq!(store.get().count, 42);
    assert_eq!(store.get().name, "updated");

    // Test set
    store.set(State {
        count: 100,
        name: "new".to_string(),
    });

    assert_eq!(store.get().count, 100);
}

#[test]
fn store_subscription() {
    let store = Store::new(0);
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    store.subscribe(move |_| {
        counter_clone.fetch_add(1, Ordering::SeqCst);
    });

    assert_eq!(counter.load(Ordering::SeqCst), 0);

    store.update(|n| *n += 1);
    assert_eq!(counter.load(Ordering::SeqCst), 1);

    store.update(|n| *n += 1);
    assert_eq!(counter.load(Ordering::SeqCst), 2);
}

#[test]
fn complex_reactive_chain() {
    let (input, set_input) = create_signal(1);

    let doubled = create_memo({
        let input = input.clone();
        move || input.get() * 2
    });

    let quadrupled = create_memo({
        let doubled = doubled.clone();
        move || doubled.get() * 2
    });

    assert_eq!(quadrupled.get(), 4);

    set_input.set(5);
    assert_eq!(quadrupled.get(), 20);
}
