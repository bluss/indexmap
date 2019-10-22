
use std::iter::Enumerate;
use std::mem::size_of;
use std::ops::Bound;
use std::ops::RangeBounds;
use std::ops::{Range, RangeFrom, RangeInclusive};

pub fn third<A, B, C>(t: (A, B, C)) -> C { t.2 }

pub fn enumerate<I>(iterable: I) -> Enumerate<I::IntoIter>
    where I: IntoIterator
{
    iterable.into_iter().enumerate()
}

/// return the number of steps from a to b
pub fn ptrdistance<T>(a: *const T, b: *const T) -> usize {
    debug_assert!(a as usize <= b as usize);
    (b as usize - a as usize) / size_of::<T>()
}

// Trait for calling &slice[r] for range r on either shared or mutable slices
pub trait SliceWithRange {
    fn slice_range(self, r: Range<usize>) -> Self;
    fn slice_rangefrom(self, r: RangeFrom<usize>) -> Self;
    fn slice_rangeincl(self, r: RangeInclusive<usize>) -> Self;
}

impl<'a, T> SliceWithRange for &'a [T] {
    fn slice_range(self, r: Range<usize>) -> Self { &self[r] }
    fn slice_rangefrom(self, r: RangeFrom<usize>) -> Self { &self[r] }
    fn slice_rangeincl(self, r: RangeInclusive<usize>) -> Self { &self[r] }
}

impl<'a, T> SliceWithRange for &'a mut [T] {
    fn slice_range(self, r: Range<usize>) -> Self { &mut self[r] }
    fn slice_rangefrom(self, r: RangeFrom<usize>) -> Self { &mut self[r] }
    fn slice_rangeincl(self, r: RangeInclusive<usize>) -> Self { &mut self[r] }
}

/// Fluent API for slicing with RangeBounds on vectors and slices
pub trait SliceWith {
    fn slice<I>(&self, r: I) -> &Self where I: RangeBounds<usize>;
    fn slice_mut<I>(&mut self, r: I) -> &mut Self where I: RangeBounds<usize>;
}

impl<T> SliceWith for [T] {
    fn slice<I>(&self, r: I) -> &Self where I: RangeBounds<usize>
    { slice_by(self, r) }

    fn slice_mut<I>(&mut self, r: I) -> &mut Self where I: RangeBounds<usize>
    { slice_by(self, r) }
}

fn slice_by<S, I>(v: S, range: I) -> S
    where S: SliceWithRange, I: RangeBounds<usize>
{
    let start = match range.start_bound() {
        Bound::Unbounded => 0,
        Bound::Included(&i) => i,
        Bound::Excluded(&i) => check_exclusive_to_inclusive_start(i),
    };
    match range.end_bound() {
        Bound::Excluded(&i) => v.slice_range(start..i),
        Bound::Included(&i) => v.slice_rangeincl(start..=i),
        Bound::Unbounded => v.slice_rangefrom(start..),
    }
}

/// Compute i + 1 but check for overflow
#[inline]
fn check_exclusive_to_inclusive_start(i: usize) -> usize {
    // Range exclusive from usize max is always out of bounds.
    // Exclusive from usize max to unbounded is a well-formed range (!0, infinity)
    // but always out of bounds in slices which have len <= !0 anyway.
    //
    // Range exclusive from usize max to Included/Excluded usize value is
    // an ill-formed range because it does not have start <= end, but we
    // don't make the difference here.
    //
    // start at i + 1 but panic for oob on overflow
    let (iplus1, overflow) = i.overflowing_add(1);
    if overflow {
        panic!(concat!("Range out of bounds: exclusive from ",
                stringify!(std::usize::MAX)));
    }
    iplus1
}

#[cfg(test)]
mod tests {
    use std::ops::Bound;
    use std::ops::Range;
    use util::enumerate;
    use util::ptrdistance;
    use util::slice_by;

    const LEN: usize = 15;

    fn mkarray() -> [usize; LEN] {
        let mut arr  = [0; LEN];
        for (i, elt) in enumerate(&mut arr) {
            *elt = i as usize;
        }
        arr
    }

    fn assert_range(start: &[usize], slice: &[usize], r: Range<usize>) {
        assert_eq!(slice.len(), r.len());
        if r.len() > 0 {
            assert_eq!(slice[0], r.start);
        }
        assert_eq!(r.start, ptrdistance(start.as_ptr(), slice.as_ptr()));
    }

    #[test]
    fn range() {
        let data = &mkarray()[..];
        let s = slice_by(data, ..);
        assert_range(data, s, 0..LEN);

        for i in 0..LEN {
            for j in i..LEN {
                let s = slice_by(data, i..j);
                assert_range(data, s, i..j);
                
                if j + 1 < LEN {
                    let s = slice_by(data, i..=j);
                    assert_range(data, s, i..j + 1);
                }
            }

            let s = slice_by(data, i..);
            assert_range(data, s, i..LEN);

            let s = slice_by(data, ..i);
            assert_range(data, s, 0..i);
        }
    }

    #[test]
    fn bound() {
        let data = &mkarray()[..];
        let s = slice_by(data, (Bound::Excluded(1), Bound::Unbounded));
        assert_range(data, s, 2..LEN);

        let s = slice_by(data, (Bound::Excluded(1), Bound::Excluded(2)));
        assert_range(data, s, 2..2);

        let s = slice_by(data, (Bound::Excluded(1), Bound::Excluded(3)));
        assert_range(data, s, 2..3);

        let s = slice_by(data, (Bound::Excluded(1), Bound::Included(1)));
        assert_range(data, s, 2..2);

        let s = slice_by(data, (Bound::Excluded(1), Bound::Included(2)));
        assert_range(data, s, 2..3);
    }

    #[test]
    #[should_panic="out of bounds"]
    fn exclusive_start_at_the_end() {
        let data = &mut mkarray()[..];
        slice_by(data, (Bound::Excluded(!0), Bound::Unbounded));
    }

    #[test]
    #[should_panic]
    fn exclusive_start_before_end() {
        let data = &mkarray()[..];
        slice_by(data, (Bound::Excluded(1), Bound::Excluded(1)));
    }

    #[test]
    #[should_panic]
    fn inclusive_end_at_length() {
        let data = &mkarray()[..];
        slice_by(data, 0..=LEN);
    }
}
