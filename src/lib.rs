//! # Tincan
//!
//! A fine-grained reactive state management library for Rust.
//!
//! Tincan provides two levels of abstraction for managing reactive state:
//!
//! ## Signals (Low-level primitives)
//!
//! Fine-grained reactive primitives for building reactive systems:
//! - `Signal<T>` - Reactive values with combinator support
//! - `Memo<T>` - Computed values that automatically track dependencies
//! - `Effect` - Side effects that run when dependencies change
//!
//! ### Example
//!
//! ```ignore
//! use tincan::Signal;
//!
//! let count = Signal::new(0);
//! let doubled = count.map(|n| n * 2);
//!
//! let _guard = doubled.watch(|value| {
//!     println!("Doubled: {}", value);
//! });
//!
//! count.set(5); // Prints: "Doubled: 10"
//! ```
//!
//! ## Store (High-level state management)
//!
//! Convenient abstractions for managing complex application state:
//! - `Store<T>` - Thread-safe state container
//! - Automatic change detection and notification
//!
//! ### Example
//!
//! ```ignore
//! use tincan::Store;
//!
//! #[derive(Clone)]
//! struct AppState {
//!     count: i32,
//! }
//!
//! let store = Store::new(AppState { count: 0 });
//! store.subscribe(|state| println!("Count: {}", state.count));
//! store.update(|state| state.count += 1);
//! ```

pub mod effect;
pub mod memo;
pub mod runtime;
pub mod signal;

// Re-export main types for convenience
pub use effect::Effect;
pub use memo::Memo;
pub use signal::{Signal, WatchGuard};

pub mod store;
pub use store::Store;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // Basic smoke test
        let signal = Signal::new(0);
        assert_eq!(signal.get(), 0);
        signal.set(42);
        assert_eq!(signal.get(), 42);
    }

    #[test]
    fn combinator_works() {
        let count = Signal::new(5);
        let doubled = count.map(|n| n * 2);
        assert_eq!(doubled.get(), 10);
    }
}
