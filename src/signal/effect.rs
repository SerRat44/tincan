use crate::runtime::ReactiveRuntime;
use std::sync::Arc;

/// A side effect that runs when its dependencies change.
pub struct Effect {
    run: Arc<dyn Fn() + Send + Sync>,
    id: usize,
}

impl Effect {
    fn new<F>(effect: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        let runtime = ReactiveRuntime::current();
        let id = runtime.next_id();
        let effect = Arc::new(effect);
        let effect_clone = effect.clone();

        // Register the effect with the runtime
        runtime.create_observer(id, move || {
            effect_clone();
        });

        // Run immediately within the observer context to track dependencies
        runtime.with_observer(id, || {
            effect();
        });

        Self { run: effect, id }
    }

    /// Manually trigger the effect.
    pub fn run(&self) {
        (self.run)();
    }
}

/// Create a new effect that runs when dependencies change.
///
/// The effect runs immediately and then again whenever any signal
/// it reads changes.
///
/// # Example
///
/// ```ignore
/// let (count, set_count) = create_signal(0);
///
/// create_effect(move || {
///     println!("Count is: {}", count.get());
/// });
/// ```
pub fn create_effect<F>(effect: F) -> Effect
where
    F: Fn() + Send + Sync + 'static,
{
    Effect::new(effect)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn effect_runs_immediately() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        create_effect(move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
