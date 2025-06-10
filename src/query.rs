//! Utilities and decorators for working with queries.

use std::marker::PhantomData;

use bevy_ecs::archetype::Archetype;
use bevy_ecs::component::{ComponentId, Components, Tick};
use bevy_ecs::prelude::*;
use bevy_ecs::query::{FilteredAccess, QueryData, ReadOnlyQueryData, WorldQuery};
use bevy_ecs::storage::{Table, TableRow};

/// A trait for types that can be constructed from query data.
///
/// See [`Get`] for more information on usage.
pub trait MapQuery {
    /// The query type which this type can be constructed from.
    type Query: ReadOnlyQueryData;

    /// The output type that this query will map to.
    type Output;

    /// Called at the time of query execution to map the query data into `Self`.
    fn map(data: <Self::Query as QueryData>::Item<'_>) -> Self::Output;
}

/// A query decorator which maps some query data into `T` using [`FromQuery`].
///
/// This is useful for when you want to compute a processed value from some query data.
///
/// # Example
///
/// ```rust
/// use bevy::prelude::*;
/// use moonshine_util::prelude::*;
///
/// struct Height;
///
/// impl MapQuery for Height {
///     type Query = &'static GlobalTransform;
///
///     type Output = f32;
///
///     fn map(data: &GlobalTransform) -> f32 {
///         data.translation().y
///     }
/// }
///
/// fn average_height(query: Query<Get<Height>>) -> f32 {
///     let mut total_height = 0.0;
///     let mut count = 0;
///     for h in query.iter() {
///         total_height += h;
///         count += 1;
///     }
///
///     total_height / count as f32
/// }
///
/// # bevy_ecs::system::assert_is_system(average_height);
/// ```
pub struct Get<T>(PhantomData<T>);

unsafe impl<T: MapQuery> WorldQuery for Get<T> {
    type Fetch<'a> = <T::Query as WorldQuery>::Fetch<'a>;

    type State = <T::Query as WorldQuery>::State;

    fn shrink_fetch<'wlong: 'wshort, 'wshort>(fetch: Self::Fetch<'wlong>) -> Self::Fetch<'wshort> {
        T::Query::shrink_fetch(fetch)
    }

    unsafe fn init_fetch<'w>(
        world: bevy_ecs::world::unsafe_world_cell::UnsafeWorldCell<'w>,
        state: &Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Fetch<'w> {
        unsafe { T::Query::init_fetch(world, state, last_run, this_run) }
    }

    const IS_DENSE: bool = T::Query::IS_DENSE;

    unsafe fn set_archetype<'w>(
        fetch: &mut Self::Fetch<'w>,
        state: &Self::State,
        archetype: &'w Archetype,
        table: &'w Table,
    ) {
        unsafe { T::Query::set_archetype(fetch, state, archetype, table) }
    }

    unsafe fn set_table<'w>(fetch: &mut Self::Fetch<'w>, state: &Self::State, table: &'w Table) {
        unsafe { T::Query::set_table(fetch, state, table) }
    }

    fn update_component_access(state: &Self::State, access: &mut FilteredAccess<ComponentId>) {
        T::Query::update_component_access(state, access)
    }

    fn init_state(world: &mut World) -> Self::State {
        T::Query::init_state(world)
    }

    fn get_state(components: &Components) -> Option<Self::State> {
        T::Query::get_state(components)
    }

    fn matches_component_set(
        state: &Self::State,
        set_contains_id: &impl Fn(ComponentId) -> bool,
    ) -> bool {
        T::Query::matches_component_set(state, set_contains_id)
    }
}

unsafe impl<T: MapQuery> QueryData for Get<T> {
    type ReadOnly = Self;

    const IS_READ_ONLY: bool = <T::Query as QueryData>::IS_READ_ONLY;

    type Item<'a> = T::Output;

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::Item<'wlong>) -> Self::Item<'wshort> {
        item
    }

    unsafe fn fetch<'w>(
        fetch: &mut Self::Fetch<'w>,
        entity: Entity,
        table_row: TableRow,
    ) -> Self::Item<'w> {
        T::map(T::Query::fetch(fetch, entity, table_row))
    }
}

unsafe impl<T: MapQuery> ReadOnlyQueryData for Get<T> {}
