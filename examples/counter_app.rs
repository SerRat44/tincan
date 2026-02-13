//! Complete counter application demonstrating all features together

use std::thread;
use std::time::Duration;
use tincan::{Effect, Memo, Signal, Store};

#[derive(Clone, Debug)]
struct CounterState {
    count: i32,
    step: i32,
    history: Vec<i32>,
}

impl CounterState {
    fn new() -> Self {
        Self {
            count: 0,
            step: 1,
            history: vec![0],
        }
    }

    fn increment(&mut self) {
        self.count += self.step;
        self.history.push(self.count);
    }

    fn decrement(&mut self) {
        self.count -= self.step;
        self.history.push(self.count);
    }

    fn reset(&mut self) {
        self.count = 0;
        self.history.push(0);
    }
}

fn main() {
    println!("=== Complete Counter Application ===\n");

    // Using Store for complex state management
    println!("1. Initializing counter with Store");
    let store = Store::new(CounterState::new());

    // Setup a subscriber to log changes
    store.subscribe(|state| {
        println!("   [State] Count: {}, Step: {}", state.count, state.step);
    });

    // Using Signals for real-time updates
    println!("\n2. Creating signal for display updates");
    let display_count = Signal::new(0);

    // Sync store to signal
    let display_clone = display_count.clone();
    let _sync_effect = Effect::new({
        let store = store.clone();
        move || {
            let count = store.read(|s| s.count);
            display_clone.set(count);
        }
    });

    // Using Memo for derived computations
    println!("\n3. Setting up memoized computations");
    let is_positive = Memo::new({
        let display = display_count.clone();
        move || display.get() > 0
    });

    let is_even = Memo::new({
        let display = display_count.clone();
        move || display.get() % 2 == 0
    });

    let absolute_value = Memo::new({
        let display = display_count.clone();
        move || display.get().abs()
    });

    // Display initial state
    let print_state = || {
        let count = display_count.get();
        let pos = is_positive.get();
        let even = is_even.get();
        let abs = absolute_value.get();
        println!(
            "   Count: {} | Positive: {} | Even: {} | Abs: {}",
            count, pos, even, abs
        );
    };

    println!("\n4. Initial state:");
    print_state();

    // Simulate user interactions
    println!("\n5. Incrementing...");
    store.update(|state| state.increment());
    thread::sleep(Duration::from_millis(50));
    print_state();

    store.update(|state| state.increment());
    thread::sleep(Duration::from_millis(50));
    print_state();

    store.update(|state| state.increment());
    thread::sleep(Duration::from_millis(50));
    print_state();

    println!("\n6. Changing step size to 5");
    store.update(|state| state.step = 5);

    println!("\n7. Incrementing with new step...");
    store.update(|state| state.increment());
    thread::sleep(Duration::from_millis(50));
    print_state();

    println!("\n8. Decrementing...");
    store.update(|state| state.decrement());
    thread::sleep(Duration::from_millis(50));
    print_state();

    store.update(|state| state.decrement());
    thread::sleep(Duration::from_millis(50));
    print_state();

    store.update(|state| state.decrement());
    thread::sleep(Duration::from_millis(50));
    print_state();

    println!("\n9. History:");
    store.read(|state| {
        println!("   {:?}", state.history);
    });

    println!("\n10. Resetting...");
    store.update(|state| state.reset());
    thread::sleep(Duration::from_millis(50));
    print_state();

    println!("\n11. Final history:");
    store.read(|state| {
        println!("   {:?}", state.history);
    });

    println!("\n✓ Counter application complete!");
}
