use std::sync::Arc;
use tokio::sync::OnceCell;

/// Once<T> - Initialize-once pattern for lazy initialization
/// Thread-safe one-time initialization of a value
/// After initialization, provides fast read-only access
pub struct Once<T> {
    inner: Arc<OnceCell<T>>,
}

impl<T> Once<T> {
    /// Create a new uninitialized Once<T>
    pub fn new() -> Self {
        Self {
            inner: Arc::new(OnceCell::new()),
        }
    }

    /// Create a new Once<T> with an initial value
    pub fn with_value(value: T) -> Self {
        let once = Self::new();
        // This will always succeed since we just created it
        let _ = once.set(value);
        once
    }

    /// Initialize the value
    /// Returns true if successfully initialized
    /// Returns false if already initialized
    pub fn set(&self, value: T) -> bool {
        self.inner.set(value).is_ok()
    }

    /// Get the value if initialized
    /// Returns None if not yet initialized
    pub fn get(&self) -> Option<&T> {
        self.inner.get()
    }

    /// Get or initialize the value with a closure
    /// The closure is only called if the value is not yet initialized
    pub async fn get_or_init<F>(&self, f: F) -> &T
    where
        F: FnOnce() -> T,
    {
        self.inner.get_or_init(|| async { f() }).await
    }

    /// Get or try to initialize with a fallible closure
    /// The closure is only called if the value is not yet initialized
    pub async fn get_or_try_init<F, E>(&self, f: F) -> Result<&T, E>
    where
        F: FnOnce() -> Result<T, E>,
    {
        self.inner
            .get_or_try_init(|| async { f() })
            .await
    }

    /// Check if the value has been initialized
    pub fn is_initialized(&self) -> bool {
        self.inner.initialized()
    }

    /// Take the value, leaving the Once uninitialized
    /// This is useful for cleanup or moving the value out
    pub fn take(&mut self) -> Option<T> {
        Arc::get_mut(&mut self.inner).and_then(|cell| cell.take())
    }
}

impl<T> Clone for Once<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> Default for Once<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to create a new Once<T>
pub fn once<T>() -> Once<T> {
    Once::new()
}

/// Helper to create a new Once<T> with an initial value
pub fn once_with<T>(value: T) -> Once<T> {
    Once::with_value(value)
}