use std::{mem::MaybeUninit};
use bitvec::{ vec::BitVec };


pub struct SyncStore<V>
where
    V: Send + Sync + Clone + Default,
{
    values: ValueStruct<V>,
    init: BitVec,
}

impl<V> SyncStore<V> 
where
    V: Send + Sync + Clone + Default,
{
    #[inline]
    pub fn new(values: Vec<MaybeUninit<V>>, init: BitVec) -> Self {
        Self {
            values: ValueStruct::new(values),
            init
        }
    }

    #[inline]
    pub fn update(&mut self, idx: usize, value: V) {
        if self.init[idx] {
            unsafe { 
                std::ptr::drop_in_place(self.values.inner[idx].as_mut_ptr()); 
            }
        } 

        self.values.inner[idx].write(value);
        self.init.set(idx, true);
    }

    #[inline]
    pub fn remove_value(&mut self, idx: usize) {
        if self.init[idx] {
            unsafe {
                std::ptr::drop_in_place(self.values.inner[idx].as_mut_ptr());
            }
            self.init.set(idx, false);
        }
    }

    #[inline]
    pub fn get_value(&self, idx: usize) -> Option<&V> {
        if self.init[idx] {
            let v = unsafe { self.values.inner[idx].assume_init_ref() };
            Some(v)
        } else {
            None
        }
    }
}

impl<V> Drop for SyncStore<V> 
where
    V: Send + Sync + Clone + Default,
{
    fn drop(&mut self) {
        for (i, initialized) in self.init.iter().enumerate() {
            if *initialized {
                unsafe {
                    self.values.inner[i].assume_init_drop();
                }
            }
        }
    }
}


#[repr(transparent)]
pub struct ValueStruct<V>
where
    V: Send + Sync + Clone + Default,
{
    inner: Vec<MaybeUninit<V>>,
}

impl<V> ValueStruct<V> 
where
    V: Send + Sync + Clone + Default,
{
    fn new(values: Vec<MaybeUninit<V>>) -> Self {
        Self {
            inner: values,
        }
    }
}
