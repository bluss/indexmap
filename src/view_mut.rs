use super::*;

/// A mutable view of an `OrderMap`.
///
/// Mutable views are not just mutable references into the map --
/// they can also be split into smaller views!
pub struct OrderMapViewMut<'a, K: 'a, V: 'a, S: 'a = RandomState> {
    offset: usize,
    mask: usize,
    indices: &'a [Pos],
    entries: &'a mut [Bucket<K, V>],
    hash_builder: &'a S,
    size_class_is_64bit: bool,
}

impl<'a, K, V, S> fmt::Debug for OrderMapViewMut<'a, K, V, S>
    where K: fmt::Debug + Hash + Eq,
          V: fmt::Debug,
          S: BuildHasher,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<'a, K, V, S> OrderMapViewMut<'a, K, V, S> {
    /// Creates a read-only view of a map.
    pub fn new(map: &mut OrderMap<K, V, S>) -> OrderMapViewMut<K, V, S> {
        OrderMapViewMut {
            offset: 0,
            mask: map.mask,
            indices: &map.indices,
            hash_builder: &map.hash_builder,
            size_class_is_64bit: map.size_class_is_64bit(),
            entries: &mut map.entries,
        }
    }

    // TODO pub fn view(&self) -> OrderMapView<K, V, S>

    /// Reborrow the current view.
    pub fn view_mut(&mut self) -> OrderMapViewMut<K, V, S> {
        OrderMapViewMut { entries: &mut *self.entries, ..*self }
    }

    /// Return the number of key-value pairs in the map view.
    ///
    /// Computes in **O(1)** time.
    pub fn len(&self) -> usize { self.entries.len() }

    /// Returns true if the map view contains no elements.
    ///
    /// Computes in **O(1)** time.
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }

    /// Return a reference to the map's `BuildHasher`.
    pub fn hasher(&self) -> &'a S
        where S: BuildHasher
    {
        self.hash_builder
    }

    #[inline(always)]
    fn size_class_is_64bit(&self) -> bool {
        self.size_class_is_64bit
    }
}

impl<'a, K, V, S> OrderMapViewMut<'a, K, V, S>
    where K: Hash + Eq,
          S: BuildHasher,
{
    /// Return an iterator over the key-value pairs of the map view, in their order
    pub fn iter(&self) -> Iter<K, V> {
        Iter {
            iter: self.entries.iter(),
        }
    }

    /// Return an iterator over the key-value pairs of the map, in their order
    pub fn iter_mut(&mut self) -> IterMut<K, V> {
        IterMut {
            iter: self.entries.iter_mut()
        }
    }

    /// Return an iterator over the keys of the map view, in their order
    pub fn keys(&self) -> Keys<K, V> {
        Keys {
            iter: self.entries.iter(),
        }
    }

    /// Return an iterator over the values of the map view, in their order
    pub fn values(&self) -> Values<K, V> {
        Values {
            iter: self.entries.iter(),
        }
    }

    /// Return an iterator over mutable references to the the values of the map,
    /// in their order
    pub fn values_mut(&mut self) -> ValuesMut<K, V> {
        ValuesMut {
            iter: self.entries.iter_mut()
        }
    }

    /// Return `true` if and equivalent to `key` exists in the map view.
    ///
    /// Computes in **O(1)** time (average).
    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
        where Q: Hash + Equivalent<K>,
    {
        self.find(key).is_some()
    }

    /// Return a reference to the value stored for `key`, if it is present in
    /// this map view, else `None`.
    ///
    /// Computes in **O(1)** time (average).
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
        where Q: Hash + Equivalent<K>,
    {
        self.get_full(key).map(third)
    }

    /// Return item index, key and value
    pub fn get_full<Q: ?Sized>(&self, key: &Q) -> Option<(usize, &K, &V)>
        where Q: Hash + Equivalent<K>,
    {
        if let Some((_, found)) = self.find(key) {
            let entry = &self.entries[found];
            Some((found, &entry.key, &entry.value))
        } else {
            None
        }
    }

    pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
        where Q: Hash + Equivalent<K>,
    {
        self.get_full_mut(key).map(third)
    }

    pub fn get_full_mut<Q: ?Sized>(&mut self, key: &Q)
        -> Option<(usize, &K, &mut V)>
        where Q: Hash + Equivalent<K>,
    {
        if let Some((_, found)) = self.find(key) {
            let entry = &mut self.entries[found];
            Some((found, &entry.key, &mut entry.value))
        } else {
            None
        }
    }

    /// Return probe (indices) and position (entries)
    fn find<Q: ?Sized>(&self, key: &Q) -> Option<(usize, usize)>
        where Q: Hash + Equivalent<K>,
    {
        if self.len() == 0 { return None; }
        let h = hash_elem_using(self.hash_builder, key);
        self.find_using(h, move |entry| { Q::equivalent(key, &entry.key) })
    }
}

