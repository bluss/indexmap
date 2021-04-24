use crate::Indexable;
use core::iter::Enumerate;
use core::ops::{Bound, Range, RangeBounds};

pub(crate) fn third<A, B, C>(t: (A, B, C)) -> C {
    t.2
}

pub(crate) fn enumerate<I>(iterable: I) -> Enumerate<I::IntoIter>
where
    I: IntoIterator,
{
    iterable.into_iter().enumerate()
}

fn usize_bound<Idx: Indexable>(bound: Bound<&Idx>) -> Bound<usize> {
    match bound {
        Bound::Included(&i) => Bound::Included(i.into_usize()),
        Bound::Excluded(&i) => Bound::Excluded(i.into_usize()),
        Bound::Unbounded => Bound::Unbounded,
    }
}

pub(crate) fn simplify_range<R, Idx>(range: R, len: usize) -> Range<usize>
where
    R: RangeBounds<Idx>,
    Idx: Indexable,
{
    let start_bound = usize_bound(range.start_bound());
    let start = match start_bound {
        Bound::Unbounded => 0,
        Bound::Included(i) if i <= len => i,
        Bound::Excluded(i) if i < len => i + 1,
        bound => panic!("range start {:?} should be <= length {}", bound, len),
    };

    let end_bound = usize_bound(range.end_bound());
    let end = match end_bound {
        Bound::Unbounded => len,
        Bound::Excluded(i) if i <= len => i,
        Bound::Included(i) if i < len => i + 1,
        bound => panic!("range end {:?} should be <= length {}", bound, len),
    };

    if start > end {
        panic!(
            "range start {:?} should be <= range end {:?}",
            start_bound, end_bound
        );
    }
    start..end
}
