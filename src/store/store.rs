use std::sync::{Arc, RwLock};

type Subscriber<T> = Box<dyn Fn(&T) + Send + Sync>;

/// A thread-safe store for managing application state.
///
/// Stores provide a higher-level abstraction over signals for managing
/// complex state with automatic change detection.
pub struct Store<T> {
    state: Arc<RwLock<T>>,
    subscribers: Arc<RwLock<Vec<Subscriber<T>>>>,
}

impl<T: Clone> Store<T> {
    /// Create a new store with the given initial state.
    pub fn new(initial: T) -> Self {
        Self {
            state: Arc::new(RwLock::new(initial)),
            subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get a clone of the current state.
    pub fn get(&self) -> T {
        self.state.read().unwrap().clone()
    }

    /// Update the state using a function.
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        {
            let mut state = self.state.write().unwrap();
            f(&mut *state);
        }
        self.notify();
    }

    /// Set a new state value.
    pub fn set(&self, new_state: T) {
        *self.state.write().unwrap() = new_state;
        self.notify();
    }

    /// Subscribe to state changes.
    ///
    /// The callback will be called whenever the state is updated.
    pub fn subscribe<F>(&self, callback: F)
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        self.subscribers.write().unwrap().push(Box::new(callback));
    }

    /// Notify all subscribers of a state change.
    fn notify(&self) {
        let state = self.state.read().unwrap();
        let subscribers = self.subscribers.read().unwrap();
        for subscriber in subscribers.iter() {
            subscriber(&*state);
        }
    }

    /// Read state without triggering reactivity.
    pub fn read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let state = self.state.read().unwrap();
        f(&*state)
    }
}

impl<T: Clone> Clone for Store<T> {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            subscribers: Arc::clone(&self.subscribers),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Clone, Debug, PartialEq)]
    struct AppState {
        count: usize,
        name: String,
    }

    #[test]
    fn store_get_set() {
        let store = Store::new(AppState {
            count: 0,
            name: "test".to_string(),
        });

        assert_eq!(store.get().count, 0);

        store.set(AppState {
            count: 42,
            name: "updated".to_string(),
        });

        assert_eq!(store.get().count, 42);
        assert_eq!(store.get().name, "updated");
    }

    #[test]
    fn store_update() {
        let store = Store::new(AppState {
            count: 0,
            name: "test".to_string(),
        });

        store.update(|state| {
            state.count += 10;
        });

        assert_eq!(store.get().count, 10);
    }

    #[test]
    fn store_subscribe() {
        let store = Store::new(AppState {
            count: 0,
            name: "test".to_string(),
        });

        let call_count = Arc::new(AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        store.subscribe(move |_state| {
            call_count_clone.fetch_add(1, Ordering::SeqCst);
        });

        assert_eq!(call_count.load(Ordering::SeqCst), 0);

        store.update(|state| state.count += 1);
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        store.update(|state| state.count += 1);
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }
}
