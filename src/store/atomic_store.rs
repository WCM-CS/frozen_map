use std::sync::Arc;
use std::{ptr};
use std::sync::atomic::{AtomicPtr, Ordering};


pub struct AtomicStore<V>
where
    V: Send + Sync + Clone + Default,
{
    values: Vec<AtomicPtr<Arc<V>>>,
}

impl<V> AtomicStore<V> 
where
    V: Send + Sync + Clone + Default,
{
    #[inline]
    pub fn new(capacity: usize) -> Self {
        let mut values = Vec::with_capacity(capacity);

        for _ in 0..capacity {
            values.push(AtomicPtr::new(ptr::null_mut()));
        }
        
        Self { values }
    }

    #[inline]
    pub fn update(&self, idx: usize, value: V) { // take a non mut ref to self for the atomic operation
        let boxed_ptr = Box::into_raw(Box::new(Arc::new(value)));
        let old_ptr = self.values[idx].swap(boxed_ptr, Ordering::AcqRel);

        if !old_ptr.is_null() {
            unsafe { drop(Box::from_raw(old_ptr)); } //drop old value
        }
    }

    #[inline]
    pub fn remove_value(&self, idx: usize) {
        let old_ptr = self.values[idx].swap(std::ptr::null_mut(), Ordering::AcqRel);
        if !old_ptr.is_null() {
            unsafe { drop(Box::from_raw(old_ptr)); } //drop old value
        }
    }

    #[inline]
    pub fn get_value(&self, idx: usize) -> Option<Arc<V>> {
        let ptr = self.values[idx].load(Ordering::Acquire);

        if ptr.is_null() {
            None
        } else {
            unsafe { Some((*ptr).clone()) }
        }
    }
}

impl<V> Drop for AtomicStore<V> 
where
    V: Send + Sync + Clone + Default,
{
    fn drop(&mut self) {
        for ap in &self.values {
            let ptr = ap.load(Ordering::Acquire);
            if !ptr.is_null() {
                unsafe { drop(Box::from_raw(ptr)); } //drop old value
            }
        }
    }
}


//SeqCst is slowest
// acquire is good for readers 
// AcqRel is good for writers