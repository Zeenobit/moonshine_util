//! Utility systems for generic system pipeline construction.

use bevy_ecs::prelude::*;
use bevy_ecs::query::QueryFilter;

use crate::Static;

/// A [`System`] which checks if a given resource exists.
///
/// # Example
/// ```
/// use bevy::prelude::*;
/// use moonshine_util::system::has_resource;
///
/// #[derive(Resource)]
/// struct R;
///
/// fn system(resource: Option<Res<R>>) {
///     assert!(resource.is_some());
/// }
///
/// let mut app = App::new();
/// app.add_plugins(MinimalPlugins);
/// app.add_systems(Update, system.run_if(has_resource::<R>));
/// app.update(); // If resource does't exist (it doesn't), system will panic!
/// ```
pub fn has_resource<T: Resource>(resource: Option<Res<T>>) -> bool {
    resource.is_some()
}

/// A [`System`] which checks if any event of a given type has been dispatched.
///
/// # Example
/// ```
/// use bevy::prelude::*;
/// use moonshine_util::system::has_message;
///
/// #[derive(Message)]
/// struct M;
///
/// fn system(mut reader: MessageReader<M>) {
///     assert!(reader.read().next().is_some());
/// }
///
/// let mut app = App::new();
/// app.add_plugins(MinimalPlugins).add_message::<M>();
/// app.add_systems(Update, system.run_if(has_message::<M>));
/// app.update(); // If event isn't dispatched (it wasn't), system will panic!
/// ```
pub fn has_message<T: Message>(events: MessageReader<T>) -> bool {
    !events.is_empty()
}

/// A [`System`] which removes a resource from the world using [`Commands`].
///
/// # Example
/// ```
/// use bevy::prelude::*;
/// use moonshine_util::system::{remove_resource, has_resource};
///
/// #[derive(Resource)]
/// struct R;
///
/// let mut app = App::new();
/// app.insert_resource(R);
/// app.add_plugins(MinimalPlugins);
/// app.add_systems(Update, remove_resource::<R>.run_if(has_resource::<R>));
/// app.update(); // Resource is removed from the world.
///
/// assert!(!app.world().contains_resource::<R>());
/// ```
pub fn remove_resource<T: Resource>(mut commands: Commands) {
    commands.remove_resource::<T>();
}

/// A [`System`] which removes a resource from the world immediately.
///
/// # Example
/// ```
/// use bevy::prelude::*;
/// use moonshine_util::system::{remove_resource_immediate, has_resource};
///
/// #[derive(Resource)]
/// struct R;
///
/// let mut app = App::new();
/// app.insert_resource(R);
/// app.add_plugins(MinimalPlugins);
/// app.add_systems(Update, remove_resource_immediate::<R>.run_if(has_resource::<R>));
/// app.update(); // Resource is removed from the world.
///
/// assert!(!app.world().contains_resource::<R>());
/// ```
pub fn remove_resource_immediate<T: Resource>(world: &mut World) {
    world.remove_resource::<T>();
}

/// A [`System`] which removes all instances of a given [`Component`].
///
/// # Example
/// ```
/// use bevy::prelude::*;
/// use moonshine_util::system::remove_all_components;
///
/// #[derive(Component)]
/// struct T;
///
/// let mut app = App::new();
/// app.add_plugins(MinimalPlugins);
/// app.add_systems(Update, remove_all_components::<T>);
/// let entity = app.world_mut().spawn(T).id();
/// app.update();
///
/// assert!(!app.world().entity(entity).contains::<T>());
/// ```
#[deprecated(note = "use `remove_components` instead")]
pub fn remove_all_components<T: Component>(query: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &query {
        commands.entity(entity).remove::<T>();
    }
}

/// A [`System`] which removes all instances of a given [`Component`].
///
/// # Example
/// ```
/// use bevy::prelude::*;
/// use moonshine_util::system::remove_components;
///
/// #[derive(Component)]
/// struct T;
///
/// let mut app = App::new();
/// app.add_plugins(MinimalPlugins);
/// app.add_systems(Update, remove_components::<T>);
/// let entity = app.world_mut().spawn(T).id();
/// app.update();
///
/// assert!(!app.world().entity(entity).contains::<T>());
/// ```
pub fn remove_components<T: Component>(query: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &query {
        commands.entity(entity).remove::<T>();
    }
}

/// A [`System`] which removes all instances of a given [`Component`] from all entities matching the given [`QueryFilter`].
///
/// # Example
/// ```
/// use bevy::prelude::*;
/// use moonshine_util::system::filter_remove_components;
///
/// #[derive(Component)]
/// struct T;
///
/// #[derive(Component)]
/// struct Marker;
///
/// let mut app = App::new();
/// app.add_plugins(MinimalPlugins);
/// app.add_systems(Update, filter_remove_components::<With<Marker>, T>);
/// let entity = app.world_mut().spawn((T, Marker)).id();
/// app.update();
///
/// assert!(!app.world().entity(entity).contains::<T>());
/// ```
pub fn filter_remove_components<F: QueryFilter, T: Component>(
    query: Query<Entity, (With<T>, F)>,
    mut commands: Commands,
) {
    for entity in &query {
        commands.entity(entity).remove::<T>();
    }
}

