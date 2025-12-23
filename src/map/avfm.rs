use std::{hash::{BuildHasher, Hash}, mem::MaybeUninit, sync::Arc};
use ph::{
    BuildDefaultSeededHasher, BuildSeededHasher, phast::{
        DefaultCompressedArray, Function2, Params, SeedChooser, SeedOnly, ShiftOnlyWrapped, bits_per_seed_to_100_bucket_size
    }, seeds::BitsFast
};
//use ph::BuildSeededHasher;       // trait for the builder
use fasthash::FastHasher;       // trait for the hasher itself
use std::hash::Hasher;

use crate::index::{prelude::*};
use crate::store::prelude::*;

// AtomicVerifiedFrozenMap   // highest overhead // thread safe // key verification


pub struct AtomicVerifiedFrozenMap<K, V> 
where 
    K: Hash + Eq + Send + Sync + Clone + Default,
    V: Send + Sync + Clone + Default
{
    index: VerifiedIndex<K>,
    store: AtomicStore<V>
}


// only use if the key value pair indexes line up properly
impl<K, V> AtomicVerifiedFrozenMap<K, V> 
where 
    K: Hash + Eq + Send + Sync + Clone + Default,
    V: Send + Sync + Clone + Default
{

    #[inline]
    pub fn from_vec(keys: Vec<K>) -> Self {
     
        //let hashes: Vec<u64> = keys.iter().map(|k|XXHasher(k)).collect();

        let index_map: Function2<BitsFast, ShiftOnlyWrapped::<2>, DefaultCompressedArray, BuildDefaultSeededHasher> = Function2::with_slice_p_threads_hash_sc(
            &keys, 
            &Params::new(BitsFast(8), bits_per_seed_to_100_bucket_size(8)), 
            std::thread::available_parallelism().map_or(1, |v| v.into()), 
            BuildDefaultSeededHasher::default(), 
            ShiftOnlyWrapped::<2>
        );



        //let mut sorted_keys = vec![K::default(); keys.len()]; 
        // note this is expensive to double allocate keys for no good reason aka allocating a default just know the type then we overwrite it which is slow

         // Build keys vector
        let mut sorted_keys: Vec<MaybeUninit<K>> = Vec::with_capacity(keys.len());
        unsafe { sorted_keys.set_len(keys.len()); }

        // build values vector
        let mut sorted_values: Vec<MaybeUninit<V>> = Vec::with_capacity(keys.len()); // allocated memory for n elemens
        unsafe { sorted_values.set_len(keys.len()); } // changes the actual length of the vec to n length without any overhead

       // let init_bloom = bitvec![0; keys.len()];

        keys.iter().for_each(|key| {
            let idx = index_map.get(&key);
            sorted_keys[idx].write(key.clone());
        });

        let frozen_index = VerifiedIndex {
            mphf: index_map,
            keys: WithKeys::new_from_uninit(sorted_keys)
        };

        let store = AtomicStore::new(keys.len());
        
        Self {
            index: frozen_index,
            store
        }
    }

    #[inline]
    pub fn get(&self, key: &K) -> Option<Arc<V>> {
        let idx = self.index.get_index(key);

        if self.index.keys.dead_key(idx) {
            return None;
        }

        if self.index.keys.get(idx) != key {
            return None;
        }

        self.store.get_value(idx)
    }

    #[inline]
    pub fn contains(&self, key: &K) -> bool {
        self.index.contains_key(key)
    }

    #[inline]
    pub fn upsert(&self, key: K, value: V) -> Result<(), &str> {
        let idx = self.index.get_index(&key);

        if self.index.keys.dead_key(idx) {
            return Err("Dead key")
        }

        if self.index.keys.get(idx) == &key {
            self.store.update(idx, value);
            return Ok(())
        } else {
            return Err("Failed to upsert, key does not exist")
        }
    }

    // delete the value
    #[inline]
    pub fn drop_value(&self, key: &K) -> Result<(), &str> {
        let idx = self.index.get_index(&key);

        if self.index.keys.get(idx) == key {
            self.store.remove_value(idx);
            return Ok(())
        } else {
            return Err("Failed to drop value, key does not exist")
        }
    }

    #[inline]
    pub fn reap_key(&mut self, key: &K) -> Result<(), &str> {

        let idx = self.index.get_index(&key);

        if self.index.keys.dead_key(idx) {
            return Err("Key is already dead")
        }

        if self.index.keys.get(idx) == key {
            self.index.keys.kill(idx);
            return Ok(())
        } else {
            return Err("Failed to kill key, key does not exist")
        }
    }

    #[inline]
    pub fn rehydrate(&mut self, key: &K) -> Result<(), &str> {

        let idx = self.index.get_index(&key);

        if !self.index.keys.dead_key(idx) {
            return Err("Key is already alive")
        }

        if self.index.keys.get(idx) == key {
            self.index.keys.rehydrate(idx);
            return Ok(())
        } else {
            return Err("Failed to kill key, key does not exist")
        }
    }



    #[inline]
    pub fn len(&self) -> usize {
        self.index.keys.len()
    }

}
