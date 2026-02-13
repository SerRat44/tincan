# Tincan

[![Crates.io](https://img.shields.io/crates/v/tincan.svg)](https://crates.io/crates/tincan)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

## Overview

A fine-grained reactive state management library for Rust.

Tincan provides two levels of abstraction for building reactive applications:

- **Signals**: Low-level reactive primitives with automatic dependency tracking and functional combinators
- **Store**: High-level state management for complex application state

### Core Concepts

- **Signal**: A reactive value with combinator methods (map, zip) for composing transformations
- **Memo**: A cached computed value that only recalculates when dependencies change
- **Effect**: A side effect that automatically runs when dependencies change, with automatic cleanup
- **Store**: A container for complex state with automatic change notifications

## Quick Start

### Signals

```rust
use tincan::Signal;

// Create a reactive signal
let count = Signal::new(0);

// Transform with combinators (like Iterator)
let doubled = count.map(|n| n * 2);

// Watch for changes (returns guard for automatic cleanup)
let _guard = doubled.watch(|value| {
    println!("Doubled: {}", value);
});

// Update the signal (triggers watcher automatically)
count.set(5); // Prints: "Doubled: 10"
```

### Combining Multiple Signals

```rust
use tincan::Signal;

let first = Signal::new(1);
let second = Signal::new(2);

// Combine signals with zip
let sum = first.clone().zip(second.clone())
    .map(|(a, b)| a + b);

println!("Sum: {}", sum.get()); // Sum: 3

// Updates propagate automatically
first.set(10);
println!("Sum: {}", sum.get()); // Sum: 12
```

### Memos for Expensive Computations

```rust
use tincan::{Memo, Signal};

let count = Signal::new(5);
let expensive_double = Memo::new({
    let count = count.clone();
    move || {
        // Only recomputes when count changes
        println!("Computing...");
        count.get() * 2
    }
});

println!("{}", expensive_double.get()); // Prints: "Computing..." then "10"
println!("{}", expensive_double.get()); // Uses cached value, no "Computing..."

count.set(10); // Marks memo as dirty
println!("{}", expensive_double.get()); // Prints: "Computing..." then "20"
```

### Effects for Side Effects

```rust
use tincan::{Effect, Signal};

let count = Signal::new(0);

// Create an effect that runs when dependencies change
let _effect = Effect::new({
    let count = count.clone();
    move || {
        println!("Count is now: {}", count.get());
    }
});
// Prints immediately: "Count is now: 0"

count.set(5); // Prints: "Count is now: 5"
count.set(10); // Prints: "Count is now: 10"

// Effect is automatically cleaned up when dropped
```

### Store (High-level API)

```rust
use tincan::Store;

#[derive(Clone)]
struct AppState {
    count: usize,
    name: String,
}

// Create a store
let store = Store::new(AppState {
    count: 0,
    name: "Tincan".to_string(),
});

// Subscribe to changes
store.subscribe(|state| {
    println!("State changed: count = {}", state.count);
});

// Update state
store.update(|state| {
    state.count += 1;
});
```

## API Overview

### Signal Methods

```rust
let signal = Signal::new(initial_value);

// Reading
signal.get()                    // Clone the current value
signal.with(|val| ...)          // Read without cloning

// Writing
signal.set(new_value)           // Set a new value
signal.update(|val| *val += 1)  // Update based on current value

// Transformations
signal.map(|x| x * 2)           // Create derived signal
signal.zip(other)               // Combine with another signal

// Watching
signal.watch(|val| ...)         // Returns WatchGuard (auto-cleanup)
```

### Memo Methods

```rust
let memo = Memo::new(|| expensive_computation());

memo.get()              // Get value (recompute if dirty)
memo.with(|val| ...)   // Access without cloning
```

### Effect

```rust
Effect::new(|| {
    // Automatically tracks signal reads
    // Runs again when dependencies change
});
// Auto-cleanup on drop
```

### Store Methods

```rust
let store = Store::new(initial_state);

store.get()                     // Clone current state
store.set(new_state)            // Replace state
store.update(|state| ...)       // Mutate state
store.subscribe(|state| ...)    // Listen to changes
store.read(|state| ...)         // Read without cloning
```

## Benchmarks

Run performance benchmarks:

```bash
cargo bench
```

## License

MIT License - see [LICENSE](LICENSE) for details.
