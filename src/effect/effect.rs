use crate::runtime::{ReactiveRuntime, RuntimeInner};
use std::sync::{Arc, RwLock, Weak};

/// A side effect that runs when its dependencies change.
///
/// Effects automatically track signal reads and re-run when those signals change.
/// The effect runs immediately on creation to establish initial dependencies.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use tincan::{Effect, Signal};
/// use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
///
/// let count = Signal::new(0);
/// let counter = Arc::new(AtomicUsize::new(0));
/// let counter_clone = counter.clone();
///
/// let _effect = Effect::new({
///     let count = count.clone();
///     move || {
///         let _ = count.get();
///         counter_clone.fetch_add(1, Ordering::SeqCst);
///     }
/// });
///
/// // Effect runs immediately
/// assert_eq!(counter.load(Ordering::SeqCst), 1);
/// ```
pub struct Effect {
    id: usize,
    runtime: Weak<RwLock<RuntimeInner>>,
}

impl Effect {
    /// Create a new effect that runs when dependencies change.
    ///
    /// The effect function runs immediately to establish initial dependencies,
    /// then re-runs whenever any tracked signals change.
    ///
    /// # Examples
    ///
    /// ```
    /// use tincan::{Effect, Signal};
    /// use std::sync::{Arc, atomic::{AtomicI32, Ordering}};
    /// use std::thread;
    /// use std::time::Duration;
    ///
    /// let signal = Signal::new(5);
    /// let last_value = Arc::new(AtomicI32::new(0));
    /// let last_value_clone = last_value.clone();
    ///
    /// let _effect = Effect::new({
    ///     let signal = signal.clone();
    ///     move || {
    ///         let val = signal.get();
    ///         last_value_clone.store(val, Ordering::SeqCst);
    ///     }
    /// });
    ///
    /// assert_eq!(last_value.load(Ordering::SeqCst), 5);
    ///
    /// signal.set(10);
    /// thread::sleep(Duration::from_millis(10));
    /// assert_eq!(last_value.load(Ordering::SeqCst), 10);
    /// ```
    pub fn new<F>(effect: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        let runtime = ReactiveRuntime::current();
        let id = runtime.next_id();
        let effect = Arc::new(effect);
        let effect_clone = Arc::clone(&effect);

        // Register the effect with the runtime
        runtime.create_observer(id, move || {
            effect_clone();
        });

        // Run immediately within the observer context to track dependencies
        runtime.with_observer(id, || {
            effect();
        });

        Self {
            id,
            runtime: Arc::downgrade(&runtime.inner()),
        }
    }
}

impl Drop for Effect {
    fn drop(&mut self) {
        if let Some(runtime) = self.runtime.upgrade() {
            if let Ok(mut runtime) = runtime.write() {
                runtime.remove_observer(self.id);
            }
        }
    }
}
