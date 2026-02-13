use crate::runtime::ReactiveRuntime;
use std::sync::{Arc, RwLock, Weak};

/// A reactive signal that holds a value and notifies subscribers when changed.
///
/// Signals automatically track dependencies and notify watchers when values change.
/// This is the core primitive for building reactive applications.
#[derive(Clone)]
pub struct Signal<T> {
    value: Arc<RwLock<T>>,
    id: usize,
}

impl<T: Clone + Send + Sync + 'static> Signal<T> {
    /// Create a new signal with the given initial value.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let count = Signal::new(0);
    /// assert_eq!(count.get(), 0);
    /// ```
    pub fn new(initial: T) -> Self {
        let runtime = ReactiveRuntime::current();
        let id = runtime.next_id();

        Self {
            value: Arc::new(RwLock::new(initial)),
            id,
        }
    }

    /// Get the current value of the signal.
    ///
    /// This tracks the read in the current reactive context, allowing
    /// effects and memos to automatically subscribe to changes.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let count = Signal::new(42);
    /// assert_eq!(count.get(), 42);
    /// ```
    pub fn get(&self) -> T {
        let runtime = ReactiveRuntime::current();
        runtime.track_read(self.id);
        self.value.read().unwrap().clone()
    }

    /// Set a new value for the signal.
    ///
    /// This will notify all subscribers (effects, memos, watchers).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let count = Signal::new(0);
    /// count.set(42);
    /// assert_eq!(count.get(), 42);
    /// ```
    pub fn set(&self, new_value: T) {
        *self.value.write().unwrap() = new_value;
        let runtime = ReactiveRuntime::current();
        runtime.notify_observers(self.id);
    }

    /// Update the value using a function.
    ///
    /// This is useful for making modifications based on the current value.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let count = Signal::new(10);
    /// count.update(|n| *n += 5);
    /// assert_eq!(count.get(), 15);
    /// ```
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        let mut value = self.value.write().unwrap();
        f(&mut *value);
        drop(value); // Release the write lock before notifying
        let runtime = ReactiveRuntime::current();
        runtime.notify_observers(self.id);
    }

    /// Read the value with a function without cloning.
    ///
    /// The read is still tracked for reactivity.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let text = Signal::new(String::from("hello"));
    /// let len = text.with(|s| s.len());
    /// ```
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
    ///
    /// Returns a guard that will unsubscribe when dropped.
    /// The callback is called immediately with the current value.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let count = Signal::new(0);
    /// let _guard = count.watch(|value| {
    ///     println!("Count changed to: {}", value);
    /// });
    /// count.set(5); // Prints: "Count changed to: 5"
    /// ```
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
    ///
    /// The derived signal updates automatically when the source changes.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let count = Signal::new(5);
    /// let doubled = count.map(|n| n * 2);
    /// assert_eq!(doubled.get(), 10);
    /// ```
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
        std::mem::forget(source.watch(move |value| {
            derived_clone.set(f(&value));
        }));

        derived
    }

    /// Combine two signals into one using a function.
    ///
    /// The result updates when either source signal changes.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let first = Signal::new(1);
    /// let second = Signal::new(2);
    /// let sum = Signal::zip(first, second).map(|(a, b)| a + b);
    /// ```
    pub fn zip<U>(self, other: Signal<U>) -> Signal<(T, U)>
    where
        U: Clone + Send + Sync + 'static,
    {
        let combined = Signal::new((self.get(), other.get()));

        let combined_clone1 = combined.clone();
        let other_clone1 = other.clone();
        std::mem::forget(self.watch(move |val| {
            let other_val = other_clone1.get();
            combined_clone1.set((val, other_val));
        }));

        let combined_clone2 = combined.clone();
        let self_clone = self.clone();
        std::mem::forget(other.watch(move |val| {
            let self_val = self_clone.get();
            combined_clone2.set((self_val, val));
        }));

        combined
    }
}

/// RAII guard for signal watchers.
///
/// When dropped, automatically unsubscribes the watcher.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signal_get_set() {
        let count = Signal::new(0);
        assert_eq!(count.get(), 0);
        count.set(42);
        assert_eq!(count.get(), 42);
    }

    #[test]
    fn signal_update() {
        let count = Signal::new(10);
        count.update(|n| *n += 5);
        assert_eq!(count.get(), 15);
    }

    #[test]
    fn signal_with() {
        let text = Signal::new(String::from("hello"));
        let len = text.with(|s| s.len());
        assert_eq!(len, 5);
    }

    #[test]
    fn signal_map() {
        let count = Signal::new(5);
        let doubled = count.map(|n| n * 2);
        assert_eq!(doubled.get(), 10);

        count.set(10);
        // Give the update a moment to propagate
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert_eq!(doubled.get(), 20);
    }

    #[test]
    fn signal_watch() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let count = Signal::new(0);
        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = Arc::clone(&call_count);

        let _guard = count.watch(move |_value| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
        });

        // Should be called once immediately
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        count.set(5);
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }
}
