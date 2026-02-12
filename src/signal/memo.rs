use crate::runtime::ReactiveRuntime;
use std::sync::{Arc, RwLock};

/// A memoized computed value that automatically tracks dependencies.
///
/// Memos only recompute when their dependencies change.
#[derive(Clone)]
pub struct Memo<T> {
    compute: Arc<dyn Fn() -> T + Send + Sync>,
    cached: Arc<RwLock<Option<T>>>,
    id: usize,
}

impl<T: Clone + 'static> Memo<T> {
    /// Create a new memo with the given computation function.
    fn new<F>(compute: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        let runtime = ReactiveRuntime::current();
        let id = runtime.next_id();

        // Register this as a memo with the runtime
        runtime.register_memo(id);

        Self {
            compute: Arc::new(compute),
            cached: Arc::new(RwLock::new(None)),
            id,
        }
    }

    /// Get the current value, recomputing if necessary.
    pub fn get(&self) -> T {
        let runtime = ReactiveRuntime::current();

        // Track this read in the reactive context
        runtime.track_read(self.id);

        // Check if we need to recompute
        if runtime.is_memo_dirty(self.id) {
            // Recompute within observer context to track dependencies
            let value = runtime.with_observer(self.id, || (self.compute)());
            *self.cached.write().unwrap() = Some(value.clone());
            runtime.mark_memo_clean(self.id);
            value
        } else {
            // Return cached value
            self.cached.read().unwrap().as_ref().unwrap().clone()
        }
    }
}

/// Create a new memoized computation.
///
/// # Example
///
/// ```ignore
/// let (count, set_count) = create_signal(5);
/// let doubled = create_memo(move || count.get() * 2);
/// assert_eq!(doubled.get(), 10);
/// ```
pub fn create_memo<T, F>(compute: F) -> Memo<T>
where
    T: Clone + 'static,
    F: Fn() -> T + Send + Sync + 'static,
{
    Memo::new(compute)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::create_signal;

    #[test]
    fn memo_basic() {
        let (count, set_count) = create_signal(5);
        let doubled = create_memo(move || count.get() * 2);

        assert_eq!(doubled.get(), 10);

        set_count.set(10);
        assert_eq!(doubled.get(), 20);
    }
}
