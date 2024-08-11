#![doc = include_str!("../README.md")]

pub mod app;
pub mod diagnostics;
pub mod expect;
pub mod future;
pub mod hierarchy;
pub mod system;

pub mod prelude {
    pub use crate::{
        app::{AddPluginFn, AddPluginFnToGroup, FnPlugin},
        expect::Expect,
        system::*,
    };
}
