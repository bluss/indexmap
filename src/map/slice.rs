use super::{Bucket, Entries, IndexMap, Iter, IterMut, Keys, Values, ValuesMut};
use crate::util::{simplify_range, try_simplify_range};

use core::cmp::Ordering;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::ops::{self, Bound, Index, IndexMut, RangeBounds};

/// A dynamically-sized slice of key-value pairs in an `IndexMap`.
///
/// This supports indexed operations much like a `[(K, V)]` slice,
/// but not any hashed operations on the map keys.
///
/// Unlike `IndexMap`, `Slice` does consider the order for `PartialEq`
/// and `Eq`, and it also implements `PartialOrd`, `Ord`, and `Hash`.
#[repr(transparent)]
pub struct Slice<K, V> {
    pub(crate) entries: [Bucket<K, V>],
}

#[allow(unsafe_code)]
impl<K, V> Slice<K, V> {
    fn from_slice(entries: &[Bucket<K, V>]) -> &Self {
        // SAFETY: `Slice<K, V>` is a transparent wrapper around `[Bucket<K, V>]`,
        // and the lifetimes are bound together by this function's signature.
        unsafe { &*(entries as *const [Bucket<K, V>] as *const Self) }
    }

    fn from_mut_slice(entries: &mut [Bucket<K, V>]) -> &mut Self {
        // SAFETY: `Slice<K, V>` is a transparent wrapper around `[Bucket<K, V>]`,
        // and the lifetimes are bound together by this function's signature.
        unsafe { &mut *(entries as *mut [Bucket<K, V>] as *mut Self) }
    }
}

impl<K, V, S> IndexMap<K, V, S> {
    /// Returns a slice of all the entries in the map.
    pub fn as_slice(&self) -> &Slice<K, V> {
        Slice::from_slice(self.as_entries())
    }

    /// Returns a mutable slice of all the entries in the map.
    pub fn as_mut_slice(&mut self) -> &mut Slice<K, V> {
        Slice::from_mut_slice(self.as_entries_mut())
    }

    /// Returns a slice of entries in the given range of indices.
    ///
    /// Valid indices are *0 <= index < self.len()*
    pub fn get_range<R: RangeBounds<usize>>(&self, range: R) -> Option<&Slice<K, V>> {
        let entries = self.as_entries();
        let range = try_simplify_range(range, entries.len())?;
        entries.get(range).map(Slice::from_slice)
    }

    /// Returns a mutable slice of entries in the given range of indices.
    ///
    /// Valid indices are *0 <= index < self.len()*
    pub fn get_range_mut<R: RangeBounds<usize>>(&mut self, range: R) -> Option<&mut Slice<K, V>> {
        let entries = self.as_entries_mut();
        let range = try_simplify_range(range, entries.len())?;
        entries.get_mut(range).map(Slice::from_mut_slice)
    }
}

impl<'a, K, V> Iter<'a, K, V> {
    /// Returns a slice of the remaining entries in the iterator.
    pub fn as_slice(&self) -> &'a Slice<K, V> {
        Slice::from_slice(self.iter.as_slice())
    }
}

impl<'a, K, V> IterMut<'a, K, V> {
    /// Returns a slice of the remaining entries in the iterator.
    ///
    /// To avoid creating `&mut` references that alias, this is forced to consume the iterator.
    pub fn into_slice(self) -> &'a mut Slice<K, V> {
        Slice::from_mut_slice(self.iter.into_slice())
    }
}

impl<K, V> Slice<K, V> {
    /// Return the number of key-value pairs in the map slice.
    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true if the map slice contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get a key-value pair by index.
    ///
    /// Valid indices are *0 <= index < self.len()*
    pub fn get_index(&self, index: usize) -> Option<(&K, &V)> {
        self.entries.get(index).map(Bucket::refs)
    }

    /// Get a key-value pair by index, with mutable access to the value.
    ///
    /// Valid indices are *0 <= index < self.len()*
    pub fn get_index_mut(&mut self, index: usize) -> Option<(&K, &mut V)> {
        // NB: we're not returning `&mut K` like `IndexMap::get_index_mut`,
        // because that was a mistake that should have required `MutableKeys`.
        self.entries.get_mut(index).map(Bucket::ref_mut)
    }

