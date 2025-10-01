#![doc = include_str!("../README.md")]
#![warn(missing_docs)]

pub mod component;
pub mod defer;
pub mod diagnostics;
pub mod event;
pub mod expect;
pub mod hierarchy;
pub mod query;
pub mod reflect;
pub mod spawn;
pub mod system;

pub mod prelude {
    //! Prelude module to import the most essential utilities.

    pub use crate::component::{Merge, MergeComponent, MergeFrom, MergeWith};
    pub use crate::defer::{run_deferred_systems, RunDeferredSystem};
    pub use crate::event::{AddSingleObserver, OnSingle, SingleEvent, TriggerSingle};
    pub use crate::expect::Expect;
    pub use crate::query::{Get, MapQuery};
    pub use crate::reflect::Registerable;
    pub use crate::spawn::{SpawnUnrelated, WithChild};
    pub use crate::Static;

    pub use crate::relationship;
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

/// Convenient alias for `'static + Send + Sync` because recently I've started mumbling
/// `'static + Send + Sync` in my sleep. My doctor has recommended I use this trait instead.
///
/// And so should you! It's clinically proven to reduce stress and wrist tension.
pub trait Static: 'static + Send + Sync {}

impl<T: 'static + Send + Sync> Static for T {}
