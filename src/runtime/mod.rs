//! Runtime support for reactive primitives.
//!
//! This module provides the infrastructure for dependency tracking,
//! reactive graph management, and execution contexts.

mod context;

pub(crate) use context::{ReactiveRuntime, RuntimeInner};
