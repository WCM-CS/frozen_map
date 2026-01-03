use ph::{
    BuildDefaultSeededHasher,
    phast::{DefaultCompressedArray, Function2, ShiftOnlyWrapped},
    seeds::BitsFast,
};
use std::{hash::Hash, marker::PhantomData, mem::MaybeUninit};

use bitvec::{bitvec, vec::BitVec};

type Mphf =
    Function2<BitsFast, ShiftOnlyWrapped<2>, DefaultCompressedArray, BuildDefaultSeededHasher>;

pub type VerifiedIndex<K> = FrozenIndex<WithKeys<K>>;
pub type UnverifiedIndex<K> = FrozenIndex<NoKeys<K>>;

pub struct FrozenIndex<S>
where
    S: KeyStorage,
    S::Key: Hash + Eq + Clone + Send + Sync + Default,
{
    pub mphf: Mphf,
    pub keys: S,
}

impl<S> FrozenIndex<S>
where
    S: KeyStorage,
    S::Key: Hash + Eq + Clone + Send + Sync + Default,
{
    #[inline]
    pub fn get_index(&self, key: &S::Key) -> usize {
        self.mphf.get(key)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.keys.len()
    }
}

impl<K> FrozenIndex<WithKeys<K>>
where
    K: Hash + Eq + Clone + Send + Sync + Default,
{
    #[inline]
    pub fn contains_key(&self, key: &K) -> bool {
        let idx = self.get_index(key);

        if self.keys.dead_key(idx) {
            return false;
        }

        self.keys.get(idx) == key
    }
}

pub trait KeyStorage {
    type Key;

    fn get(&self, idx: usize) -> &Self::Key;
    fn len(&self) -> usize;
    fn kill(&mut self, idx: usize);
    fn rehydrate(&mut self, idx: usize);
    fn dead_key(&self, idx: usize) -> bool;
}

pub struct WithKeys<K> {
    keys: Box<[K]>,
    len: usize,
    tombstone: BitVec,
}

impl<K> WithKeys<K>
where
    K: Hash + Eq + Send + Sync + Clone + Default,
{
    pub fn new_from_uninit(keys: Vec<MaybeUninit<K>>) -> Self {
        let n = keys.len();

        let keys_k: Box<[K]> = keys // fixed size heap alloc for keys
            .into_iter()
            .map(|maybe| unsafe { maybe.assume_init() })
            .collect::<Vec<K>>()
            .into_boxed_slice();

        let tombstone = bitvec![0; n];

        Self {
            keys: keys_k,
            len: n,
            tombstone,
        }
    }

    pub fn get_keys(&self) -> Vec<K> {
        self.keys.to_vec()
    }
}

// should these be repr c structs?

pub struct NoKeys<K> {
    _ghost: PhantomData<K>,
    len: usize,
    tombstone: BitVec,
}

impl<K> NoKeys<K> {
    pub fn new(len: usize) -> Self {
        let tombstone = bitvec![0; len];

        Self {
            _ghost: PhantomData,
            len,
            tombstone,
        }
    }
}

impl<K> KeyStorage for WithKeys<K> {
    type Key = K;

    #[inline]
    fn get(&self, idx: usize) -> &K {
        &self.keys[idx]
    }

    #[inline]
    fn len(&self) -> usize {
        self.len
    }

    #[inline]
    fn kill(&mut self, idx: usize) {
        if !self.tombstone[idx] {
            self.tombstone.set(idx, true);
            self.len -= 1;
        }
    }

    #[inline]
    fn rehydrate(&mut self, idx: usize) {
        if self.tombstone[idx] {
            self.tombstone.set(idx, false);
            self.len += 1;
        }
    }

    #[inline]
    fn dead_key(&self, idx: usize) -> bool {
        self.tombstone[idx]
    }
}

impl<K> KeyStorage for NoKeys<K> {
    type Key = K;

    #[inline]
    fn get(&self, _: usize) -> &K {
        unreachable!("unverified index does not store keys")
    }

    #[inline]
    fn len(&self) -> usize {
        self.len
    }

    #[inline]
    fn kill(&mut self, idx: usize) {
        if !self.tombstone[idx] {
            self.tombstone.set(idx, true);
            self.len -= 1;
        }
    }

    #[inline]
    fn rehydrate(&mut self, idx: usize) {
        if self.tombstone[idx] {
            self.tombstone.set(idx, false);
            self.len += 1;
        }
    }

    #[inline]
    fn dead_key(&self, idx: usize) -> bool {
        self.tombstone[idx]
    }
}
