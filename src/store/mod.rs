//! High-level state management with stores.
//!
//! Stores provide a convenient abstraction for managing complex application state
//! with automatic reactivity, derived values, and middleware support.

mod store;

pub use store::Store;
