#![doc = include_str!("../README.md")]

pub mod diagnostics;
pub mod expect;
pub mod hierarchy;
pub mod query;
pub mod system;

#[deprecated(since = "0.2.6")]
pub mod future;

pub mod prelude {
    pub use crate::expect::Expect;
    pub use crate::query::{FromQuery, Get};
    pub use crate::system::*;
}

/// Wrapper for [`disqualified::ShortName`] since it was removed from Bevy standard.
///
/// This avoids the need to add a dependency on [`disqualified`] if you're already using `moonshine` crates.
pub fn get_short_name(name: &str) -> String {
    disqualified::ShortName(name).to_string()
}
