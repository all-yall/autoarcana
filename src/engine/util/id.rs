use std::{iter::Iterator, marker::PhantomData, fmt::Debug, any::type_name, hash::Hash, cell::Cell};

pub struct ID<T> (usize, PhantomData<T>);

/// SAFETY: PhantomData holds no data, and it is safe to send usizes between threads
unsafe impl<T> Send for ID<T> {}
unsafe impl<T> Sync for ID<T> {}

pub trait IDMapper<T> {
    fn get(&self, id: ID<T>) -> &T;
    fn get_mut(&mut self, id: ID<T>) -> &mut T;
}

impl<T> Hash for ID<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<T> Clone for ID<T> {
    fn clone(&self) -> Self {
        Self(self.0, PhantomData::default())
    }
}

impl<T> Debug for ID<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("'{} ID {}'", type_name::<T>(), self.0))
    }
}

impl<T> Copy for ID<T> {}

impl<T> PartialEq for ID<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T> Eq for ID<T> {}

impl<T> Ord for ID<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl<T> PartialOrd for ID<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}


pub struct IDFactory<I>(Cell<usize>, PhantomData<I>);

impl<T> IDFactory<ID<T>> {
    pub fn new() -> Self {
        Self(Cell::new(0), PhantomData::default())
    }

    pub fn get_id(&self) -> ID<T> {
        let mut num = self.0.get();
        num += 1;
        self.0.set(num);
        ID(num, PhantomData::default())
    }
}

impl<T> Iterator for IDFactory<ID<T>> {
    type Item = ID<T>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.get_id())
    }
}
