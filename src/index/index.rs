use std::{hash::Hash, marker::PhantomData, mem::MaybeUninit};
use ph::{
    BuildDefaultSeededHasher, 
    phast::{DefaultCompressedArray, Function2, ShiftOnlyWrapped}, 
    seeds::{BitsFast}
};
use bumpalo::{Bump, boxed::Box};


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

impl<'a, K> FrozenIndex<WithKeys<K>> 
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

pub struct WithKeys<K> {
    arena: Bump,
    keys_ptr: *const [K],
}

impl<K> WithKeys<K> 
where  
    K: Hash + Eq + Send + Sync + Clone + Default,
{
    pub fn new(keys: &[K]) -> Self {
        let arena = Bump::new();
        let alloc_keys = arena.alloc_slice_clone(keys);
        let keys_ptr = alloc_keys as *const [K];


        Self {
            arena,
            keys_ptr,
        }
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
        unsafe { &(*self.keys_ptr)[idx] }
    }

    #[inline]
    fn len(&self) -> usize {
        unsafe { (&(*self.keys_ptr)).len() }
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

