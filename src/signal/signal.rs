use crate::runtime::ReactiveRuntime;
use std::sync::{Arc, Mutex, RwLock, Weak};

/// A reactive signal that holds a value and notifies subscribers when changed.
#[derive(Clone)]
pub struct Signal<T> {
    value: Arc<RwLock<T>>,
    id: usize,
    _dependencies: Arc<Mutex<Vec<WatchGuard>>>,
}

impl<T: Clone + Send + Sync + 'static> Signal<T> {
    /// Create a new signal with the given initial value.
    pub fn new(initial: T) -> Self {
        let runtime = ReactiveRuntime::current();
        let id = runtime.next_id();

        Self {
            value: Arc::new(RwLock::new(initial)),
            id,
            _dependencies: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get the current value of the signal.
    pub fn get(&self) -> T {
        let runtime = ReactiveRuntime::current();
        runtime.track_read(self.id);
        self.value.read().unwrap().clone()
    }

    /// Set a new value for the signal.
    pub fn set(&self, new_value: T) {
        *self.value.write().unwrap() = new_value;
        let runtime = ReactiveRuntime::current();
        runtime.notify_observers(self.id);
    }

    /// Update the value using a function.
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        let mut value = self.value.write().unwrap();
        f(&mut *value);
        drop(value); // Release the write lock before notifying
        let runtime = ReactiveRuntime::current();
        runtime.notify_observers(self.id);
    }

    /// Read the value with a function without cloning.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let runtime = ReactiveRuntime::current();
        runtime.track_read(self.id);
        let value = self.value.read().unwrap();
        f(&*value)
    }

    /// Get the signal's unique ID.
    pub fn id(&self) -> usize {
        self.id
    }

    /// Watch this signal for changes.
    pub fn watch<F>(&self, callback: F) -> WatchGuard
    where
        F: Fn(T) + Send + Sync + 'static,
    {
        let runtime = ReactiveRuntime::current();
        let observer_id = runtime.next_id();
        let value = Arc::clone(&self.value);
        let callback = Arc::new(callback);
        let callback_clone = Arc::clone(&callback);

        runtime.create_observer(observer_id, move || {
            let val = value.read().unwrap().clone();
            callback_clone(val);
        });

        // Subscribe to this signal
        runtime.with_observer(observer_id, || {
            runtime.track_read(self.id);
        });

        // Call immediately with current value
        let val = self.value.read().unwrap().clone();
        callback(val);

        WatchGuard {
            observer_id,
            runtime: Arc::downgrade(&runtime.inner()),
        }
    }

    /// Create a derived signal by applying a function to this signal's value.
    pub fn map<U, F>(&self, f: F) -> Signal<U>
    where
        U: Clone + Send + Sync + 'static,
        F: Fn(&T) -> U + Send + Sync + 'static,
    {
        let source = self.clone();
        let derived = Signal::new(f(&self.get()));
        let derived_clone = derived.clone();
        let f = Arc::new(f);

        // Watch the source and update the derived signal
        let guard = source.watch(move |value| {
            derived_clone.set(f(&value));
        });

        // Store the watch guard to keep the observer alive
        derived._dependencies.lock().unwrap().push(guard);
        derived
    }

    /// Combine two signals into one using a function.
    pub fn zip<U>(self, other: Signal<U>) -> Signal<(T, U)>
    where
        U: Clone + Send + Sync + 'static,
    {
        let combined = Signal::new((self.get(), other.get()));

        let combined_clone1 = combined.clone();
        let other_clone1 = other.clone();
        let guard1 = self.watch(move |val| {
            let other_val = other_clone1.get();
            combined_clone1.set((val, other_val));
        });

        let combined_clone2 = combined.clone();
        let self_clone = self.clone();
        let guard2 = other.watch(move |val| {
            let self_val = self_clone.get();
            combined_clone2.set((self_val, val));
        });

        // Store the watch guards to keep the observers alive
        combined._dependencies.lock().unwrap().push(guard1);
        combined._dependencies.lock().unwrap().push(guard2);
        combined
    }
}

/// RAII guard for signal watchers.
pub struct WatchGuard {
    observer_id: usize,
    runtime: Weak<RwLock<crate::runtime::RuntimeInner>>,
}

impl Drop for WatchGuard {
    fn drop(&mut self) {
        if let Some(runtime) = self.runtime.upgrade() {
            if let Ok(mut runtime) = runtime.write() {
                runtime.remove_observer(self.observer_id);
            }
        }
    }
}
