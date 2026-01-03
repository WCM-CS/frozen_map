use std::{hash::Hash, mem::MaybeUninit};
use ph::{
    BuildDefaultSeededHasher, 
    phast::{
        DefaultCompressedArray, Function2, Params, ShiftOnlyWrapped, bits_per_seed_to_100_bucket_size
    }, 
    seeds::BitsFast
};
use bitvec::bitvec;

use crate::index::{prelude::*};
use crate::store::prelude::*;


//  SyncVerifiedFrozenMap    // higher overhead // no thread safe // key verification

pub struct FrozenMap<K, V> 
where 
    K: Hash + Eq + Send + Sync + Clone + Default,
    V: Send + Sync + Clone + Default
{
    index: VerifiedIndex<K>,
    store: Store<V>
}



// only use if the key value pair indexes line up properly
impl<K, V> FrozenMap<K, V> 
where 
    K: Hash + Eq + Send + Sync + Clone + Default,
    V: Send + Sync + Clone + Default
{


    #[inline]// encode the keys outside of this call idealy
    pub fn from_vec(keys: Vec<K>) -> Self {
        let index_map: Function2<BitsFast, ShiftOnlyWrapped::<2>, DefaultCompressedArray, BuildDefaultSeededHasher> = Function2::with_slice_p_threads_hash_sc(
            &keys, 
            &Params::new(BitsFast(10), bits_per_seed_to_100_bucket_size(8)), 
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

        let init_bloom = bitvec![0; keys.len()];

        keys.into_iter().for_each(|key| {
            let idx = index_map.get(&key);

            sorted_keys[idx].write(key);
        });

        let frozen_index = VerifiedIndex {
            mphf: index_map,
            keys: WithKeys::new_from_uninit(sorted_keys)
        };

        let store = Store::new(sorted_values, init_bloom);
        
        Self {
            index: frozen_index,
            store
        }
    }

    
    #[inline]
    pub fn get(&self, key: &K) -> Option<&V> {
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
    pub fn contains_value(&self, key: &K) -> bool {
        let idx = self.index.get_index(key);
        if self.store.get_value(idx).is_none() {
           false 
        } else {
            true
        }
    }

    #[inline]
    pub fn upsert(&mut self, key: K, value: V) -> Result<(), &str>{
        let idx = self.index.get_index(&key);

        if self.index.keys.dead_key(idx) {
            return Err("Dead key")
        }

        if self.index.keys.get(idx) == &key {
            self.store.update(idx, value);
            Ok(())
        } else {
            Err("Failed to upsert, key does not exist")
        }
    }

    #[inline]
    pub fn drop_value(&mut self, key: &K) -> Result<(), &str> {
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
    pub fn rehydrate_key(&mut self, key: &K) -> Result<(), &str> {
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

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (K, V)> {
        self.index.keys.get_keys().into_iter().zip(self.store.get_values().into_iter()).filter_map(|(k, v)| v.map(|v| (k, v)))
    }

    #[inline]
    pub fn iter_keys(&self) -> impl Iterator<Item = K> {
        self.index.keys.get_keys().into_iter()
    }

}
