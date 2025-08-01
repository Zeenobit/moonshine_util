#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

pub mod diagnostics;
pub mod event;
pub mod expect;
pub mod hierarchy;
pub mod query;
pub mod spawn;
pub mod system;

pub mod prelude {
    //! Prelude module to import the most essential utilities.

    pub use crate::event::{AddSingleObserver, SingleEvent, SingleTrigger, TriggerSingle};
    pub use crate::expect::Expect;
    pub use crate::query::{Get, MapQuery};
    pub use crate::spawn::{SpawnUnrelated, WithChild};
}

/// Wrapper for [`disqualified::ShortName`] since it was removed from Bevy standard.
///
/// This avoids the need to add a dependency on [`disqualified`] if you're already using `moonshine` crates.
pub fn get_short_name(name: &str) -> String {
    disqualified::ShortName(name).to_string()
}

/// Convenient wrapper for [`get_short_name`] which infers the type name from the type parameter.
pub fn get_short_type_name<T>() -> String {
    get_short_name(std::any::type_name::<T>())
}