impl<'a, K, V, S> OrderMapViewMut<'a, K, V, S> {
    /// Get a key-value pair by index
    ///
    /// Valid indices are *0 <= index < self.len()*
    ///
    /// Computes in **O(1)** time.
    pub fn get_index(&self, index: usize) -> Option<(&K, &V)> {
        self.entries.get(index).map(|ent| (&ent.key, &ent.value))
    }

    /// Get a key-value pair by index
    ///
    /// Valid indices are *0 <= index < self.len()*
    ///
    /// Computes in **O(1)** time.
    pub fn get_index_mut(&mut self, index: usize) -> Option<(&mut K, &mut V)> {
        self.entries.get_mut(index).map(|ent| (&mut ent.key, &mut ent.value))
    }

    /// Divides a view into two at an index.
    ///
    /// ***Panics*** if `mid > self.len()`
    pub fn split_at(self, mid: usize) -> (Self, Self) {
        let (left, right) = self.entries.split_at_mut(mid);
        (OrderMapViewMut { entries: left, ..self },
         OrderMapViewMut { entries: right, offset: self.offset + mid, ..self })
    }

    /// Divides a view into two at an index.
    ///
    /// ***Panics*** if `mid > self.len()`
    pub fn split_at_mut(self, mid: usize) -> (Self, Self) {
        let (left, right) = self.entries.split_at_mut(mid);
        (OrderMapViewMut { entries: left, ..self },
         OrderMapViewMut { entries: right, offset: self.offset + mid, ..self })
    }

    /// Returns the first key-value pair and a view of all the rest,
    /// or `None` if it is empty.
    pub fn split_first(self) -> Option<(&'a K, &'a V, Self)> {
        if let Some((ent, rest)) = self.entries.split_first_mut() {
            Some((&ent.key, &ent.value,
                  OrderMapViewMut { offset: self.offset + 1, entries: rest, ..self }))
        } else {
            None
        }
    }

    /// Returns the first key-value pair and a view of all the rest,
    /// or `None` if it is empty.
    pub fn split_first_mut(self) -> Option<(&'a mut K, &'a mut V, Self)> {
        if let Some((ent, rest)) = self.entries.split_first_mut() {
            Some((&mut ent.key, &mut ent.value,
                  OrderMapViewMut { offset: self.offset + 1, entries: rest, ..self }))
        } else {
            None
        }
    }

    /// Returns the last key-value pair and a view of all the rest,
    /// or `None` if it is empty.
    pub fn split_last(self) -> Option<(&'a K, &'a V, Self)> {
        if let Some((ent, rest)) = self.entries.split_last_mut() {
            Some((&ent.key, &ent.value,
                  OrderMapViewMut { entries: rest, ..self }))
        } else {
            None
        }
    }

    /// Returns the last key-value pair and a view of all the rest,
    /// or `None` if it is empty.
    pub fn split_last_mut(self) -> Option<(&'a mut K, &'a mut V, Self)> {
        if let Some((ent, rest)) = self.entries.split_last_mut() {
            Some((&mut ent.key, &mut ent.value,
                  OrderMapViewMut { entries: rest, ..self }))
        } else {
            None
        }
    }
}

// Methods that don't use any properties (Hash / Eq) of K.
//
// It's cleaner to separate them out, then the compiler checks that we are not
// using Hash + Eq at all in these methods.
//
// However, we should probably not let this show in the public API or docs.
impl<'a, K, V, S> OrderMapViewMut<'a, K, V, S> {
    /// Return probe (indices) and position (entries)
    fn find_using<F>(&self, hash: HashValue, key_eq: F) -> Option<(usize, usize)>
        where F: Fn(&Bucket<K, V>) -> bool,
    {
        dispatch_32_vs_64!(self.find_using_impl::<_>(hash, key_eq))
    }

    fn find_using_impl<Sz, F>(&self, hash: HashValue, key_eq: F) -> Option<(usize, usize)>
        where F: Fn(&Bucket<K, V>) -> bool,
              Sz: Size,
    {
        debug_assert!(self.len() > 0);
        let mut probe = desired_pos(self.mask, hash);
        let mut dist = 0;
        probe_loop!(probe < self.indices.len(), {
            if let Some((i, hash_proxy)) = self.indices[probe].resolve::<Sz>() {
                // Unfortunately, we can only look at indexes within our slice
                // of entries, which might mean we'll keep probing farther than
                // it would be strictly necessary.
                if (self.offset <= i) && (i < self.offset + self.entries.len()) {
                    let i = i - self.offset;
                    let entry_hash = hash_proxy.get_short_hash(&self.entries, i);
                    if dist > probe_distance(self.mask, entry_hash.into_hash(), probe) {
                        // give up when probe distance is too long
                        return None;
                    } else if entry_hash == hash && key_eq(&self.entries[i]) {
                        return Some((probe, i));
                    }
                }
            } else {
                return None;
            }
            dist += 1;
        });
    }
}

