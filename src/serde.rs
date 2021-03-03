use serde::de::value::{MapDeserializer, SeqDeserializer};
use serde::de::{
    Deserialize, Deserializer, Error, IntoDeserializer, MapAccess, SeqAccess, Visitor,
};
use serde::ser::{Serialize, Serializer};

use core::fmt::{self, Formatter};
use core::hash::{BuildHasher, Hash};
use core::marker::PhantomData;

use crate::IndexMap;
use crate::Indexable;

/// Requires crate feature `"serde"` or `"serde-1"`
impl<K, V, S, Idx> Serialize for IndexMap<K, V, S, Idx>
where
    K: Serialize + Hash + Eq,
    V: Serialize,
    S: BuildHasher,
    Idx: Indexable,
{
    fn serialize<T>(&self, serializer: T) -> Result<T::Ok, T::Error>
    where
        T: Serializer,
    {
        serializer.collect_map(self)
    }
}

struct IndexMapVisitor<K, V, S, Idx>(PhantomData<(K, V, S, Idx)>);

impl<'de, K, V, S, Idx> Visitor<'de> for IndexMapVisitor<K, V, S, Idx>
where
    K: Deserialize<'de> + Eq + Hash,
    V: Deserialize<'de>,
    S: Default + BuildHasher,
    Idx: Indexable,
{
    type Value = IndexMap<K, V, S, Idx>;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "a map")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values =
            IndexMap::with_capacity_and_hasher(map.size_hint().unwrap_or(0), S::default());

        while let Some((key, value)) = map.next_entry()? {
            values.insert(key, value);
        }

        Ok(values)
    }
}

/// Requires crate feature `"serde"` or `"serde-1"`
impl<'de, K, V, S, Idx> Deserialize<'de> for IndexMap<K, V, S, Idx>
where
    K: Deserialize<'de> + Eq + Hash,
    V: Deserialize<'de>,
    S: Default + BuildHasher,
    Idx: Indexable,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(IndexMapVisitor(PhantomData))
    }
}

impl<'de, K, V, S, Idx, E> IntoDeserializer<'de, E> for IndexMap<K, V, S, Idx>
where
    K: IntoDeserializer<'de, E> + Eq + Hash,
    V: IntoDeserializer<'de, E>,
    S: BuildHasher,
    Idx: Indexable,
    E: Error,
{
    type Deserializer = MapDeserializer<'de, <Self as IntoIterator>::IntoIter, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        MapDeserializer::new(self.into_iter())
    }
}

use crate::IndexSet;

/// Requires crate feature `"serde"` or `"serde-1"`
impl<T, S, Idx> Serialize for IndexSet<T, S, Idx>
where
    T: Serialize + Hash + Eq,
    S: BuildHasher,
    Idx: Indexable,
{
    fn serialize<Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
    where
        Se: Serializer,
    {
        serializer.collect_seq(self)
    }
}

struct IndexSetVisitor<T, S, Idx>(PhantomData<(T, S, Idx)>);

impl<'de, T, S, Idx> Visitor<'de> for IndexSetVisitor<T, S, Idx>
where
    T: Deserialize<'de> + Eq + Hash,
    S: Default + BuildHasher,
    Idx: Indexable,
{
    type Value = IndexSet<T, S, Idx>;

    fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "a set")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values =
            IndexSet::with_capacity_and_hasher(seq.size_hint().unwrap_or(0), S::default());

        while let Some(value) = seq.next_element()? {
            values.insert(value);
        }

        Ok(values)
    }
}

/// Requires crate feature `"serde"` or `"serde-1"`
impl<'de, T, S, Idx> Deserialize<'de> for IndexSet<T, S, Idx>
where
    T: Deserialize<'de> + Eq + Hash,
    S: Default + BuildHasher,
    Idx: Indexable,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(IndexSetVisitor(PhantomData))
    }
}

impl<'de, T, S, Idx, E> IntoDeserializer<'de, E> for IndexSet<T, S, Idx>
where
    T: IntoDeserializer<'de, E> + Eq + Hash,
    S: BuildHasher,
    Idx: Indexable,
    E: Error,
{
    type Deserializer = SeqDeserializer<<Self as IntoIterator>::IntoIter, E>;

    fn into_deserializer(self) -> Self::Deserializer {
        SeqDeserializer::new(self.into_iter())
    }
}
