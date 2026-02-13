use crate::runtime::{ReactiveRuntime, RuntimeInner};
use std::sync::{Arc, RwLock, Weak};

/// A side effect that runs when its dependencies change.
pub struct Effect {
    id: usize,
    runtime: Weak<RwLock<RuntimeInner>>,
}

impl Effect {
    /// Create a new effect that runs when dependencies change.
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
