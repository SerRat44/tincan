use crate::runtime::ReactiveRuntime;
use std::sync::{Arc, RwLock};

/// A reactive signal that holds a value and notifies subscribers when changed.
#[derive(Clone)]
pub struct Signal<T> {
    value: Arc<RwLock<T>>,
    id: usize,
}

impl<T: Clone> Signal<T> {
    /// Create a new signal with the given initial value.
    pub fn new(initial: T) -> Self {
        let runtime = ReactiveRuntime::current();
        let id = runtime.next_id();

        Self {
            value: Arc::new(RwLock::new(initial)),
            id,
        }
    }

    /// Get the current value of the signal.
    /// This tracks the read in the current reactive context.
    pub fn get(&self) -> T {
        let runtime = ReactiveRuntime::current();
        runtime.track_read(self.id);
        self.value.read().unwrap().clone()
    }

    /// Set a new value for the signal.
    /// This will notify all subscribers.
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

    /// Get the signal's unique ID.
    pub fn id(&self) -> usize {
        self.id
    }
}

/// Read-only handle to a signal.
#[derive(Clone)]
pub struct ReadSignal<T>(Signal<T>);

impl<T: Clone> ReadSignal<T> {
    pub fn get(&self) -> T {
        self.0.get()
    }
}

/// Write-only handle to a signal.
#[derive(Clone)]
pub struct WriteSignal<T>(Signal<T>);

impl<T: Clone> WriteSignal<T> {
    pub fn set(&self, value: T) {
        self.0.set(value);
    }

    pub fn update(&self, f: impl FnOnce(&mut T)) {
        self.0.update(f);
    }
}

/// Create a new signal with split read/write handles.
///
/// # Example
///
/// ```ignore
/// let (count, set_count) = create_signal(0);
/// assert_eq!(count.get(), 0);
/// set_count(42);
/// assert_eq!(count.get(), 42);
/// ```
pub fn create_signal<T: Clone>(initial: T) -> (ReadSignal<T>, WriteSignal<T>) {
    let signal = Signal::new(initial);
    (ReadSignal(signal.clone()), WriteSignal(signal))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signal_get_set() {
        let (count, set_count) = create_signal(0);
        assert_eq!(count.get(), 0);
        set_count.set(42);
        assert_eq!(count.get(), 42);
    }

    #[test]
    fn signal_update() {
        let (count, set_count) = create_signal(10);
        set_count.update(|n| *n += 5);
        assert_eq!(count.get(), 15);
    }
}
