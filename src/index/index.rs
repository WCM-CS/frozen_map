use std::{hash::Hash, marker::PhantomData, mem::MaybeUninit, sync::atomic::{AtomicBool, AtomicUsize, Ordering}};
use ph::{
    BuildDefaultSeededHasher, 
    phast::{DefaultCompressedArray, Function2, ShiftOnlyWrapped}, 
    seeds::{BitsFast}
};
use bumpalo::{Bump};
use bitvec::{ vec::BitVec, bitvec };


type Index = Function2<BitsFast, ShiftOnlyWrapped::<3>, DefaultCompressedArray, BuildDefaultSeededHasher>;

pub type VerifiedIndex<K> = FrozenIndex<WithKeys<K>>;
pub type UnverifiedIndex<K> = FrozenIndex<NoKeys<K>>;


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
    fn kill(&self, idx: usize);
    fn rehydrate(&self, idx: usize);
    fn dead_key(&self, idx: usize) -> bool;
}


pub struct WithKeys<K> {
    #[allow(dead_code)]
    _arena_handle: Bump,
    keys_ptr: *const [K],
    len: AtomicUsize,
    tombstone: Vec<AtomicBool>
}

impl<K> WithKeys<K> 
where  
    K: Hash + Eq + Send + Sync + Clone + Default,
{
    pub fn new_from_uninit(keys: Vec<MaybeUninit<K>>) -> Self {
        let arena = Bump::new();
        let arena_keys: &mut [K] = arena.alloc_slice_fill_with(keys.len(), |i| unsafe {
            keys[i].assume_init_read() // moves the value out of MaybeUninit
        });

        let keys_ptr = arena_keys as *const [K];
        let tombstone = (0..keys.len())
            .map(|_| AtomicBool::new(true))
            .collect::<Vec<_>>();

        Self {
            _arena_handle: arena,
            keys_ptr,
            len: AtomicUsize::new(keys.len()),
            tombstone
        }
    }
}

// should these be repr c structs?

pub struct NoKeys<K> {
    _ghost: PhantomData<K>,
    len: AtomicUsize,
    tombstone: Vec<AtomicBool>
}

impl<K> NoKeys<K> {
    pub fn new(len: usize) -> Self {
        let tombstone = (0..len)
            .map(|_| AtomicBool::new(true))
            .collect::<Vec<_>>();
        
        Self {
            _ghost: PhantomData,
            len: AtomicUsize::new(len),
            tombstone
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
        self.len.load(Ordering::Acquire)
    }

    #[inline]
    fn kill(&self, idx: usize) {
        if self.tombstone[idx].swap(false, Ordering::AcqRel) {
            self.len.fetch_sub(1, Ordering::AcqRel); // only decrease count if it was dead before
        }
    }

    #[inline]
    fn rehydrate(&self, idx: usize) {
        if !self.tombstone[idx].swap(true, Ordering::AcqRel) {
            self.len.fetch_add(1, Ordering::AcqRel);
        }
    }

    #[inline]
    fn dead_key(&self, idx: usize) -> bool {
        !self.tombstone[idx].load(Ordering::Acquire)
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
        self.len.load(Ordering::Acquire)
    }

    #[inline]
    fn kill(&self, idx: usize) {
        if self.tombstone[idx].swap(false, Ordering::AcqRel) {
            self.len.fetch_sub(1, Ordering::AcqRel); // only decrease count if it was dead before
        }
    }

    #[inline]
    fn rehydrate(&self, idx: usize) {
        if !self.tombstone[idx].swap(true, Ordering::AcqRel) {
            self.len.fetch_add(1, Ordering::AcqRel);
        }
    }

    #[inline]
    fn dead_key(&self, idx: usize) -> bool {
        !self.tombstone[idx].load(Ordering::Acquire)
    }
}

