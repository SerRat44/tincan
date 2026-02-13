use std::sync::{Arc, RwLock};

type Subscriber<T> = Box<dyn Fn(&T) + Send + Sync>;

/// A thread-safe store for managing application state.
///
/// Stores provide a higher-level abstraction over signals for managing
/// complex state with automatic change detection.
///
/// # Examples
///
/// ```
/// use tincan::Store;
///
/// #[derive(Clone)]
/// struct AppState {
///     count: i32,
///     name: String,
/// }
///
/// let store = Store::new(AppState {
///     count: 0,
///     name: "Tincan".to_string(),
/// });
///
/// store.update(|state| {
///     state.count += 1;
/// });
///
/// assert_eq!(store.get().count, 1);
/// ```
pub struct Store<T> {
    state: Arc<RwLock<T>>,
    subscribers: Arc<RwLock<Vec<Subscriber<T>>>>,
}

impl<T: Clone> Store<T> {
    /// Create a new store with the given initial state.
    ///
    /// # Examples
    ///
    /// ```
    /// use tincan::Store;
    ///
    /// let store = Store::new(0);
    /// assert_eq!(store.get(), 0);
    /// ```
    pub fn new(initial: T) -> Self {
        Self {
            state: Arc::new(RwLock::new(initial)),
            subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get a clone of the current state.
    ///
    /// # Examples
    ///
    /// ```
    /// use tincan::Store;
    ///
    /// let store = Store::new(42);
    /// assert_eq!(store.get(), 42);
    /// ```
    pub fn get(&self) -> T {
        self.state.read().unwrap().clone()
    }

    /// Update the state using a function.
    ///
    /// All subscribers will be notified after the update.
    ///
    /// # Examples
    ///
    /// ```
    /// use tincan::Store;
    ///
    /// let store = Store::new(0);
    /// store.update(|n| *n += 10);
    /// assert_eq!(store.get(), 10);
    /// ```
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
    ///
    /// All subscribers will be notified.
    ///
    /// # Examples
    ///
    /// ```
    /// use tincan::Store;
    ///
    /// let store = Store::new(0);
    /// store.set(42);
    /// assert_eq!(store.get(), 42);
    /// ```
    pub fn set(&self, new_state: T) {
        *self.state.write().unwrap() = new_state;
        self.notify();
    }

    /// Subscribe to state changes.
    ///
    /// The callback will be called whenever the state is updated.
    ///
    /// # Examples
    ///
    /// ```
    /// use tincan::Store;
    /// use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
    ///
    /// let store = Store::new(0);
    /// let counter = Arc::new(AtomicUsize::new(0));
    /// let counter_clone = counter.clone();
    ///
    /// store.subscribe(move |_| {
    ///     counter_clone.fetch_add(1, Ordering::SeqCst);
    /// });
    ///
    /// store.update(|n| *n += 1);
    /// assert_eq!(counter.load(Ordering::SeqCst), 1);
    /// ```
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
    ///
    /// Useful for accessing state without cloning.
    ///
    /// # Examples
    ///
    /// ```
    /// use tincan::Store;
    ///
    /// let store = Store::new(String::from("hello"));
    /// let len = store.read(|s| s.len());
    /// assert_eq!(len, 5);
    /// ```
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
