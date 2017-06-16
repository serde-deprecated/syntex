// Copyright 2016 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt::Debug;
use std::iter::FromIterator;
use std::slice;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};
use std::fmt;
use std::vec;
use std::u32;

use serde::{Serialize, Serializer, Deserialize, Deserializer};

/// Represents some newtyped `usize` wrapper.
///
/// (purpose: avoid mixing indexes for different bitvector domains.)
pub trait Idx: Copy + 'static + Eq + Debug {
    fn new(idx: usize) -> Self;
    fn index(self) -> usize;
}

impl Idx for usize {
    fn new(idx: usize) -> Self { idx }
    fn index(self) -> usize { self }
}

impl Idx for u32 {
    fn new(idx: usize) -> Self { assert!(idx <= u32::MAX as usize); idx as u32 }
    fn index(self) -> usize { self as usize }
}

#[derive(Clone)]
pub struct IndexVec<I: Idx, T> {
    pub raw: Vec<T>,
    _marker: PhantomData<Fn(&I)>
}

impl<I, T> Serialize for IndexVec<I, T>
    where I: Idx,
          T: Serialize
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        self.raw.serialize(serializer)
    }
}

impl<'de, I, T> Deserialize<'de> for IndexVec<I, T>
    where I: Idx,
          T: Deserialize<'de>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        Deserialize::deserialize(deserializer).map(|v| {
            IndexVec { raw: v, _marker: PhantomData }
        })
    }
}

impl<I: Idx, T: fmt::Debug> fmt::Debug for IndexVec<I, T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.raw, fmt)
    }
}

impl<I: Idx, T> IndexVec<I, T> {
    #[inline]
    pub fn new() -> Self {
        IndexVec { raw: Vec::new(), _marker: PhantomData }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        IndexVec { raw: Vec::with_capacity(capacity), _marker: PhantomData }
    }

    #[inline]
    pub fn from_elem<S>(elem: T, universe: &IndexVec<I, S>) -> Self
        where T: Clone
    {
        IndexVec { raw: vec![elem; universe.len()], _marker: PhantomData }
    }

    #[inline]
    pub fn from_elem_n(elem: T, n: usize) -> Self
        where T: Clone
    {
        IndexVec { raw: vec![elem; n], _marker: PhantomData }
    }

    #[inline]
    pub fn push(&mut self, d: T) -> I {
        let idx = I::new(self.len());
        self.raw.push(d);
        idx
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.raw.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    #[inline]
    pub fn into_iter(self) -> vec::IntoIter<T> {
        self.raw.into_iter()
    }

    #[inline]
    pub fn iter(&self) -> slice::Iter<T> {
        self.raw.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> slice::IterMut<T> {
        self.raw.iter_mut()
    }

    #[inline]
    pub fn last(&self) -> Option<I> {
        self.len().checked_sub(1).map(I::new)
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.raw.shrink_to_fit()
    }

    #[inline]
    pub fn swap(&mut self, a: usize, b: usize) {
        self.raw.swap(a, b)
    }

    #[inline]
    pub fn truncate(&mut self, a: usize) {
        self.raw.truncate(a)
    }

    #[inline]
    pub fn get(&self, index: I) -> Option<&T> {
        self.raw.get(index.index())
    }

    #[inline]
    pub fn get_mut(&mut self, index: I) -> Option<&mut T> {
        self.raw.get_mut(index.index())
    }
}

impl<I: Idx, T: Clone> IndexVec<I, T> {
    #[inline]
    pub fn resize(&mut self, new_len: usize, value: T) {
        self.raw.resize(new_len, value)
    }
}

impl<I: Idx, T> Index<I> for IndexVec<I, T> {
    type Output = T;

    #[inline]
    fn index(&self, index: I) -> &T {
        &self.raw[index.index()]
    }
}

impl<I: Idx, T> IndexMut<I> for IndexVec<I, T> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut T {
        &mut self.raw[index.index()]
    }
}

impl<I: Idx, T> Default for IndexVec<I, T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<I: Idx, T> Extend<T> for IndexVec<I, T> {
    #[inline]
    fn extend<J: IntoIterator<Item = T>>(&mut self, iter: J) {
        self.raw.extend(iter);
    }
}

impl<I: Idx, T> FromIterator<T> for IndexVec<I, T> {
    #[inline]
    fn from_iter<J>(iter: J) -> Self where J: IntoIterator<Item=T> {
        IndexVec { raw: FromIterator::from_iter(iter), _marker: PhantomData }
    }
}

impl<I: Idx, T> IntoIterator for IndexVec<I, T> {
    type Item = T;
    type IntoIter = vec::IntoIter<T>;

    #[inline]
    fn into_iter(self) -> vec::IntoIter<T> {
        self.raw.into_iter()
    }

}

impl<'a, I: Idx, T> IntoIterator for &'a IndexVec<I, T> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    #[inline]
    fn into_iter(self) -> slice::Iter<'a, T> {
        self.raw.iter()
    }
}

impl<'a, I: Idx, T> IntoIterator for &'a mut IndexVec<I, T> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    #[inline]
    fn into_iter(mut self) -> slice::IterMut<'a, T> {
        self.raw.iter_mut()
    }
}
