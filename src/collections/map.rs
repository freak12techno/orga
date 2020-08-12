use crate::{Decode, Encode, Result, State, Store};
use std::borrow::Borrow;
use std::marker::PhantomData;

/// A map data structure.
pub struct Map<S, K, V>
where
    S: Store,
    K: Encode + Decode,
    V: Encode + Decode,
{
    store: S,
    key_type: PhantomData<K>,
    value_type: PhantomData<V>,
}

impl<S, K, V> State<S> for Map<S, K, V>
where
    S: Store,
    K: Encode + Decode,
    V: Encode + Decode,
{
    fn wrap_store(store: S) -> Result<Self> {
        Ok(Self {
            store,
            key_type: PhantomData,
            value_type: PhantomData,
        })
    }
}

impl<S, K, V> Map<S, K, V>
where
    S: Store,
    K: Encode + Decode,
    V: Encode + Decode,
{
    pub fn insert(&mut self, key: K, value: V) -> Result<()> {
        let key_bytes = key.encode()?;
        let value_bytes = value.encode()?;
        self.store.put(key_bytes, value_bytes)
    }

    pub fn delete<B: Borrow<K>>(&mut self, key: B) -> Result<()> {
        let (key_bytes, key_length) = encode_key_array(key.borrow())?;
        self.store.delete(&key_bytes[..key_length])
    }

    pub fn get<B: Borrow<K>>(&self, key: B) -> Result<Option<V>> {
        let (key_bytes, key_length) = encode_key_array(key.borrow())?;
        self.store
            .get(&key_bytes[..key_length])?
            .map(|value_bytes| V::decode(value_bytes.as_slice()))
            .transpose()
    }
}

impl<'a, 'b: 'a, S, K, V> Map<S, K, V>
where
    S: Store + crate::Iter<'a, 'b>,
    K: Encode + Decode,
    V: Encode + Decode,
{
    pub fn iter_from(&'a self, start: &K) -> Result<Iter<'a, 'b, S::Iter, K, V>> {
        let start_bytes = start.encode()?;
        let iter = self.store.iter_from(start_bytes.as_slice());
        Ok(Iter::new(iter))
    }

    pub fn iter(&'a self) -> Iter<'a, 'b, S::Iter, K, V> {
        let iter = self.store.iter();
        Iter::new(iter)
    }
}

pub struct Iter<'a, 'b: 'a, I, K, V>
where
    I: Iterator<Item = (&'b [u8], &'b [u8])>,
    K: Decode,
    V: Decode,
{
    iter: I,
    phantom_a: PhantomData<&'a u8>,
    phantom_k: PhantomData<K>,
    phantom_v: PhantomData<V>,
}

impl<'a, 'b: 'a, I, K, V> Iter<'a, 'b, I, K, V>
where
    I: Iterator<Item = (&'b [u8], &'b [u8])>,
    K: Decode,
    V: Decode,
{
    fn new(iter: I) -> Self {
        Iter {
            iter,
            phantom_a: PhantomData,
            phantom_k: PhantomData,
            phantom_v: PhantomData,
        }
    }
}

impl<'a, 'b: 'a, I, K, V> Iterator for Iter<'a, 'b, I, K, V>
where
    I: Iterator<Item = (&'b [u8], &'b [u8])>,
    K: Decode,
    V: Decode,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(key, value)| (K::decode(key).unwrap(), V::decode(value).unwrap()))
    }
}

fn encode_key_array<K: Encode>(key: &K) -> Result<([u8; 256], usize)> {
    let mut bytes = [0; 256];
    key.encode_into(&mut &mut bytes[..])?;
    Ok((bytes, key.encoding_length()?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn simple() {
        let mut store = MapStore::new();
        let mut map: Map<_, u64, u64> = Map::wrap_store(&mut store).unwrap();

        assert_eq!(map.get(1234).unwrap(), None);

        map.insert(1234, 5678).unwrap();
        assert_eq!(map.get(1234).unwrap(), Some(5678));

        map.delete(1234).unwrap();
        assert_eq!(map.get(1234).unwrap(), None);
    }

    #[test]
    fn iter() {
        let store = MapStore::new();
        let mut map: Map<_, u64, u64> = Map::wrap_store(store).unwrap();

        map.insert(123, 456).unwrap();
        map.insert(100, 100).unwrap();
        map.insert(400, 100).unwrap();

        let mut iter = map.iter_from(&101).unwrap();
        assert_eq!(iter.next(), Some((123, 456)));
        assert_eq!(iter.next(), Some((400, 100)));
        assert_eq!(iter.next(), None);
    }
}
