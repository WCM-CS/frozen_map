use std::{hash::Hash, mem::MaybeUninit};
use ph::{
    BuildDefaultSeededHasher, 
    phast::{
        DefaultCompressedArray, Function2, Params,
        ShiftOnlyWrapped, bits_per_seed_to_100_bucket_size
    }, 
    seeds::{BitsFast}
};

use bitvec::bitvec;

use crate::index::{prelude::*};
use crate::store::prelude::*;


// SyncUnverifiedFrozenMap  // lowest overhead //not thread safe // no key verification


#[repr(C)]
pub struct SyncUnverifiedFrozenMap<K, V> 
where 
    K: Hash + Eq + Send + Sync + Clone + Default,
    V: Send + Sync + Clone + Default
{
    index: UnverifiedIndex<K>,
    store: SyncStore<V>
}


impl<K, V> SyncUnverifiedFrozenMap<K, V> 
where 
    K: Hash + Eq + Send + Sync + Clone + Default,
    V: Send + Sync + Clone + Default
{
    #[inline]
    pub fn unsafe_init(keys: Vec<K>, values: Vec<V>) -> Result<Self, &'static str> { // only use if the key value pair indexes line up properly
        if keys.len() != values.len() {
            return Err("KEY-VALUE: Index allignment issue, cannot built Froyo")
        }
        //assert_eq!(keys.len(), values.len(), "The values and keys vectors where not the same length aka indexing issues");

        // Build PHast+ MPHF
        let index_map: Function2<BitsFast, ShiftOnlyWrapped::<3>, DefaultCompressedArray, BuildDefaultSeededHasher> = Function2::with_slice_p_threads_hash_sc(
            &keys, 
            &Params::new(BitsFast(8), bits_per_seed_to_100_bucket_size(8)), 
            std::thread::available_parallelism().map_or(1, |v| v.into()), 
            BuildDefaultSeededHasher::default(), 
            ShiftOnlyWrapped::<3>
        );

        // NO need to build key vector we are not using it here

        // build values vector
        let mut sorted_values: Vec<MaybeUninit<V>> = Vec::with_capacity(keys.len()); // allocated memory for n elemens
        unsafe { sorted_values.set_len(keys.len());} // changes the actual length of the vec to n length without any overhead

        // Build Bloom Filter
        let mut init_bloom = bitvec![0; keys.len()];

        keys.iter().zip(values.into_iter()).for_each(|(key, value)| {
            let idx = index_map.get(&key);

            sorted_values[idx].write(value);
            init_bloom.set(idx, true);
        });

        let frozen_index = UnverifiedIndex {
            mphf: index_map,
            keys: NoKeys::new(keys.len())
        };

        let store = SyncStore::new(sorted_values, init_bloom);

        Ok(Self {
            index: frozen_index,
            store
        })
    }

    #[inline]
    pub fn from_vec(keys: Vec<K>) -> Self {
        let index_map: Function2<BitsFast, ShiftOnlyWrapped::<3>, DefaultCompressedArray, BuildDefaultSeededHasher> = Function2::with_slice_p_threads_hash_sc(
            &keys, 
            &Params::new(BitsFast(8), bits_per_seed_to_100_bucket_size(8)), 
            std::thread::available_parallelism().map_or(1, |v| v.into()), 
            BuildDefaultSeededHasher::default(), 
            ShiftOnlyWrapped::<3>
        );

        //let mut sorted_keys = vec![K::default(); keys.len()]; 
        // note this is expensive to double allocate keys for no good reason aka allocating a default just know the type then we overwrite it which is slow

         // No need to Build keys vector

        // build values vector
        let mut sorted_values: Vec<MaybeUninit<V>> = Vec::with_capacity(keys.len()); // allocated memory for n elemens
        unsafe { sorted_values.set_len(keys.len()); } // changes the actual length of the vec to n length without any overhead


        let init_bloom = bitvec![0; keys.len()];

        // No need to populate either keys or values

        let frozen_index = UnverifiedIndex {
            mphf: index_map,
            keys: NoKeys::new(keys.len())
        };

        let store = SyncStore::new(sorted_values, init_bloom);

        Self {
            index: frozen_index,
            store
        }
    }

    #[inline]
    pub fn get(&self, key: &K) -> Option<&V> {
        let idx = self.index.get_index(key);
        self.store.get_value(idx)
    }

    #[inline]
    pub fn upsert(&mut self, key: K, value: V) {
        let idx = self.index.get_index(&key);
        self.store.update(idx, value); 
        // i this replaced an old value return the old value
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.index.keys.len()
    }

}