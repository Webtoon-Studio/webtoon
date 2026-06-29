use parking_lot::RwLock;
use std::sync::Arc;

/// A cloneable, thread-safe cache wrapping a [`Store`].
///
/// Internally uses an `Arc<RwLock<Store<T>>>` so clones share the same underlying value.
#[derive(Debug, Clone, Default)]
pub struct Cache<T>(Arc<RwLock<Store<T>>>);

/// The state of a [`Cache`], either populated or empty.
#[derive(Debug, Clone, Default)]
pub enum Store<T> {
    /// No value has been inserted yet.
    #[default]
    Empty,
    /// A value is present.
    Value(T),
}

impl<T> Cache<T> {
    /// Creates an empty [`Cache`].
    #[inline]
    pub fn empty() -> Self {
        Self(Arc::new(RwLock::new(Store::Empty)))
    }

    /// Creates a [`Cache`] pre-populated with `item`.
    #[inline]
    pub fn new(item: T) -> Self
    where
        T: Clone,
    {
        Self(Arc::new(RwLock::new(Store::Value(item))))
    }

    /// Inserts `item` into the cache, replacing any existing value.
    #[inline]
    pub fn insert(&self, item: T)
    where
        T: Clone,
    {
        *self.0.write() = Store::Value(item);
    }

    /// Returns a clone of the current [`Store`].
    #[inline]
    pub fn get(&self) -> Store<T>
    where
        T: Clone,
    {
        self.0.read().clone()
    }
}

impl<T: Default> Store<T> {
    /// Returns the contained value, or `T::default()` if empty.
    #[inline]
    pub fn or_default(self) -> T {
        match self {
            Self::Empty => T::default(),
            Self::Value(item) => item,
        }
    }
}
