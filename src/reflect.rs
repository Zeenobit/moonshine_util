use bevy_reflect::{GetTypeRegistration, Typed};

/// Convenient alias for [`GetTypeRegistration`] + [`Typed`].
///
/// # Usage
///
/// When implementing generic plugins that use [`register_type`](bevy_app::App::register_type),
/// it is often required for types to implement [`GetTypeRegistration`] and [`Typed`].
///
/// This is a convenient alias to avoid having to explicitly import these traits when dealing with
/// generic code:
///
/// ```rust
/// use bevy::prelude::*;
/// use moonshine_util::prelude::*;
///
/// // Easy and clean! :D
/// fn register_component_plugin<A: Registerable + Component>(app: &mut App) {
///     app.register_type::<A>();
/// }
/// ```
pub trait Registerable: GetTypeRegistration + Typed {}

impl<T: GetTypeRegistration + Typed> Registerable for T {}