/// Returns a [`System`] which replaces all instances of a given [`Component`] with a clone of the provided value.
///
/// # Example
/// ```
/// use bevy::prelude::*;
/// use moonshine_util::system::replace_components;
///
/// #[derive(Component, Clone, PartialEq, Debug)]
/// struct T(i32);
///
/// let mut app = App::new();
/// app.add_plugins(MinimalPlugins);
/// app.add_systems(Update, replace_components(T(42)));
/// let entity = app.world_mut().spawn(T(0)).id();
/// app.update();
///
/// assert_eq!(app.world().entity(entity).get::<T>().unwrap(), &T(42));
/// ```
pub fn replace_components<T: Component + Clone>(new: T) -> impl System<In = (), Out = ()> {
    IntoSystem::into_system(
        move |query: Query<Entity, With<T>>, mut commands: Commands| {
            for entity in &query {
                commands.entity(entity).insert(new.clone());
            }
        },
    )
}

/// Returns a [`System`] which replaces all instances of a given [`Component`] with a value produced by the provided function.
///
/// # Example
/// ```
/// use bevy::prelude::*;
/// use moonshine_util::system::replace_components_with;
///
/// #[derive(Component, PartialEq, Debug)]
/// struct T(i32);
///
/// let mut app = App::new();
/// app.add_plugins(MinimalPlugins);
/// app.add_systems(Update, replace_components_with(|| T(42)));
/// let entity = app.world_mut().spawn(T(0)).id();
/// app.update();
///
/// assert_eq!(app.world().entity(entity).get::<T>().unwrap(), &T(42));
/// ```
pub fn replace_components_with<T: Component>(
    new: impl Static + Fn() -> T,
) -> impl System<In = (), Out = ()> {
    IntoSystem::into_system(
        move |query: Query<Entity, With<T>>, mut commands: Commands| {
            for entity in &query {
                commands.entity(entity).insert(new());
            }
        },
    )
}

/// Returns a [`System`] which replaces all instances of a given [`Component`] with a clone of the provided value
/// on all entities matching the given [`QueryFilter`].
///
/// # Example
/// ```
/// use bevy::prelude::*;
/// use moonshine_util::system::filter_replace_components;
///
/// #[derive(Component, Clone, PartialEq, Debug)]
/// struct T(i32);
///
/// #[derive(Component)]
/// struct Marker;
///
/// let mut app = App::new();
/// app.add_plugins(MinimalPlugins);
/// app.add_systems(Update, filter_replace_components::<With<Marker>, T>(T(42)));
/// let a = app.world_mut().spawn((T(0), Marker)).id();
/// let b = app.world_mut().spawn(T(0)).id();
/// app.update();
///
/// assert_eq!(app.world().entity(a).get::<T>().unwrap(), &T(42));
/// assert_eq!(app.world().entity(b).get::<T>().unwrap(), &T(0));
/// ```
pub fn filter_replace_components<F: 'static + QueryFilter, T: Component + Clone>(
    new: T,
) -> impl System<In = (), Out = ()> {
    IntoSystem::into_system(
        move |query: Query<Entity, (With<T>, F)>, mut commands: Commands| {
            for entity in &query {
                commands.entity(entity).insert(new.clone());
            }
        },
    )
}

/// Returns a [`System`] which replaces all instances of a given [`Component`] with a value produced by the provided function
/// on all entities matching the given [`QueryFilter`].
///
/// # Example
/// ```
/// use bevy::prelude::*;
/// use moonshine_util::system::filter_replace_components_with;
///
/// #[derive(Component, PartialEq, Debug)]
/// struct T(i32);
///
/// #[derive(Component)]
/// struct Marker;
///
/// let mut app = App::new();
/// app.add_plugins(MinimalPlugins);
/// app.add_systems(Update, filter_replace_components_with::<With<Marker>, T>(|| T(42)));
/// let a = app.world_mut().spawn((T(0), Marker)).id();
/// let b = app.world_mut().spawn(T(0)).id();
/// app.update();
///
/// assert_eq!(app.world().entity(a).get::<T>().unwrap(), &T(42));
/// assert_eq!(app.world().entity(b).get::<T>().unwrap(), &T(0));
/// ```
pub fn filter_replace_components_with<F: 'static + QueryFilter, T: Component>(
    new: impl Static + Fn() -> T,
) -> impl System<In = (), Out = ()> {
    IntoSystem::into_system(
        move |query: Query<Entity, (With<T>, F)>, mut commands: Commands| {
            for entity in &query {
                commands.entity(entity).insert(new());
            }
        },
    )
}
