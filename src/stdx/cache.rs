use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct Cache<T>(Arc<RwLock<Store<T>>>);

#[derive(Debug, Clone, Default)]
pub enum Store<T> {
    #[default]
    Empty,
    Value(T),
}

impl<T> Cache<T> {
    #[inline]
    pub fn empty() -> Self {
        Self(Arc::new(RwLock::new(Store::Empty)))
    }

    #[inline]
    pub fn new(item: T) -> Self
    where
        T: Clone,
    {
        Self(Arc::new(RwLock::new(Store::Value(item))))
    }

    #[inline]
    pub fn insert(&self, item: T)
    where
        T: Clone,
    {
        *self.0.write() = Store::Value(item);
    }

    #[inline]
    pub fn get(&self) -> Store<T>
    where
        T: Clone,
    {
        self.0.read().clone()
    }
}

impl<T> Store<T> {
    #[inline]
    pub fn or_default(self) -> T
    where
        T: Default,
    {
        match self {
            Self::Empty => Default::default(),
            Self::Value(item) => item,
        }
    }
}

pub enum State<T> {
    Fresh(T),
    Stale,
}

impl<T> State<T> {
    pub fn is_fresh(&self) -> bool {
        matches!(self, Self::Fresh(_))
    }

    pub fn is_stale(&self) -> bool {
        matches!(self, Self::Stale)
    }
}
