//! # Tincan
//!
//! A fine-grained reactive state management library for Rust.
//!
//! Tincan provides two levels of abstraction for managing reactive state:
//!
//! ## Signals (Low-level primitives)
//!
//! Fine-grained reactive primitives for building reactive systems:
//! - `Signal<T>` - Reactive values that notify dependents when changed
//! - `Memo<T>` - Computed values that automatically track dependencies
//! - `Effect` - Side effects that run when dependencies change
//!
//! ## Store (High-level state management)
//!
//! Convenient abstractions for managing complex application state:
//! - `Store<T>` - Thread-safe state container with derived values
//! - Automatic change detection and notification
//! - Middleware support for logging, persistence, etc.

pub mod runtime;
pub mod signal;
pub mod store;

// Re-export main types for convenience
pub use signal::{create_effect, create_memo, create_signal, Effect, Memo, Signal};
pub use store::Store;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // Basic smoke test
        let (signal, set_signal) = create_signal(0);
        assert_eq!(signal.get(), 0);
        set_signal.set(42);
        assert_eq!(signal.get(), 42);
    }
}
