use super::*;

/// A read-only view of an `OrderMap`.
///
/// Views are not just references into the map --
/// they can also be split into smaller views!
pub struct OrderMapView<'a, K: 'a, V: 'a, S: 'a = RandomState> {
    map: &'a OrderMap<K, V, S>,

    /// the index range that's covered by this view
    start: usize,
    end: usize,
}

impl<'a, K, V, S> Clone for OrderMapView<'a, K, V, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, K, V, S> Copy for OrderMapView<'a, K, V, S> {}

impl<'a, K, V, S> fmt::Debug for OrderMapView<'a, K, V, S>
    where K: fmt::Debug + Hash + Eq,
          V: fmt::Debug,
          S: BuildHasher,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<'a, K, V, S> OrderMapView<'a, K, V, S> {
    /// Creates a read-only view of a map.
    pub fn new(map: &OrderMap<K, V, S>) -> OrderMapView<K, V, S> {
        OrderMapView {
            map: map,

            start: 0,
            end: map.entries.len(),
        }
    }

    /// Return the number of key-value pairs in the map view.
    ///
    /// Computes in **O(1)** time.
    pub fn len(&self) -> usize { self.end - self.start }

    /// Returns true if the map view contains no elements.
    ///
    /// Computes in **O(1)** time.
    pub fn is_empty(&self) -> bool { self.len() == 0 }

    /// Return a reference to the map's `BuildHasher`.
    pub fn hasher(&self) -> &'a S
        where S: BuildHasher
    {
        self.map.hasher()
    }

    fn entries_iter(&self) -> SliceIter<'a, Bucket<K, V>> {
        self.map.entries[self.start .. self.end].iter()
    }

    fn contains_index(&self, index: usize) -> bool {
        (self.start <= index) && (index < self.end)
    }
}

impl<'a, K, V, S> OrderMapView<'a, K, V, S>
    where K: Hash + Eq,
          S: BuildHasher,
{
    /// Return an iterator over the key-value pairs of the map view, in their order
    pub fn iter(&self) -> Iter<'a, K, V> {
        Iter {
            iter: self.entries_iter(),
        }
    }

    /// Return an iterator over the keys of the map view, in their order
    pub fn keys(&self) -> Keys<'a, K, V> {
        Keys {
            iter: self.entries_iter(),
        }
    }

    /// Return an iterator over the values of the map view, in their order
    pub fn values(&self) -> Values<'a, K, V> {
        Values {
            iter: self.entries_iter(),
        }
    }

    /// Return `true` if and equivalent to `key` exists in the map view.
    ///
    /// Computes in **O(1)** time (average).
    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
        where Q: Hash + Equivalent<K>,
    {
        if let Some((_, found)) = self.map.find(key) {
            self.contains_index(found)
        } else {
            false
        }
    }

    /// Return a reference to the value stored for `key`, if it is present in
    /// this map view, else `None`.
    ///
    /// Computes in **O(1)** time (average).
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&'a V>
        where Q: Hash + Equivalent<K>,
    {
        self.get_full(key).map(third)
    }

    /// Return item index, key and value
    pub fn get_full<Q: ?Sized>(&self, key: &Q) -> Option<(usize, &'a K, &'a V)>
        where Q: Hash + Equivalent<K>,
    {
        if let Some((index, key, value)) = self.map.get_full(key) {
            if self.contains_index(index) {
                return Some((index - self.start, key, value));
            }
        }
        None
    }
}

impl<'a, K, V, S> OrderMapView<'a, K, V, S> {
    /// Get a key-value pair by index
    ///
    /// Valid indices are *0 <= index < self.len()*
    ///
    /// Computes in **O(1)** time.
    pub fn get_index(&self, index: usize) -> Option<(&'a K, &'a V)> {
        if index < self.len() {
            self.map.get_index(self.start + index)
        } else {
            None
        }
    }

    /// Divides a view into two at an index.
    ///
    /// ***Panics*** if `mid > self.len()`
    pub fn split_at(self, mid: usize) -> (Self, Self) {
        assert!(mid <= self.len());
        let mid = self.start + mid;
        (OrderMapView { end: mid, ..self },
         OrderMapView { start: mid, ..self })
    }

    /// Returns the first key-value pair and a view of all the rest,
    /// or `None` if it is empty.
    pub fn split_first(self) -> Option<(&'a K, &'a V, Self)> {
        if self.start < self.end {
            self.map.get_index(self.start)
                .map(|(k, v)| (k, v, OrderMapView { start: self.start + 1, ..self }))
        } else {
            None
        }
    }

