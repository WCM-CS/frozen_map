use ph::{
    BuildDefaultSeededHasher,
    phast::{
        DefaultCompressedArray, Function2, Params, ShiftOnlyWrapped,
        bits_per_seed_to_100_bucket_size,
    },
    seeds::BitsFast,
};
use std::{hash::Hash, mem::MaybeUninit};

use bitvec::bitvec;

use crate::index::prelude::*;
use crate::store::prelude::*;

// SyncUnverifiedFrozenMap  // lowest overhead //not thread safe // no key verification

pub struct UnsafeFrozenMap<K, V>
where
    K: Hash + Eq + Send + Sync + Clone + Default,
    V: Send + Sync + Clone + Default,
{
    index: UnverifiedIndex<K>,
    store: Store<V>,
}

impl<K, V> UnsafeFrozenMap<K, V>
where
    K: Hash + Eq + Send + Sync + Clone + Default,
    V: Send + Sync + Clone + Default,
{

    #[inline]
    pub fn unsafe_init(keys: Vec<K>, values: Vec<V>) -> Self { // only use if the key value pair indexes line up properly
        let index_map: Function2<
            BitsFast,
            ShiftOnlyWrapped<2>,
            DefaultCompressedArray,
            BuildDefaultSeededHasher,
        > = Function2::with_slice_p_threads_hash_sc(
            &keys,
            &Params::new(BitsFast(10), bits_per_seed_to_100_bucket_size(8)),
            std::thread::available_parallelism().map_or(1, |v| v.into()),
            BuildDefaultSeededHasher::default(),
            ShiftOnlyWrapped::<2>,
        );

        //let mut sorted_keys = vec![K::default(); keys.len()];
        // note this is expensive to double allocate keys for no good reason aka allocating a default just know the type then we overwrite it which is slow

        // no need to build key vector

        // build values vector
        let mut sorted_values: Vec<MaybeUninit<V>> = Vec::with_capacity(keys.len()); // allocated memory for n elemens
        unsafe {
            sorted_values.set_len(keys.len());
        } // changes the actual length of the vec to n length without any overhead

        let init_bloom = bitvec![1; keys.len()];

        keys.into_iter().zip(values.into_iter()).for_each(|(key, val)| {
            let idx = index_map.get(&key);

            //sorted_keys[idx].write(key);
            sorted_values[idx].write(val);
        });

        let frozen_index = UnverifiedIndex {
            mphf: index_map,
            keys: NoKeys::new(sorted_values.len()),
        };

        let store = Store::new(sorted_values, init_bloom);

        Self {
            index: frozen_index,
            store,
        }
    }

    #[inline]
    pub fn from_vec(keys: Vec<K>) -> Self {
        let index_map: Function2<
            BitsFast,
            ShiftOnlyWrapped<2>,
            DefaultCompressedArray,
            BuildDefaultSeededHasher,
        > = Function2::with_slice_p_threads_hash_sc(
            &keys,
            &Params::new(BitsFast(10), bits_per_seed_to_100_bucket_size(8)),
            std::thread::available_parallelism().map_or(1, |v| v.into()),
            BuildDefaultSeededHasher::default(),
            ShiftOnlyWrapped::<2>,
        );

        //let mut sorted_keys = vec![K::default(); keys.len()];
        // note this is expensive to double allocate keys for no good reason aka allocating a default just know the type then we overwrite it which is slow

        // No need to Build keys vector

        // build values vector
        let mut sorted_values: Vec<MaybeUninit<V>> = Vec::with_capacity(keys.len()); // allocated memory for n elemens
        unsafe {
            sorted_values.set_len(keys.len());
        } // changes the actual length of the vec to n length without any overhead

        let init_bloom = bitvec![0; keys.len()];

        // No need to populate either keys or values

        let frozen_index = UnverifiedIndex {
            mphf: index_map,
            keys: NoKeys::new(keys.len()),
        };

        let store = Store::new(sorted_values, init_bloom);

        Self {
            index: frozen_index,
            store,
        }
    }

    #[inline]
    pub fn get(&self, key: &K) -> Option<&V> {
        let idx = self.index.get_index(key);

        if self.index.keys.dead_key(idx) {
            return None;
        }

        self.store.get_value(idx)
    }

    #[inline]
    pub fn upsert(&mut self, key: K, value: V) {
        let idx = self.index.get_index(&key);

        if !self.index.keys.dead_key(idx) {
            self.store.update(idx, value);
        }
    }

    #[inline]
    pub fn drop_value(&mut self, key: &K) {
        let idx = self.index.get_index(key);

        self.store.remove_value(idx);
    }

    #[inline]
    pub fn reap_key(&mut self, key: &K) -> Result<(), &str> {
        let idx = self.index.get_index(key);

        if self.index.keys.dead_key(idx) {
            return Err("Key is already dead");
        }

        self.index.keys.kill(idx);
        Ok(())
    }

    #[inline]
    pub fn rehydrate_key(&mut self, key: &K) -> Result<(), &str> {
        let idx = self.index.get_index(key);

        if !self.index.keys.dead_key(idx) {
            return Err("Key is already alive");
        }

        self.index.keys.rehydrate(idx);
        Ok(())
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.index.keys.len()
    }
}
