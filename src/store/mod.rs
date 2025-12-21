pub mod sync_store;
pub mod atomic_store;

pub use sync_store::*;
pub use atomic_store::*;


// Prelude for easy import in maps
pub mod prelude {
    pub use crate::store::sync_store::SyncStore;
    pub use crate::store::atomic_store::AtomicStore;
}