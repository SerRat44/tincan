use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};

/// Reactive context for tracking dependencies (thread-local).
struct ReactiveContext {
    current_observer: Option<usize>,
    // Map from signal ID to set of observer IDs that depend on it
    dependencies: HashMap<usize, HashSet<usize>>,
    // Map from observer ID to set of signal IDs it depends on
    observer_deps: HashMap<usize, HashSet<usize>>,
    // Map from observer ID to the effect function
    observers: HashMap<usize, Arc<dyn Fn() + Send + Sync>>,
    // Map from memo ID to dirty state
    memo_dirty: HashMap<usize, bool>,
}

impl ReactiveContext {
    fn new() -> Self {
        Self {
            current_observer: None,
            dependencies: HashMap::new(),
            observer_deps: HashMap::new(),
            observers: HashMap::new(),
            memo_dirty: HashMap::new(),
        }
    }

    fn clear(&mut self) {
        self.current_observer = None;
        self.dependencies.clear();
        self.observer_deps.clear();
        self.observers.clear();
        self.memo_dirty.clear();
    }
}

/// Inner runtime state that can be shared.
pub struct RuntimeInner {
    context: Mutex<ReactiveContext>,
}

impl RuntimeInner {
    fn new() -> Self {
        Self {
            context: Mutex::new(ReactiveContext::new()),
        }
    }

    pub fn remove_observer(&mut self, observer_id: usize) {
        let mut ctx = self.context.lock().unwrap();
        // Remove observer
        ctx.observers.remove(&observer_id);

        // Clear dependencies
        if let Some(old_deps) = ctx.observer_deps.remove(&observer_id) {
            for signal_id in old_deps {
                if let Some(deps) = ctx.dependencies.get_mut(&signal_id) {
                    deps.remove(&observer_id);
                }
            }
        }
    }

    fn clear(&mut self) {
        let mut ctx = self.context.lock().unwrap();
        ctx.clear();
    }
}

/// Hybrid reactive runtime for managing reactive primitives.
///
/// Supports both global runtime (default) and scoped runtimes for isolation.
/// The runtime tracks dependencies between signals, effects, and memos,
/// and manages the reactive graph.
///
/// # Examples
///
/// Using the default global runtime:
///
/// ```
/// use tincan::Signal;
///
/// let signal = Signal::new(42);
/// assert_eq!(signal.get(), 42);
/// ```
///
/// Using scoped runtimes for isolation:
///
/// ```
/// use tincan::runtime::ReactiveRuntime;
/// use tincan::Signal;
///
/// ReactiveRuntime::scope(|| {
///     let signal = Signal::new(0);
///     assert_eq!(signal.get(), 0);
/// });
/// // Runtime and all its state is dropped here
/// ```
pub struct ReactiveRuntime {
    next_id: AtomicUsize,
    inner: Arc<RwLock<RuntimeInner>>,
}

// Thread-local stack for scoped runtimes
thread_local! {
    static RUNTIME_STACK: RefCell<Vec<Arc<ReactiveRuntime>>> = RefCell::new(vec![]);
}

impl ReactiveRuntime {
    /// Create a new isolated runtime.
    ///
    /// This creates a completely independent reactive runtime with its own
    /// dependency graph. Useful for testing or creating isolated contexts.
    fn new() -> Arc<Self> {
        Arc::new(ReactiveRuntime {
            next_id: AtomicUsize::new(0),
            inner: Arc::new(RwLock::new(RuntimeInner::new())),
        })
    }

    /// Run a function with a fresh isolated runtime.
    ///
    /// Useful for testing or creating isolated reactive contexts.
    /// The runtime and all its state is automatically cleaned up when
    /// the function returns.
    ///
    /// # Examples
    ///
    /// ```
    /// use tincan::runtime::ReactiveRuntime;
    /// use tincan::Signal;
    ///
    /// ReactiveRuntime::scope(|| {
    ///     let signal = Signal::new(0);
    ///     assert_eq!(signal.get(), 0);
    /// });
    /// // Runtime and all its state is dropped here
    /// ```
    pub fn scope<F, R>(f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let runtime = Self::new();
        Self::with_runtime(runtime, f)
    }

    /// Get or create the global runtime (fallback).
    ///
    /// This is used as the default runtime when no scoped runtime is active.
    pub fn global() -> Arc<Self> {
        use std::sync::OnceLock;
        static RUNTIME: OnceLock<Arc<ReactiveRuntime>> = OnceLock::new();
        Arc::clone(RUNTIME.get_or_init(|| Self::new()))
    }

    /// Get the current reactive runtime (scoped or global fallback).
    ///
    /// Returns the runtime from the top of the thread-local stack,
    /// or the global runtime if no scoped runtime is active.
    pub fn current() -> Arc<Self> {
        RUNTIME_STACK.with(|stack| {
            stack
                .borrow()
                .last()
                .cloned()
                .unwrap_or_else(|| Self::global())
        })
    }

