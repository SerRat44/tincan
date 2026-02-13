use crate::runtime::{ReactiveRuntime, RuntimeInner};
use std::sync::{Arc, RwLock, Weak};

/// A side effect that runs when its dependencies change.
///
/// Effects automatically track signal reads and re-run when those signals change.
pub struct Effect {
    id: usize,
    runtime: Weak<RwLock<RuntimeInner>>,
}

impl Effect {
    /// Create a new effect that runs when dependencies change.
    ///
    /// The effect runs immediately to establish initial dependencies.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let count = Signal::new(0);
    /// let _effect = Effect::new(move || {
    ///     println!("Count is: {}", count.get());
    /// });
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::Signal;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn effect_runs_immediately() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);

        let _effect = Effect::new(move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn effect_tracks_signals() {
        let count = Signal::new(0);
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = Arc::clone(&counter);
        let count_clone = count.clone();

        let _effect = Effect::new(move || {
            let _ = count_clone.get();
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        // Runs immediately
        assert_eq!(counter.load(Ordering::SeqCst), 1);

        count.set(5);
        std::thread::sleep(std::time::Duration::from_millis(10));

        // Should run again after signal changes
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }
}
