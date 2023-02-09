#[cfg(feature = "merk-full")]
use super::{MerkStore, ProofBuilder};
use crate::store::{BufStore, MapStore, NullStore, Read, Shared, Write, KV};
use crate::{Error, Result};
use merk::proofs::query::Map as ProofMap;
use std::ops::Bound;

#[cfg(feature = "merk-full")]
type WrappedMerkStore = Shared<BufStore<Shared<BufStore<Shared<MerkStore>>>>>;

#[derive(Clone)]
pub enum BackingStore {
    #[cfg(feature = "merk-full")]
    WrappedMerk(WrappedMerkStore),
    #[cfg(feature = "merk-full")]
    ProofBuilder(ProofBuilder),
    #[cfg(feature = "merk-full")]
    Merk(Shared<MerkStore>),
    MapStore(Shared<MapStore>),
    ProofMap(Shared<ProofStore>),
    Null(NullStore),
}

impl Default for BackingStore {
    fn default() -> Self {
        BackingStore::Null(NullStore)
    }
}

impl Read for BackingStore {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        match self {
            #[cfg(feature = "merk-full")]
            BackingStore::WrappedMerk(ref store) => store.get(key),
            #[cfg(feature = "merk-full")]
            BackingStore::ProofBuilder(ref builder) => builder.get(key),
            #[cfg(feature = "merk-full")]
            BackingStore::Merk(ref store) => store.get(key),
            BackingStore::MapStore(ref store) => store.get(key),
            BackingStore::ProofMap(ref map) => map.get(key),
            BackingStore::Null(ref null) => null.get(key),
        }
    }

    fn get_next(&self, key: &[u8]) -> Result<Option<KV>> {
        match self {
            #[cfg(feature = "merk-full")]
            BackingStore::WrappedMerk(ref store) => store.get_next(key),
            #[cfg(feature = "merk-full")]
            BackingStore::ProofBuilder(ref builder) => builder.get_next(key),
            #[cfg(feature = "merk-full")]
            BackingStore::Merk(ref store) => store.get_next(key),
            BackingStore::MapStore(ref store) => store.get_next(key),
            BackingStore::ProofMap(ref map) => map.get_next(key),
            BackingStore::Null(ref null) => null.get_next(key),
        }
    }
}

impl Write for BackingStore {
    fn put(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<()> {
        match self {
            #[cfg(feature = "merk-full")]
            BackingStore::WrappedMerk(ref mut store) => store.put(key, value),
            #[cfg(feature = "merk-full")]
            BackingStore::Merk(ref mut store) => store.put(key, value),
            #[cfg(feature = "merk-full")]
            BackingStore::ProofBuilder(_) => {
                panic!("put() is not implemented for ProofBuilder")
            }
            BackingStore::MapStore(ref mut store) => store.put(key, value),
            BackingStore::ProofMap(_) => {
                panic!("put() is not implemented for ProofMap")
            }
            BackingStore::Null(ref mut store) => store.put(key, value),
        }
    }
    fn delete(&mut self, key: &[u8]) -> Result<()> {
        match self {
            #[cfg(feature = "merk-full")]
            BackingStore::WrappedMerk(ref mut store) => store.delete(key),
            #[cfg(feature = "merk-full")]
            BackingStore::Merk(ref mut store) => store.delete(key),
            #[cfg(feature = "merk-full")]
            #[cfg(feature = "merk-full")]
            BackingStore::ProofBuilder(_) => {
                panic!("delete() is not implemented for ProofBuilder")
            }
            BackingStore::MapStore(ref mut store) => store.delete(key),
            BackingStore::ProofMap(_) => {
                panic!("delete() is not implemented for ProofMap")
            }
            BackingStore::Null(ref mut store) => store.delete(key),
        }
    }
}

impl BackingStore {
    #[cfg(feature = "merk-full")]
    pub fn into_proof_builder(self) -> Result<ProofBuilder> {
        match self {
            #[cfg(feature = "merk-full")]
            BackingStore::ProofBuilder(builder) => Ok(builder),
            _ => Err(Error::Downcast(
                "Failed to downcast backing store to proof builder".into(),
            )),
        }
    }

    #[cfg(feature = "merk-full")]
    pub fn into_wrapped_merk(self) -> Result<WrappedMerkStore> {
        match self {
            #[cfg(feature = "merk-full")]
            BackingStore::WrappedMerk(store) => Ok(store),
            _ => Err(Error::Downcast(
                "Failed to downcast backing store to wrapped merk".into(),
            )),
        }
    }

    pub fn into_map_store(self) -> Result<Shared<MapStore>> {
        match self {
            BackingStore::MapStore(store) => Ok(store),
            _ => Err(Error::Downcast(
                "Failed to downcast backing store to map store".into(),
            )),
        }
    }

    #[cfg(feature = "merk-full")]
    pub fn use_merkstore<F: FnOnce(&MerkStore) -> T, T>(&self, f: F) -> T {
        let wrapped_store = match self {
            BackingStore::WrappedMerk(_store) => todo!(),
            BackingStore::Merk(store) => store,
            _ => panic!("Cannot get MerkStore from BackingStore variant"),
        };

        let store = wrapped_store.borrow();

        f(&store)
    }
}

#[cfg(feature = "merk-full")]
impl From<WrappedMerkStore> for BackingStore {
    fn from(store: WrappedMerkStore) -> BackingStore {
        BackingStore::WrappedMerk(store)
    }
}

#[cfg(feature = "merk-full")]
impl From<Shared<MerkStore>> for BackingStore {
    fn from(store: Shared<MerkStore>) -> BackingStore {
        let builder = ProofBuilder::new(store);
        BackingStore::ProofBuilder(builder)
    }
}

impl From<Shared<MapStore>> for BackingStore {
    fn from(store: Shared<MapStore>) -> BackingStore {
        BackingStore::MapStore(store)
    }
}

pub struct ProofStore(pub ProofMap);

impl Read for ProofStore {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let maybe_value = self.0.get(key)?;
        Ok(maybe_value.map(|value| value.to_vec()))
    }

    fn get_next(&self, key: &[u8]) -> Result<Option<KV>> {
        let mut iter = self.0.range((Bound::Excluded(key), Bound::Unbounded));
        let item = iter.next().transpose()?;
        Ok(item.map(|(k, v)| (k.to_vec(), v.to_vec())))
    }
}
