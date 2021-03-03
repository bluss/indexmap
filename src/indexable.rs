use crate::IndexMap;
use core::ops::{Index, IndexMut};

/// Indexable types can convert to and from `usize`.
pub trait Indexable: Sized + Copy + Ord + Send + Sync {
    /// Creates an index from a `usize` or returns `None`.
    fn try_from_usize(i: usize) -> Option<Self>;

    /// Creates an index from a `usize`; panics on failure.
    #[inline]
    fn from_usize(i: usize) -> Self {
        Self::try_from_usize(i).expect("invalid index!")
    }

    /// Converts the index to a `usize` or returns `None`.
    fn try_into_usize(self) -> Option<usize>;

    /// Converts the index to a `usize`; panics on failure.
    #[inline]
    fn into_usize(self) -> usize {
        self.try_into_usize().expect("invalid index!")
    }
}

impl Indexable for usize {
    #[inline]
    fn try_from_usize(i: usize) -> Option<Self> {
        Some(i)
    }

    #[inline]
    fn from_usize(i: usize) -> Self {
        i
    }

    #[inline]
    fn try_into_usize(self) -> Option<usize> {
        Some(self)
    }

    #[inline]
    fn into_usize(self) -> usize {
        self
    }
}

macro_rules! impl_indexable {
    ($($ty:ident)*) => {$(
        impl Indexable for $ty {
            #[inline]
            fn try_from_usize(i: usize) -> Option<Self> {
                if i <= core::$ty::MAX as usize {
                    Some(i as $ty)
                } else {
                    None
                }
            }

            #[inline]
            fn try_into_usize(self) -> Option<usize> {
                if self <= core::usize::MAX as $ty {
                    Some(self as usize)
                } else {
                    None
                }
            }
        }

        impl<K, V, S> Index<$ty> for IndexMap<K, V, S, $ty> {
            type Output = V;

            fn index(&self, index: $ty) -> &V {
                self.get_index(index)
                    .expect("IndexMap: index out of bounds")
                    .1
            }
        }

        impl<K, V, S> IndexMut<$ty> for IndexMap<K, V, S, $ty> {
            fn index_mut(&mut self, index: $ty) -> &mut V {
                self.get_index_mut(index)
                    .expect("IndexMap: index out of bounds")
                    .1
            }
        }
    )*}
}

impl_indexable!(u8 u16 u32 u64 u128);
