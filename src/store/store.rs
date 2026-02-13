use std::sync::{Arc, RwLock};

type Subscriber<T> = Box<dyn Fn(&T) + Send + Sync>;

/// A thread-safe store for managing application state.
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
