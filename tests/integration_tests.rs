//! Integration tests for Tincan

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tincan::runtime::ReactiveRuntime;
use tincan::{Effect, Memo, Signal, Store};

#[test]
fn signal_integration() {
    ReactiveRuntime::scope(|| {
        let count = Signal::new(0);

        // Test read
        assert_eq!(count.get(), 0);

        // Test write
        count.set(42);
        assert_eq!(count.get(), 42);

        // Test update
        count.update(|n| *n += 10);
        assert_eq!(count.get(), 52);
    });
}

#[test]
fn signal_with() {
    ReactiveRuntime::scope(|| {
        let text = Signal::new(String::from("hello world"));
        let len = text.with(|s| s.len());
        assert_eq!(len, 11);
    });
}

#[test]
fn signal_map() {
    ReactiveRuntime::scope(|| {
        let count = Signal::new(5);
        let doubled = count.map(|n| n * 2);
        assert_eq!(doubled.get(), 10);

        count.set(10);
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert_eq!(doubled.get(), 20);
    });
}

#[test]
fn memo_integration() {
    ReactiveRuntime::scope(|| {
        let a = Signal::new(5);
        let b = Signal::new(10);

        let sum = Memo::new({
            let a = a.clone();
            let b = b.clone();
            move || a.get() + b.get()
        });

        assert_eq!(sum.get(), 15);

        a.set(20);
        assert_eq!(sum.get(), 30);

        b.set(5);
        assert_eq!(sum.get(), 25);
    });
}

#[test]
fn effect_integration() {
    ReactiveRuntime::scope(|| {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let signal = Signal::new(0);

        let _effect = Effect::new({
            let signal = signal.clone();
            move || {
                let _ = signal.get();
                counter_clone.fetch_add(1, Ordering::SeqCst);
            }
        });

        // Effect runs immediately
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    });
}

#[test]
fn signal_watch() {
    ReactiveRuntime::scope(|| {
        let count = Signal::new(0);
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let _guard = count.watch(move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        // Called immediately
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        count.set(5);
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    });
}

#[test]
fn store_integration() {
    ReactiveRuntime::scope(|| {
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
    });
}

#[test]
fn store_subscription() {
    ReactiveRuntime::scope(|| {
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
    });
}

#[test]
fn complex_reactive_chain() {
    ReactiveRuntime::scope(|| {
        let input: Signal<i32> = Signal::new(1);

        let doubled = Memo::new({
            let input = input.clone();
            move || input.get() * 2
        });

        let quadrupled = Memo::new({
            let doubled = doubled.clone();
            move || doubled.get() * 2
        });

        assert_eq!(quadrupled.get(), 4);

        input.set(5);
        assert_eq!(quadrupled.get(), 20);
    });
}
