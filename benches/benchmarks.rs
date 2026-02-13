use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;

use tincan::{Memo, Signal, Store};

fn signal_creation_benchmark(c: &mut Criterion) {
    c.bench_function("signal_creation", |b| {
        b.iter(|| {
            let signal: Signal<i32> = Signal::new(black_box(42));
            signal
        });
    });
}

fn signal_read_benchmark(c: &mut Criterion) {
    let signal: Signal<i32> = Signal::new(42);

    c.bench_function("signal_read", |b| {
        b.iter(|| {
            black_box(signal.get());
        });
    });
}

fn signal_write_benchmark(c: &mut Criterion) {
    let signal: Signal<i32> = Signal::new(0);

    c.bench_function("signal_write", |b| {
        let mut i = 0;
        b.iter(|| {
            signal.set(black_box(i));
            i += 1;
        });
    });
}

fn memo_computation_benchmark(c: &mut Criterion) {
    let a: Signal<i32> = Signal::new(5);
    let b: Signal<i32> = Signal::new(10);

    let sum = Memo::new({
        let a = a.clone();
        let b = b.clone();
        move || a.get() + b.get()
    });

    c.bench_function("memo_computation", |b| {
        b.iter(|| {
            black_box(sum.get());
        });
    });
}

fn store_update_benchmark(c: &mut Criterion) {
    #[derive(Clone)]
    struct State {
        counter: usize,
        name: String,
    }

    let store = Store::new(State {
        counter: 0,
        name: "test".to_string(),
    });

    c.bench_function("store_update", |b| {
        let mut i = 0;
        b.iter(|| {
            store.update(|state| {
                state.counter = black_box(i);
            });
            i += 1;
        });
    });
}

fn store_subscribe_benchmark(c: &mut Criterion) {
    #[derive(Clone)]
    struct State {
        value: usize,
    }

    let mut group = c.benchmark_group("store_subscribe");

    for subscriber_count in [1, 10, 100].iter() {
        let store = Store::new(State { value: 0 });

        for _ in 0..*subscriber_count {
            store.subscribe(|_| {
                // Empty subscriber
            });
        }

        group.bench_with_input(
            BenchmarkId::from_parameter(subscriber_count),
            subscriber_count,
            |b, _| {
                let mut i = 0;
                b.iter(|| {
                    store.update(|state| state.value = black_box(i));
                    i += 1;
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    signal_creation_benchmark,
    signal_read_benchmark,
    signal_write_benchmark,
    memo_computation_benchmark,
    store_update_benchmark,
    store_subscribe_benchmark,
);
criterion_main!(benches);
