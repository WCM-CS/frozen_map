use std::{hash::Hash, mem::MaybeUninit, sync::Arc};
use ph::{
    BuildDefaultSeededHasher, 
    phast::{
        DefaultCompressedArray, Function2, Params, SeedOnly, ShiftOnlyWrapped, bits_per_seed_to_100_bucket_size
    }, 
    seeds::BitsFast
};

use crate::index::{prelude::*};
use crate::store::prelude::*;


//use crate::{KeyStorage, UnverifiedIndex, AtomicStore, NoKeys};

// AtomicUnverifiedFrozenMap  // medium overhead // is thread safe // no keys stored


pub struct AtomicUnverifiedFrozenMap<K, V> 
where 
    K: Hash + Eq + Send + Sync + Clone + Default,
    V: Send + Sync + Clone + Default,
{
    index: UnverifiedIndex<K>,
    store: AtomicStore<V>
}


impl<K, V> AtomicUnverifiedFrozenMap<K, V> 
where 
    K: Hash + Eq + Send + Sync + Clone + Default,
    V: Send + Sync + Clone + Default
{
    #[inline]
    pub fn from_vec(keys: Vec<K> ) -> Self {
        let index_map: Function2<BitsFast, ShiftOnlyWrapped::<2>, DefaultCompressedArray, BuildDefaultSeededHasher> = Function2::with_slice_p_threads_hash_sc(
            &keys, 
            &Params::new(BitsFast(8), bits_per_seed_to_100_bucket_size(8)), 
            std::thread::available_parallelism().map_or(1, |v| v.into()), 
            BuildDefaultSeededHasher::default(), 
            ShiftOnlyWrapped::<2>
        );

        //let mut sorted_keys = vec![K::default(); keys.len()]; 
        // note this is expensive to double allocate keys for no good reason aka allocating a default just know the type then we overwrite it which is slow

         // No need to Build keys vector

        // build values vector
        let mut sorted_values: Vec<MaybeUninit<V>> = Vec::with_capacity(keys.len()); // allocated memory for n elemens
        unsafe { sorted_values.set_len(keys.len()); } // changes the actual length of the vec to n length without any overhead


       // let init_bloom = bitvec![0; keys.len()];

        // No need to populate either keys or values

        let frozen_index = UnverifiedIndex {
            mphf: index_map,
            keys: NoKeys::new(keys.len())
        };

        let store = AtomicStore::new(keys.len());

   //     let jj = Value

        Self {
            index: frozen_index,
            store: store
        }
    }

    #[inline]
    pub fn get(&self, key: &K) -> Option<Arc<V>> {
        let idx = self.index.get_index(key);

        if self.index.keys.dead_key(idx) {
            return None
        } 
        
        self.store.get_value(idx)
        
    }

    #[inline]
    pub fn upsert(&self, key: K, value: V) {
        let idx = self.index.get_index(&key);

        if !self.index.keys.dead_key(idx) {
            self.store.update(idx, value); 
        }
    }


    // delete the value
    #[inline]
    pub fn drop_value(&self, key: &K) {
        let idx = self.index.get_index(&key);

        self.store.remove_value(idx);
    }

    #[inline]
    pub fn reap_key(&mut self, key: &K) -> Result<(), &str> {

        let idx = self.index.get_index(&key);

        if self.index.keys.dead_key(idx) {
            return Err("Key is already dead")
        }

        self.index.keys.kill(idx);
        Ok(())
    }



    #[inline]
    pub fn len(&self) -> usize {
        self.index.keys.len()
    }

}