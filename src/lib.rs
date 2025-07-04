#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

pub mod diagnostics;
pub mod expect;
pub mod hierarchy;
pub mod query;
pub mod spawn;
pub mod system;

pub mod prelude {
    //! Prelude module to import the most essential utilities.

    pub use crate::expect::Expect;
    pub use crate::hierarchy::WithChild;
    pub use crate::query::{Get, MapQuery};
    pub use crate::spawn::SpawnUnrelated;
}

/// Wrapper for [`disqualified::ShortName`] since it was removed from Bevy standard.
///
/// This avoids the need to add a dependency on [`disqualified`] if you're already using `moonshine` crates.
pub fn get_short_name(name: &str) -> String {
    disqualified::ShortName(name).to_string()
}

pub fn get_short_type_name<T>() -> String {
    get_short_name(std::any::type_name::<T>())
}