    /// Returns a slice of key-value pairs in the given range of indices.
    ///
    /// Valid indices are *0 <= index < self.len()*
    pub fn get_range<R: RangeBounds<usize>>(&self, range: R) -> Option<&Self> {
        let range = try_simplify_range(range, self.entries.len())?;
        self.entries.get(range).map(Slice::from_slice)
    }

    /// Returns a mutable slice of key-value pairs in the given range of indices.
    ///
    /// Valid indices are *0 <= index < self.len()*
    pub fn get_range_mut<R: RangeBounds<usize>>(&mut self, range: R) -> Option<&mut Self> {
        let range = try_simplify_range(range, self.entries.len())?;
        self.entries.get_mut(range).map(Slice::from_mut_slice)
    }

    /// Get the first key-value pair.
    pub fn first(&self) -> Option<(&K, &V)> {
        self.entries.first().map(Bucket::refs)
    }

    /// Get the first key-value pair, with mutable access to the value.
    pub fn first_mut(&mut self) -> Option<(&K, &mut V)> {
        self.entries.first_mut().map(Bucket::ref_mut)
    }

    /// Get the last key-value pair.
    pub fn last(&self) -> Option<(&K, &V)> {
        self.entries.last().map(Bucket::refs)
    }

    /// Get the last key-value pair, with mutable access to the value.
    pub fn last_mut(&mut self) -> Option<(&K, &mut V)> {
        self.entries.last_mut().map(Bucket::ref_mut)
    }

    /// Divides one slice into two at an index.
    ///
    /// ***Panics*** if `index > len`.
    pub fn split_at(&self, index: usize) -> (&Self, &Self) {
        let (first, second) = self.entries.split_at(index);
        (Self::from_slice(first), Self::from_slice(second))
    }

    /// Divides one mutable slice into two at an index.
    ///
    /// ***Panics*** if `index > len`.
    pub fn split_at_mut(&mut self, index: usize) -> (&mut Self, &mut Self) {
        let (first, second) = self.entries.split_at_mut(index);
        (Self::from_mut_slice(first), Self::from_mut_slice(second))
    }

    /// Returns the first key-value pair and the rest of the slice,
    /// or `None` if it is empty.
    pub fn split_first(&self) -> Option<((&K, &V), &Self)> {
        if let Some((first, rest)) = self.entries.split_first() {
            Some((first.refs(), Self::from_slice(rest)))
        } else {
            None
        }
    }

    /// Returns the first key-value pair and the rest of the slice,
    /// with mutable access to the value, or `None` if it is empty.
    pub fn split_first_mut(&mut self) -> Option<((&K, &mut V), &mut Self)> {
        if let Some((first, rest)) = self.entries.split_first_mut() {
            Some((first.ref_mut(), Self::from_mut_slice(rest)))
        } else {
            None
        }
    }

    /// Returns the last key-value pair and the rest of the slice,
    /// or `None` if it is empty.
    pub fn split_last(&self) -> Option<((&K, &V), &Self)> {
        if let Some((last, rest)) = self.entries.split_last() {
            Some((last.refs(), Self::from_slice(rest)))
        } else {
            None
        }
    }

    /// Returns the last key-value pair and the rest of the slice,
    /// with mutable access to the value, or `None` if it is empty.
    pub fn split_last_mut(&mut self) -> Option<((&K, &mut V), &mut Self)> {
        if let Some((last, rest)) = self.entries.split_last_mut() {
            Some((last.ref_mut(), Self::from_mut_slice(rest)))
        } else {
            None
        }
    }

    /// Return an iterator over the key-value pairs of the map slice.
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            iter: self.entries.iter(),
        }
    }

    /// Return an iterator over the key-value pairs of the map slice.
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut {
            iter: self.entries.iter_mut(),
        }
    }

    /// Return an iterator over the keys of the map slice.
    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys {
            iter: self.entries.iter(),
        }
    }

    /// Return an iterator over the values of the map slice.
    pub fn values(&self) -> Values<'_, K, V> {
        Values {
            iter: self.entries.iter(),
        }
    }

    /// Return an iterator over mutable references to the the values of the map slice.
    pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
        ValuesMut {
            iter: self.entries.iter_mut(),
        }
    }
}

