use crate::runtime::ReactiveRuntime;
use std::sync::{Arc, RwLock};

/// A memoized computed value that automatically tracks dependencies.
///
/// Memos only recompute when their dependencies change, making them
/// efficient for expensive computations.
#[derive(Clone)]
pub struct Memo<T> {
    cached_value: Arc<RwLock<Option<T>>>,
    compute: Arc<dyn Fn() -> T + Send + Sync>,
    id: usize,
}

impl<T: Clone + 'static> Memo<T> {
    /// Create a new memo with the given computation function.
    ///
    /// The computation runs immediately to establish initial dependencies.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let count = Signal::new(5);
    /// let doubled = Memo::new(move || count.get() * 2);
    /// assert_eq!(doubled.get(), 10);
    /// ```
    pub fn new<F>(compute: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        let runtime = ReactiveRuntime::current();
        let id = runtime.next_id();

        // Register this as a memo with the runtime
        runtime.register_memo(id);

        Self {
            cached_value: Arc::new(RwLock::new(None)),
            compute: Arc::new(compute),
            id,
        }
    }

    /// Get the current value, recomputing if necessary.
    ///
    /// This tracks the read in the reactive context and recomputes
    /// if any dependencies have changed since the last call.
    pub fn get(&self) -> T {
        let runtime = ReactiveRuntime::current();

        // Track this read in the reactive context
        runtime.track_read(self.id);

        // Check if we need to recompute
        if runtime.is_memo_dirty(self.id) {
            // Recompute within observer context to track dependencies
            let value = runtime.with_observer(self.id, || (self.compute)());
            *self.cached_value.write().unwrap() = Some(value.clone());
            runtime.mark_memo_clean(self.id);
            value
        } else {
            // Return cached value
            self.cached_value.read().unwrap().as_ref().unwrap().clone()
        }
    }

    /// Read the memoized value with a function without cloning.
    ///
    /// The read is still tracked for reactivity.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let text = Memo::new(|| expensive_string_computation());
    /// let len = text.with(|s| s.len());
    /// ```
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let runtime = ReactiveRuntime::current();
        runtime.track_read(self.id);

        if runtime.is_memo_dirty(self.id) {
            let value = runtime.with_observer(self.id, || (self.compute)());
            *self.cached_value.write().unwrap() = Some(value.clone());
            runtime.mark_memo_clean(self.id);
            let cached = self.cached_value.read().unwrap();
            f(cached.as_ref().unwrap())
        } else {
            let cached = self.cached_value.read().unwrap();
            f(cached.as_ref().unwrap())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::Signal;

    #[test]
    fn memo_basic() {
        let count = Signal::new(5);
        let count_clone = count.clone();
        let doubled = Memo::new(move || count_clone.get() * 2);

        assert_eq!(doubled.get(), 10);

        count.set(10);
        assert_eq!(doubled.get(), 20);
    }

    #[test]
    fn memo_with() {
        let count = Signal::new(5);
        let count_clone = count.clone();
        let text = Memo::new(move || format!("Count: {}", count_clone.get()));

        let len = text.with(|s| s.len());
        assert!(len > 0);
    }
}