    /// Returns the last key-value pair and a view of all the rest,
    /// or `None` if it is empty.
    pub fn split_last(self) -> Option<(&'a K, &'a V, Self)> {
        if self.start < self.end {
            self.map.get_index(self.end - 1)
                .map(|(k, v)| (k, v, OrderMapView { end: self.end - 1, ..self }))
        } else {
            None
        }
    }
}

impl<'a, K, V, S> IntoIterator for OrderMapView<'a, K, V, S>
    where K: Hash + Eq,
          S: BuildHasher,
{
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, 'q, K, V, Q: ?Sized, S> Index<&'q Q> for OrderMapView<'a, K, V, S>
    where Q: Hash + Equivalent<K>,
          K: Hash + Eq,
          S: BuildHasher,
{
    type Output = V;

    /// ***Panics*** if `key` is not present in the map view.
    fn index(&self, key: &Q) -> &V {
        if let Some(v) = self.get(key) {
            v
        } else {
            panic!("OrderMap: key not found")
        }
    }
}

impl<'a, 'b, K, V1, S1, V2, S2> PartialEq<OrderMapView<'b, K, V2, S2>> for OrderMapView<'a, K, V1, S1>
    where K: Hash + Eq,
          V1: PartialEq<V2>,
          S1: BuildHasher,
          S2: BuildHasher
{
    fn eq(&self, other: &OrderMapView<K, V2, S2>) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter().all(|(key, value)| other.get(key).map_or(false, |v| *value == *v))
    }
}

impl<'a, K, V, S> Eq for OrderMapView<'a, K, V, S>
    where K: Eq + Hash,
          V: Eq,
          S: BuildHasher
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut map = OrderMap::new();
        assert_eq!(map.view().is_empty(), true);
        map.insert(1, ());
        map.insert(1, ());
        let view = map.view();
        assert_eq!(view.len(), 1);
        assert!(view.get(&1).is_some());
        assert_eq!(view.is_empty(), false);
    }

    #[test]
    fn new() {
        let map = OrderMap::<String, String>::new();
        let view = map.view();
        println!("{:?}", view);
        assert_eq!(view.len(), 0);
        assert_eq!(view.is_empty(), true);
    }

    #[test]
    fn insert_order() {
        let insert = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];
        let mut map = OrderMap::new();

        for &elt in &insert {
            map.insert(elt, ());
        }

        let view = map.view();
        assert_eq!(view.keys().count(), view.len());
        assert_eq!(view.keys().count(), insert.len());
        for (a, b) in insert.iter().zip(view.keys()) {
            assert_eq!(a, b);
        }
        for (i, k) in (0..insert.len()).zip(view.keys()) {
            assert_eq!(view.get_index(i).unwrap().0, k);
        }
    }

    #[test]
    fn partial_eq_and_eq() {
        let mut map_a = OrderMap::new();
        map_a.insert(1, "1");
        map_a.insert(2, "2");
        let mut map_b = map_a.clone();
        assert_eq!(map_a.view(), map_b.view());
        map_b.remove(&1);
        assert_ne!(map_a.view(), map_b.view());

        let map_c: OrderMap<_, String> = map_b.into_iter().map(|(k, v)| (k, v.to_owned())).collect();
        assert_ne!(map_a.view(), map_c.view());
        assert_ne!(map_c.view(), map_a.view());
    }

    #[test]
    fn splits() {
        fn check(view: OrderMapView<i32, ()>, slice: &[i32], other: &[i32]) {
            assert_eq!(view.len(), slice.len());
            assert_eq!(view.iter().count(), slice.len());
            assert_eq!(view.keys().count(), slice.len());
            assert_eq!(view.values().count(), slice.len());
            for (a, b) in view.keys().zip(slice) {
                assert_eq!(a, b);
            }
            for x in slice {
                assert!(view.contains_key(x));
                assert_eq!(view.get(x), Some(&()));
                let _ = &view[x];
            }
            for x in other {
                assert!(!view.contains_key(x));
            }
        }

        fn check_split_at(view: OrderMapView<i32, ()>, slice: &[i32], index: usize) {
            let (slice1, slice2) = slice.split_at(index);
            let (view1, view2) = view.split_at(index);
            check(view1, slice1, slice2);
            check(view2, slice2, slice1);
        }

        let insert = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];

        let mut map = OrderMap::new();
        for &elt in &insert {
            map.insert(elt, ());
        }

        let view = map.view();
        for i in 0..insert.len() + 1 {
            check_split_at(view, &insert, i);
        }

        let (first, slice) = insert.split_first().unwrap();
        let (k, &(), tail) = view.split_first().unwrap();
        assert_eq!(k, first);
        check(tail, slice, &[*first]);

        let (last, slice) = insert.split_last().unwrap();
        let (k, &(), tail) = view.split_last().unwrap();
        assert_eq!(k, last);
        check(tail, slice, &[*last]);
    }
}
