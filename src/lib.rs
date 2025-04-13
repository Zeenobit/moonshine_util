#![doc = include_str!("../README.md")]

pub mod diagnostics;
pub mod expect;
pub mod future;
pub mod hierarchy;
pub mod query;
pub mod system;

pub mod prelude {
    pub use crate::expect::Expect;
    pub use crate::query::{FromQuery, Get};
    pub use crate::system::*;
}

pub fn get_short_name(name: &str) -> String {
    disqualified::ShortName(name).to_string()
}