impl<'a, K, V> IntoIterator for &'a Slice<K, V> {
    type IntoIter = Iter<'a, K, V>;
    type Item = (&'a K, &'a V);

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, K, V> IntoIterator for &'a mut Slice<K, V> {
    type IntoIter = IterMut<'a, K, V>;
    type Item = (&'a K, &'a mut V);

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<K, V> Default for &'_ Slice<K, V> {
    fn default() -> Self {
        Slice::from_slice(&[])
    }
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for Slice<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self).finish()
    }
}

impl<K: PartialEq, V: PartialEq> PartialEq for Slice<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other)
    }
}

impl<K: Eq, V: Eq> Eq for Slice<K, V> {}

impl<K: PartialOrd, V: PartialOrd> PartialOrd for Slice<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.iter().partial_cmp(other)
    }
}

impl<K: Ord, V: Ord> Ord for Slice<K, V> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.iter().cmp(other)
    }
}

impl<K: Hash, V: Hash> Hash for Slice<K, V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.len().hash(state);
        for (key, value) in self {
            key.hash(state);
            value.hash(state);
        }
    }
}

impl<K, V> Index<usize> for Slice<K, V> {
    type Output = V;

    fn index(&self, index: usize) -> &V {
        &self.entries[index].value
    }
}

impl<K, V> IndexMut<usize> for Slice<K, V> {
    fn index_mut(&mut self, index: usize) -> &mut V {
        &mut self.entries[index].value
    }
}

// We can't have `impl<I: RangeBounds<usize>> Index<I>` because that conflicts
// both upstream with `Index<usize>` and downstream with `Index<&Q>`.
// Instead, we repeat the implementations for all the core range types.
macro_rules! impl_index {
    ($($range:ty),*) => {$(
        impl<K, V, S> Index<$range> for IndexMap<K, V, S> {
            type Output = Slice<K, V>;

            fn index(&self, range: $range) -> &Self::Output {
                Slice::from_slice(&self.as_entries()[range])
            }
        }

        impl<K, V, S> IndexMut<$range> for IndexMap<K, V, S> {
            fn index_mut(&mut self, range: $range) -> &mut Self::Output {
                Slice::from_mut_slice(&mut self.as_entries_mut()[range])
            }
        }

        impl<K, V> Index<$range> for Slice<K, V> {
            type Output = Slice<K, V>;

            fn index(&self, range: $range) -> &Self {
                Self::from_slice(&self.entries[range])
            }
        }

        impl<K, V> IndexMut<$range> for Slice<K, V> {
            fn index_mut(&mut self, range: $range) -> &mut Self {
                Self::from_mut_slice(&mut self.entries[range])
            }
        }
    )*}
}
impl_index!(
    ops::Range<usize>,
    ops::RangeFrom<usize>,
    ops::RangeFull,
    ops::RangeInclusive<usize>,
    ops::RangeTo<usize>,
    ops::RangeToInclusive<usize>
);

// NB: with MSRV 1.53, we can forward `Bound` pairs to direct slice indexing like other ranges

impl<K, V, S> Index<(Bound<usize>, Bound<usize>)> for IndexMap<K, V, S> {
    type Output = Slice<K, V>;

    fn index(&self, range: (Bound<usize>, Bound<usize>)) -> &Self::Output {
        let entries = self.as_entries();
        let range = simplify_range(range, entries.len());
        Slice::from_slice(&entries[range])
    }
}

impl<K, V, S> IndexMut<(Bound<usize>, Bound<usize>)> for IndexMap<K, V, S> {
    fn index_mut(&mut self, range: (Bound<usize>, Bound<usize>)) -> &mut Self::Output {
        let entries = self.as_entries_mut();
        let range = simplify_range(range, entries.len());
        Slice::from_mut_slice(&mut entries[range])
    }
}

impl<K, V> Index<(Bound<usize>, Bound<usize>)> for Slice<K, V> {
    type Output = Slice<K, V>;

    fn index(&self, range: (Bound<usize>, Bound<usize>)) -> &Self {
        let entries = &self.entries;
        let range = simplify_range(range, entries.len());
        Slice::from_slice(&entries[range])
    }
}

impl<K, V> IndexMut<(Bound<usize>, Bound<usize>)> for Slice<K, V> {
    fn index_mut(&mut self, range: (Bound<usize>, Bound<usize>)) -> &mut Self {
        let entries = &mut self.entries;
        let range = simplify_range(range, entries.len());
        Slice::from_mut_slice(&mut entries[range])
    }
}
