use std::{iter::Iterator, marker::PhantomData};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct ID<const N:usize> (usize);

pub struct IDFactory<I>(usize, PhantomData<I>);

impl<const N:usize> IDFactory<ID<N>> {
    pub fn new() -> Self {
        Self(0, PhantomData::default())
    }

    pub fn get_id(&mut self) -> ID<N> {
        self.0 += 1;
        ID(self.0)
    }
}

impl<const N:usize> Iterator for IDFactory<ID<N>> {
    type Item = ID<N>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.get_id())
    }
}
