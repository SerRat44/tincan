# Tincan

[![Crates.io](https://img.shields.io/crates/v/tincan.svg)](https://crates.io/crates/tincan)
[![Documentation](https://docs.rs/tincan/badge.svg)](https://docs.rs/tincan)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

## Overview

A fine-grained reactive state management library for Rust.

Tincan provides two levels of abstraction for building reactive applications:

- **Signals**: Low-level reactive primitives with automatic dependency tracking
- **Store**: High-level state management for complex application state

### Core Concepts

- **Signal**: A reactive value that notifies dependents when changed
- **Memo**: A cached computed value that only recalculates when dependencies change
- **Effect**: A side effect that automatically runs when dependencies change
- **Store**: A container for complex state with automatic change notifications

## Quick Start

### Signals (Low-level API)

```rust
use tincan::{create_signal, create_memo, create_effect};

// Create a reactive signal
let (count, set_count) = create_signal(0);

// Create a derived value
let doubled = create_memo(move || count.get() * 2);

// Create a side effect
create_effect(move || {
    println!("Count: {}, Doubled: {}", count.get(), doubled.get());
});

// Update the signal (triggers effect automatically)
set_count.set(5);
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

## Examples

Run the included examples:

```bash
# Basic signal usage
cargo run --example basic_signal

# Store with complex state
cargo run --example store_example

# Derived values with memos
cargo run --example memo_example
```

## Benchmarks

Run performance benchmarks:

```bash
cargo bench
```
