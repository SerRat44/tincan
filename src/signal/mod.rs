//! Fine-grained reactive primitives.
//!
//! This module provides the core building blocks for reactive programming:
//! - Signals: Reactive state containers
//! - Memos: Cached computed values
//! - Effects: Side effects that react to changes

mod effect;
mod memo;
mod signal;

pub use effect::{create_effect, Effect};
pub use memo::{create_memo, Memo};
pub use signal::{create_signal, ReadSignal, Signal, WriteSignal};
