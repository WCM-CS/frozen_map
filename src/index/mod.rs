pub mod index;
pub use index::*;

pub mod prelude {
    pub use crate::index::{NoKeys, WithKeys, KeyStorage, UnverifiedIndex, VerifiedIndex};
}

