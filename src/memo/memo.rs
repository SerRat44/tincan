use crate::runtime::ReactiveRuntime;
use std::sync::{Arc, RwLock};

/// A memoized computed value that automatically tracks dependencies.
#[derive(Clone)]
pub struct Memo<T> {
    cached_value: Arc<RwLock<Option<T>>>,
    compute: Arc<dyn Fn() -> T + Send + Sync>,
    id: usize,
}

impl<T: Clone + 'static> Memo<T> {
    /// Create a new memo with the given computation function.
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
