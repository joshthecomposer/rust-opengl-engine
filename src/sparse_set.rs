use core::slice;
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct SparseSet<T> {
    pub dense: Vec<Entry<T>>,
    pub sparse: Vec<usize>
}

#[derive(Debug)]
pub struct Entry<T> {
    key: usize,
    pub value: T,
}

impl<T> Entry<T> {
    /// Read-only access to the entry's key.
    pub fn key(&self) -> usize {
        self.key
    }

    /// Returns the value. Mainly for symmetry with key() since the value is public anyway.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Returns the value, mutable. Mainly for symmetry with key() since the value is public
    /// anyway.
    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T> SparseSet<T> {
    /// Creates a SparseSet with the given capacity.
    pub fn with_capacity(size: usize) -> Self {
        let mut sparse = Vec::with_capacity(size);
        unsafe { sparse.set_len(size) }
        SparseSet {
            dense: Vec::with_capacity(size),
            sparse,
        }
    }

    pub fn len(&self) -> usize {
        self.dense.len()
    }
    pub fn capacity(&self) -> usize {
        self.sparse.len()
    }

    /// Clears the SparseSet in O(1) for simple T and O(n) if T implements Drop.
    pub fn clear(&mut self) {
        self.dense.clear();
    }

    fn dense_idx(&self, key: usize) -> Option<usize> {
        let dense_idx = self.sparse[key];
        if dense_idx < self.len() {
            let entry = &self.dense[dense_idx];
            if entry.key == key {
                return Some(dense_idx);
            }
        }
        None
    }

    /// Returns a reference to the value corresponding to the given key in O(1).
    pub fn get(&self, key: usize) -> Option<&T> {
        if let Some(dense_idx) = self.dense_idx(key) {
            Some(&self.dense[dense_idx].value)
        } else {
            None
        }
    }

    /// Returns a mutable reference to the value corresponding to the given key in O(1).
    pub fn get_mut(&mut self, key: usize) -> Option<&mut T> {
        if let Some(dense_idx) = self.dense_idx(key) {
            Some(&mut self.dense[dense_idx].value)
        } else {
            None
        }
    }

    /// Test if the given key is contained in the set in O(1).
    pub fn contains(&self, key: usize) -> bool {
        self.dense_idx(key).is_some()
    }

    /// Insert in the set a value for the given key in O(1).
    ///
    /// * returns true if the key was set.
    /// * returns false if the key was already set.
    ///
    /// If the key was already set, the previous value is overridden.
    pub fn insert(&mut self, key: usize, value: T) -> bool {
        assert!(
            key < self.capacity(),
            "key ({}) must be under capacity ({})",
            key,
            self.capacity()
        );
        if let Some(stored_value) = self.get_mut(key) {
            *stored_value = value;
            return false;
        }
        let n = self.dense.len();
        self.dense.push(Entry {
            key: key,
            value: value,
        });
        self.sparse[key] = n;
        true
    }

    /// Removes the given key in O(1).
    /// Returns the removed value or None if key not found.
    pub fn remove(&mut self, key: usize) -> Option<T> {
        if self.contains(key) {
            let dense_idx = self.sparse[key];
            let r = self.dense.swap_remove(dense_idx).value;
            if dense_idx < self.len() {
                let swapped_entry = &self.dense[dense_idx];
                self.sparse[swapped_entry.key] = dense_idx;
            }
            // not strictly necessary, just nice to
            // restrict any future contains(key) to one test.
            self.sparse[key] = self.capacity();
            Some(r)
        } else {
            None
        }
    }
}

/// Deref to a slice.
impl<T> Deref for SparseSet<T> {
    type Target = [Entry<T>];

    fn deref(&self) -> &Self::Target {
        &self.dense[..]
    }
}

/// Deref to a mutable slice.
impl<T> DerefMut for SparseSet<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.dense[..]
    }
}

/// Move into an interator, consuming the SparseSet.
impl<T> IntoIterator for SparseSet<T> {
    type Item = Entry<T>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.dense.into_iter()
    }
}

/// An interator over the elements of the SparseSet.
impl<'a, T> IntoIterator for &'a SparseSet<T> {
    type Item = &'a Entry<T>;
    type IntoIter = slice::Iter<'a, Entry<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An interator over mutable elements of the SparseSet.
impl<'a, T> IntoIterator for &'a mut SparseSet<T> {
    type Item = &'a mut Entry<T>;
    type IntoIter = slice::IterMut<'a, Entry<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