impl<'a, K, V, S> IntoIterator for OrderMapViewMut<'a, K, V, S>
    where K: Hash + Eq,
          S: BuildHasher,
{
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        IterMut {
            iter: self.entries.iter_mut()
        }
    }
}

impl<'a, 'q, K, V, Q: ?Sized, S> Index<&'q Q> for OrderMapViewMut<'a, K, V, S>
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

/// Mutable indexing allows changing / updating values of key-value
/// pairs that are already present.
///
/// You can **not** insert new pairs with index syntax, use `.insert()`.
impl<'a, 'q, K, V, Q: ?Sized, S> IndexMut<&'q Q> for OrderMapViewMut<'a, K, V, S>
    where Q: Hash + Equivalent<K>,
          K: Hash + Eq,
          S: BuildHasher,
{
    /// ***Panics*** if `key` is not present in the map view.
    fn index_mut(&mut self, key: &Q) -> &mut V {
        if let Some(v) = self.get_mut(key) {
            v
        } else {
            panic!("OrderMap: key not found")
        }
    }
}

impl<'a, 'b, K, V1, S1, V2, S2> PartialEq<OrderMapViewMut<'b, K, V2, S2>> for OrderMapViewMut<'a, K, V1, S1>
    where K: Hash + Eq,
          V1: PartialEq<V2>,
          S1: BuildHasher,
          S2: BuildHasher
{
    fn eq(&self, other: &OrderMapViewMut<K, V2, S2>) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.iter().all(|(key, value)| other.get(key).map_or(false, |v| *value == *v))
    }
}

impl<'a, K, V, S> Eq for OrderMapViewMut<'a, K, V, S>
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
        assert_eq!(map.view_mut().is_empty(), true);
        map.insert(1, ());
        map.insert(1, ());
        let view = map.view_mut();
        assert_eq!(view.len(), 1);
        assert!(view.get(&1).is_some());
        assert_eq!(view.is_empty(), false);
    }

    #[test]
    fn new() {
        let mut map = OrderMap::<String, String>::new();
        let view = map.view_mut();
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

        let view = map.view_mut();
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
        assert_eq!(map_a.view_mut(), map_b.view_mut());
        map_b.remove(&1);
        assert_ne!(map_a.view_mut(), map_b.view_mut());

        let mut map_c: OrderMap<_, String> = map_b.into_iter().map(|(k, v)| (k, v.to_owned())).collect();
        assert_ne!(map_a.view_mut(), map_c.view_mut());
        assert_ne!(map_c.view_mut(), map_a.view_mut());
    }

    #[test]
    fn splits() {
        fn check(mut view: OrderMapViewMut<i32, ()>, slice: &[i32], other: &[i32]) {
            assert_eq!(view.len(), slice.len());
            assert_eq!(view.iter().count(), slice.len());
            assert_eq!(view.keys().count(), slice.len());
            assert_eq!(view.values().count(), slice.len());
            assert_eq!(view.values_mut().count(), slice.len());
            for (a, b) in view.keys().zip(slice) {
                assert_eq!(a, b);
            }
            for x in slice {
                assert!(view.contains_key(x));
                assert_eq!(view.get_mut(x), Some(&mut ()));
                let _ = &mut view[x];
            }
            for x in other {
                assert!(!view.contains_key(x));
            }
        }

        fn check_split_at(view: OrderMapViewMut<i32, ()>, slice: &[i32], index: usize) {
            let (slice1, slice2) = slice.split_at(index);
            let (view1, view2) = view.split_at_mut(index);
            check(view1, slice1, slice2);
            check(view2, slice2, slice1);
        }

        let insert = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];

        let mut map = OrderMap::new();
        for &elt in &insert {
            map.insert(elt, ());
        }

        for i in 0..insert.len() + 1 {
            check_split_at(map.view_mut(), &insert, i);
        }

        {
            let (first, slice) = insert.split_first().unwrap();
            let (k, &mut (), tail) = map.view_mut().split_first_mut().unwrap();
            assert_eq!(k, first);
            check(tail, slice, &[*first]);
        }

        {
            let (last, slice) = insert.split_last().unwrap();
            let (k, &mut (), tail) = map.view_mut().split_last_mut().unwrap();
            assert_eq!(k, last);
            check(tail, slice, &[*last]);
        }
    }
}
