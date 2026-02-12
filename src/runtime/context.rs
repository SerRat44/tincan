use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Global reactive runtime for managing reactive primitives.
///
/// This handles:
/// - Dependency tracking
/// - Reactive graph management
/// - Batched updates
/// - Effect scheduling
pub struct ReactiveRuntime {
    next_id: AtomicUsize,
}

impl ReactiveRuntime {
    /// Get or create the current reactive runtime.
    pub fn current() -> &'static Self {
        // Use a simple static instance for ID generation
        static RUNTIME: ReactiveRuntime = ReactiveRuntime {
            next_id: AtomicUsize::new(0),
        };
        &RUNTIME
    }

    /// Generate the next unique ID for a reactive primitive.
    pub fn next_id(&self) -> usize {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Track a read of a signal by the current observer.
    pub fn track_read(&self, signal_id: usize) {
        CONTEXT.with(|ctx| {
            let mut ctx = ctx.borrow_mut();
            if let Some(current_observer) = ctx.current_observer {
                // Add dependency: signal -> observer
                ctx.dependencies
                    .entry(signal_id)
                    .or_insert_with(HashSet::new)
                    .insert(current_observer);
                // Track that this observer depends on this signal
                ctx.observer_deps
                    .entry(current_observer)
                    .or_insert_with(HashSet::new)
                    .insert(signal_id);
            }
        });
    }

    /// Notify all observers that depend on a signal.
    pub fn notify_observers(&self, signal_id: usize) {
        CONTEXT.with(|ctx| {
            let ctx_ref = ctx.borrow();
            if let Some(observers) = ctx_ref.dependencies.get(&signal_id) {
                // Collect observers to avoid borrow issues
                let observers: Vec<usize> = observers.iter().copied().collect();
                drop(ctx_ref);

                for observer_id in observers {
                    self.mark_observer_dirty(observer_id);
                }
            }
        });
    }

    /// Mark an observer (memo or effect) as dirty and propagate to dependents.
    fn mark_observer_dirty(&self, observer_id: usize) {
        let effects_to_run = CONTEXT.with(|ctx| {
            let ctx_ref = ctx.borrow();
            let mut effects_to_run = Vec::new();

            // If it's a memo, mark it as dirty
            if ctx_ref.memo_dirty.borrow().contains_key(&observer_id) {
                let already_dirty = ctx_ref
                    .memo_dirty
                    .borrow()
                    .get(&observer_id)
                    .copied()
                    .unwrap_or(false);
                if !already_dirty {
                    ctx_ref.memo_dirty.borrow_mut().insert(observer_id, true);

                    // Recursively mark dependents as dirty
                    if let Some(dependents) = ctx_ref.dependencies.get(&observer_id) {
                        let dependents: Vec<usize> = dependents.iter().copied().collect();
                        drop(ctx_ref);
                        for dependent_id in dependents {
                            self.mark_observer_dirty(dependent_id);
                        }
                        return effects_to_run;
                    }
                }
            }

            // If it's an effect, collect it for execution
            if let Some(effect) = ctx_ref.observers.get(&observer_id) {
                effects_to_run.push(effect.clone());
            }

            effects_to_run
        });

        // Execute effects outside of the borrow
        for effect in effects_to_run {
            effect();
        }
    }

    /// Run a function as an observer, tracking all reads.
    pub fn create_observer<F>(&self, observer_id: usize, f: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        CONTEXT.with(|ctx| {
            let mut ctx = ctx.borrow_mut();
            // Clear old dependencies for this observer
            if let Some(old_deps) = ctx.observer_deps.remove(&observer_id) {
                for signal_id in old_deps {
                    if let Some(deps) = ctx.dependencies.get_mut(&signal_id) {
                        deps.remove(&observer_id);
                    }
                }
            }
            // Store the observer effect
            ctx.observers.insert(observer_id, Arc::new(Box::new(f)));
        });
    }

    /// Run a function with a specific observer as the current context.
    pub fn with_observer<F, R>(&self, observer_id: usize, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        CONTEXT.with(|ctx| {
            let prev = ctx.borrow_mut().current_observer.replace(observer_id);
            let result = f();
            ctx.borrow_mut().current_observer = prev;
            result
        })
    }

    /// Register a memo and mark it as clean initially.
    pub fn register_memo(&self, memo_id: usize) {
        CONTEXT.with(|ctx| {
            ctx.borrow().memo_dirty.borrow_mut().insert(memo_id, true);
        });
    }

    /// Check if a memo is dirty (needs recomputation).
    pub fn is_memo_dirty(&self, memo_id: usize) -> bool {
        CONTEXT.with(|ctx| {
            ctx.borrow()
                .memo_dirty
                .borrow()
                .get(&memo_id)
                .copied()
                .unwrap_or(true)
        })
    }

    /// Mark a memo as clean (after recomputation).
    pub fn mark_memo_clean(&self, memo_id: usize) {
        CONTEXT.with(|ctx| {
            ctx.borrow().memo_dirty.borrow_mut().insert(memo_id, false);
        });
    }
}

// Thread-local reactive context for tracking dependencies.
thread_local! {
    static CONTEXT: RefCell<ReactiveContext> = RefCell::new(ReactiveContext::new());
}

struct ReactiveContext {
    current_observer: Option<usize>,
    // Map from signal ID to set of observer IDs that depend on it
    dependencies: HashMap<usize, HashSet<usize>>,
    // Map from observer ID to set of signal IDs it depends on
    observer_deps: HashMap<usize, HashSet<usize>>,
    // Map from observer ID to the effect function
    observers: HashMap<usize, Arc<Box<dyn Fn()>>>,
    // Map from memo ID to dirty state
    memo_dirty: RefCell<HashMap<usize, bool>>,
}

impl ReactiveContext {
    fn new() -> Self {
        Self {
            current_observer: None,
            dependencies: HashMap::new(),
            observer_deps: HashMap::new(),
            observers: HashMap::new(),
            memo_dirty: RefCell::new(HashMap::new()),
        }
    }
}
