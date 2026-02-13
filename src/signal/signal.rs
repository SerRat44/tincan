use crate::runtime::ReactiveRuntime;
use crate::Effect;
use std::sync::{Arc, RwLock, Weak};

/// A reactive signal that holds a value and notifies subscribers when changed.
///
/// Signals are the core primitive for building reactive applications. They automatically
/// track dependencies and notify watchers when values change.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use tincan::Signal;
///
/// let count = Signal::new(0);
/// assert_eq!(count.get(), 0);
///
/// count.set(42);
/// assert_eq!(count.get(), 42);
/// ```
///
/// Derived signals with map:
///
/// ```
/// use tincan::Signal;
/// use std::thread;
/// use std::time::Duration;
///
/// let celsius = Signal::new(0);
/// let fahrenheit = celsius.map(|c| c * 9 / 5 + 32);
///
/// assert_eq!(fahrenheit.get(), 32);
///
/// celsius.set(100);
/// thread::sleep(Duration::from_millis(10));
/// assert_eq!(fahrenheit.get(), 212);
/// ```
#[derive(Clone)]
pub struct Signal<T> {
    value: Arc<RwLock<T>>,
    id: usize,
    /// Internal effects that keep derived signals (from map/zip) alive
    _effects: Arc<Vec<Effect>>,
}

impl<T: Clone + Send + Sync + 'static> Signal<T> {
    /// Create a new signal with the given initial value.
    ///
    /// # Examples
    ///
    /// ```
    /// use tincan::Signal;
    ///
    /// let signal = Signal::new(42);
    /// assert_eq!(signal.get(), 42);
    /// ```
    pub fn new(initial: T) -> Self {
        let runtime = ReactiveRuntime::current();
        let id = runtime.next_id();

        Self {
            value: Arc::new(RwLock::new(initial)),
            id,
            _effects: Arc::new(Vec::new()),
        }
    }

    /// Get the current value of the signal.
    ///
    /// This tracks the read in the current reactive context, allowing
    /// effects and memos to automatically subscribe to changes.
    ///
    /// # Examples
    ///
    /// ```
    /// use tincan::Signal;
    ///
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
    /// # Examples
    ///
    /// ```
    /// use tincan::Signal;
    ///
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
    /// # Examples
    ///
    /// ```
    /// use tincan::Signal;
    ///
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
    /// # Examples
    ///
    /// ```
    /// use tincan::Signal;
    ///
    /// let text = Signal::new(String::from("hello"));
    /// let len = text.with(|s| s.len());
    /// assert_eq!(len, 5);
    /// ```
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let runtime = ReactiveRuntime::current();
        runtime.track_read(self.id);
        let value = self.value.read().unwrap();
        f(&*value)
    }

    /// Get the signal's unique ID.
    ///
    /// This is mainly used internally by the reactivity system.
    pub fn id(&self) -> usize {
        self.id
    }

    /// Watch this signal for changes.
    ///
    /// Returns a guard that will unsubscribe when dropped.
    /// The callback is called immediately with the current value.
    ///
    /// # Examples
    ///
    /// ```
    /// use tincan::Signal;
    /// use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
    ///
    /// let count = Signal::new(0);
    /// let calls = Arc::new(AtomicUsize::new(0));
    /// let calls_clone = calls.clone();
    ///
    /// let _guard = count.watch(move |_| {
    ///     calls_clone.fetch_add(1, Ordering::SeqCst);
    /// });
    ///
    /// // Called immediately
    /// assert_eq!(calls.load(Ordering::SeqCst), 1);
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
    /// The derived signal automatically updates when the source changes.
    /// The watcher is kept alive as long as the derived signal exists.
    ///
    /// # Examples
    ///
    /// ```
    /// use tincan::Signal;
    /// use std::thread;
    /// use std::time::Duration;
    ///
    /// let count = Signal::new(5);
    /// let doubled = count.map(|n| n * 2);
    ///
    /// assert_eq!(doubled.get(), 10);
    ///
    /// count.set(10);
    /// thread::sleep(Duration::from_millis(10));
    /// assert_eq!(doubled.get(), 20);
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

        // Create an effect that updates the derived signal
        // This automatically tracks the dependency and cleans up when dropped
        let effect = Effect::new(move || {
            let val = source.get();
            derived_clone.set(f(&val));
        });

        // Store the effect to keep it alive
        Signal {
            value: derived.value,
            id: derived.id,
            _effects: Arc::new(vec![effect]),
        }
    }

    /// Combine two signals into one.
    ///
    /// The combined signal updates when either source changes.
    /// The watchers are kept alive as long as the combined signal exists.
    ///
    /// # Examples
    ///
    /// ```
    /// use tincan::Signal;
    /// use std::thread;
    /// use std::time::Duration;
    ///
    /// let width = Signal::new(10);
    /// let height = Signal::new(5);
    /// let area = width.clone().zip(height.clone()).map(|(w, h)| w * h);
    ///
    /// assert_eq!(area.get(), 50);
    ///
    /// width.set(20);
    /// thread::sleep(Duration::from_millis(10));
    /// assert_eq!(area.get(), 100);
    /// ```
    pub fn zip<U>(self, other: Signal<U>) -> Signal<(T, U)>
    where
        U: Clone + Send + Sync + 'static,
    {
        let combined = Signal::new((self.get(), other.get()));

        // Create effect that tracks both signals
        let self_clone = self.clone();
        let other_clone = other.clone();
        let combined_clone = combined.clone();
        let effect = Effect::new(move || {
            let self_val = self_clone.get();
            let other_val = other_clone.get();
            combined_clone.set((self_val, other_val));
        });

        // Store the effect to keep it alive
        Signal {
            value: combined.value,
            id: combined.id,
            _effects: Arc::new(vec![effect]),
        }
    }
}

/// RAII guard for signal watchers.
///
/// When dropped, automatically unsubscribes the watcher.
/// You typically don't create this directly - it's returned by [`Signal::watch`].
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
