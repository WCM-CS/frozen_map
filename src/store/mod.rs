pub mod store;
pub use store::*;

// Prelude for easy import in maps
pub mod prelude {
    pub use crate::store::store::Store;
}