    /// Run a function with a specific runtime as the current context.
    ///
    /// This pushes the runtime onto the thread-local stack for the duration
    /// of the function execution.
    ///
    /// # Examples
    ///
    /// ```
    /// use tincan::runtime::ReactiveRuntime;
    /// use tincan::Signal;
    ///
    /// let runtime = ReactiveRuntime::new();
    /// ReactiveRuntime::with_runtime(runtime, || {
    ///     let signal = Signal::new(42);
    ///     assert_eq!(signal.get(), 42);
    /// });
    /// ```
    pub fn with_runtime<F, R>(runtime: Arc<Self>, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        RUNTIME_STACK.with(|stack| {
            stack.borrow_mut().push(runtime);
        });

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));

        RUNTIME_STACK.with(|stack| {
            stack.borrow_mut().pop();
        });

        match result {
            Ok(r) => r,
            Err(e) => std::panic::resume_unwind(e),
        }
    }

    /// Clear all observers, dependencies, and state from this runtime.
    ///
    /// Useful for resetting between tests. This removes all tracked
    /// dependencies, observers, and resets the ID counter.
    ///
    /// # Examples
    ///
    /// ```
    /// use tincan::runtime::ReactiveRuntime;
    /// use tincan::Signal;
    ///
    /// let runtime = ReactiveRuntime::new();
    /// ReactiveRuntime::with_runtime(runtime.clone(), || {
    ///     let _signal = Signal::new(42);
    /// });
    ///
    /// runtime.clear(); // Clean up all state
    /// ```
    pub fn clear(&self) {
        let mut inner = self.inner.write().unwrap();
        inner.clear();
        // Reset ID counter
        self.next_id.store(0, Ordering::SeqCst);
    }

    /// Get a reference to the inner runtime state.
    pub fn inner(&self) -> Arc<RwLock<RuntimeInner>> {
        Arc::clone(&self.inner)
    }

    /// Generate the next unique ID for a reactive primitive.
    pub fn next_id(&self) -> usize {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Track a read of a signal by the current observer.
    pub fn track_read(&self, signal_id: usize) {
        let inner = self.inner.read().unwrap();
        let mut ctx = inner.context.lock().unwrap();
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
    }

    /// Notify all observers that depend on a signal.
    pub fn notify_observers(&self, signal_id: usize) {
        let inner = self.inner.read().unwrap();
        let observers = {
            let ctx = inner.context.lock().unwrap();
            ctx.dependencies
                .get(&signal_id)
                .map(|obs| obs.iter().copied().collect::<Vec<_>>())
        };

        if let Some(observers) = observers {
            for observer_id in observers {
                self.mark_observer_dirty(observer_id);
            }
        }
    }

    /// Mark an observer (memo or effect) as dirty and propagate to dependents.
    fn mark_observer_dirty(&self, observer_id: usize) {
        let inner = self.inner.read().unwrap();
        let mut ctx = inner.context.lock().unwrap();

        // If it's a memo, mark it as dirty
        if ctx.memo_dirty.contains_key(&observer_id) {
            let already_dirty = ctx.memo_dirty.get(&observer_id).copied().unwrap_or(false);
            if !already_dirty {
                ctx.memo_dirty.insert(observer_id, true);

                // Recursively mark dependents as dirty
                let dependents = ctx
                    .dependencies
                    .get(&observer_id)
                    .map(|deps| deps.iter().copied().collect::<Vec<_>>());

                drop(ctx);
                drop(inner);

                if let Some(dependents) = dependents {
                    for dependent_id in dependents {
                        self.mark_observer_dirty(dependent_id);
                    }
                }
                return;
            }
            return;
        }

        // If it's an effect, collect it for execution
        let effect = ctx.observers.get(&observer_id).cloned();
        drop(ctx);
        drop(inner);

        if let Some(effect) = effect {
            effect();
        }
    }

    /// Run a function as an observer, tracking all reads.
    pub fn create_observer<F>(&self, observer_id: usize, f: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let inner = self.inner.read().unwrap();
        let mut ctx = inner.context.lock().unwrap();

        // Clear old dependencies for this observer
        if let Some(old_deps) = ctx.observer_deps.remove(&observer_id) {
            for signal_id in old_deps {
                if let Some(deps) = ctx.dependencies.get_mut(&signal_id) {
                    deps.remove(&observer_id);
                }
            }
        }
        // Store the observer effect
        ctx.observers.insert(observer_id, Arc::new(f));
    }

    /// Run a function with a specific observer as the current context.
    pub fn with_observer<F, R>(&self, observer_id: usize, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let inner = self.inner.read().unwrap();
        let prev = {
            let mut ctx = inner.context.lock().unwrap();
            ctx.current_observer.replace(observer_id)
        };

        let result = f();

        let mut ctx = inner.context.lock().unwrap();
        ctx.current_observer = prev;

        result
    }

    /// Register a memo and mark it as dirty initially.
    pub fn register_memo(&self, memo_id: usize) {
        let inner = self.inner.read().unwrap();
        let mut ctx = inner.context.lock().unwrap();
        ctx.memo_dirty.insert(memo_id, true);
    }

    /// Check if a memo is dirty (needs recomputation).
    pub fn is_memo_dirty(&self, memo_id: usize) -> bool {
        let inner = self.inner.read().unwrap();
        let ctx = inner.context.lock().unwrap();
        ctx.memo_dirty.get(&memo_id).copied().unwrap_or(true)
    }

    /// Mark a memo as clean (after recomputation).
    pub fn mark_memo_clean(&self, memo_id: usize) {
        let inner = self.inner.read().unwrap();
        let mut ctx = inner.context.lock().unwrap();
        ctx.memo_dirty.insert(memo_id, false);
    }
}
