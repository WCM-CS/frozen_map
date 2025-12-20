use std::{hash::Hash, marker::PhantomData, mem::MaybeUninit};
use ph::{
    BuildDefaultSeededHasher, 
    phast::{
        DefaultCompressedArray, Function, Function2, Params, Perfect, SeedChooser, 
        SeedOnly, ShiftOnly, ShiftOnlyWrapped, bits_per_seed_to_100_bucket_size
    }, 
    seeds::{Bits, Bits8, BitsFast}
};


type Index = Function2<BitsFast, ShiftOnlyWrapped::<3>, DefaultCompressedArray, BuildDefaultSeededHasher>;

pub type VerifiedIndex<K> = FrozenIndex<WithKeys<K>>;
pub type UnverifiedIndex<K> = FrozenIndex<NoKeys<K>>;

#[repr(C)]
pub struct FrozenIndex<S>
where
    S: KeyStorage,
    S::Key: Hash + Eq + Clone + Send + Sync + Default,
{
    pub mphf: Index,
    pub keys: S
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
        let x = self.keys.get(idx);

        if x == key {
            true
        } else {
            false
        }
    }
}


pub trait KeyStorage {
    type Key;

    fn get(&self, idx: usize) -> &Self::Key;
    fn len(&self) -> usize;
}

#[repr(transparent)]
pub struct WithKeys<K> {
    keys: Vec<MaybeUninit<K>>
}

impl<K> WithKeys<K> {
    pub fn new(keys: Vec<MaybeUninit<K>>) -> Self {
        Self { keys }
    }
}

pub struct NoKeys<K> {
    _ghost: PhantomData<K>,
    len: usize
}

impl<K> NoKeys<K> {
    pub fn new(len: usize) -> Self {
        Self {
            _ghost: PhantomData,
            len,
        }
    }
}

impl<K> KeyStorage for WithKeys<K> {
    type Key = K;

    #[inline]
    fn get(&self, idx: usize) -> &K {
        unsafe { self.keys[idx].assume_init_ref() }
    }

    #[inline]
    fn len(&self) -> usize {
        self.keys.len()
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
}

impl<K> Drop for WithKeys<K> {
    fn drop(&mut self) {
        unsafe {
            for k in &mut self.keys {
                k.assume_init_drop();
            }
        }
    }
}