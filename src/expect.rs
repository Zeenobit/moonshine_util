//! A multi-purpose tool for validating [`Component`] presence.

use std::marker::PhantomData;

use bevy_ecs::archetype::Archetype;
use bevy_ecs::component::{
    ComponentHook, ComponentId, Components, HookContext, Immutable, StorageType, Tick,
};
use bevy_ecs::prelude::*;
use bevy_ecs::query::{FilteredAccess, QueryData, ReadOnlyQueryData, WorldQuery};
use bevy_ecs::storage::{Table, TableRow};
use bevy_ecs::world::{unsafe_world_cell::UnsafeWorldCell, DeferredWorld};
use bevy_platform::collections::HashMap;

/// A [`QueryData`] decorator which panics if its inner query does not match.
///
/// # Usage
///
/// As a query parameter, this decorator is useful for preventing systems from silently skipping
/// over entities which may erroneously not match the query.
///
/// Consider the following erroneous example:
/// ```
/// use bevy::prelude::*;
///
/// #[derive(Component)]
/// struct A;
///
/// #[derive(Component)]
/// struct B;
///
/// // A and B are always expected to be inserted together:
/// #[derive(Bundle)]
/// struct AB {
///     a: A,
///     b: B,
/// }
///
/// fn bad_system(mut commands: Commands) {
///     commands.spawn(A); // Spawn A without B!
/// }
///
/// fn unsafe_system(q: Query<(&A, &B)>) {
///     for _ in q.iter() {
///         // An instance of `A` does exist.
///         // But because `A` does not exist *with* `B`, this system skips over it silently.
///     }
/// }
/// # bevy_ecs::system::assert_is_system(bad_system);
/// # bevy_ecs::system::assert_is_system(unsafe_system);
/// ````
///
/// This problem can be solved with [`Expect`]:
/// ```
/// # use bevy::prelude::*;
/// # #[derive(Component)] struct A;
/// # #[derive(Component)] struct B;
/// use moonshine_util::expect::Expect;
///
/// fn safe_system(q: Query<(&A, Expect<&B>)>) {
///     for _ in q.iter() {
///        // This system will panic if it finds an instance of `A` without `B`.
///     }
/// }
/// # bevy_ecs::system::assert_is_system(safe_system);
/// ```
///
/// ## Component Requirements
///
/// When used as a [`Component`], this decorator will panic if the given component type `T` does
/// not exist on the entity. This is especially useful as a component requirement:
///
/// ```should_panic
/// # use bevy::prelude::*;
/// # use bevy::ecs::system::RunSystemOnce;
/// use moonshine_util::expect::Expect;
///
/// #[derive(Component)]
/// struct A;
///
/// #[derive(Component)]
/// #[require(Expect<A>)]
/// struct B;
///
/// fn unsafe_system(mut commands: Commands) {
///    commands.spawn(B); // Spawn B without A! This will panic!
/// }
///
/// # bevy_ecs::system::assert_is_system(unsafe_system);
/// # let mut world = World::new();
/// # world.run_system_once(unsafe_system).unwrap();
/// ```
pub struct Expect<T>(PhantomData<T>);

impl<T: Component> Expect<T> {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        world.commands().queue(move |world: &mut World| {
            let expect = world.entity_mut(ctx.entity).take::<Self>().unwrap();
            if let Some(mut deferred) = world.get_resource_mut::<ExpectDeferred>() {
                deferred.add(ctx.entity, Box::new(expect));
                return;
            } else {
                expect.validate(world.entity(ctx.entity));
            }
        });
    }

    fn validate(self, entity: EntityRef) {
        if !entity.contains::<T>() {
            panic!(
                "expected component of type `{}` does not exist on entity {:?}",
                std::any::type_name::<T>(),
                entity.id()
            );
        }
    }
}

impl<T: Component> Component for Expect<T> {
    const STORAGE_TYPE: StorageType = StorageType::SparseSet;

    type Mutability = Immutable;

    fn on_add() -> Option<ComponentHook> {
        Some(Self::on_add)
    }
}

impl<T: Component> Default for Expect<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

trait ExpectValidate: 'static + Send + Sync {
    fn validate(self: Box<Self>, entity: EntityRef);
}

impl<T: Component> ExpectValidate for Expect<T> {
    fn validate(self: Box<Self>, entity: EntityRef) {
        (*self).validate(entity);
    }
}

/// When making many large changes to a world at once (such as when loading a saved world),
/// the execution order of [`Expect`] component requirements is not reliable, leading to false panics.
///
/// This [`Resource`] solves this problem by deferring all [`Expect`] requirement checks until [`expect_deferred`] is called.
///
/// # Usage
///
/// You may run [`expect_deferred`] as a system, or invoke it manually as required.
///
/// If you are using [Moonshine Save](https://crates.io/crates/moonshine-save),
/// [`LoadWorld`](https://docs.rs/moonshine-save/latest/moonshine_save/load/struct.LoadWorld.html)
/// manages this automatically.
#[derive(Resource, Default)]
pub struct ExpectDeferred(HashMap<Entity, Vec<Box<dyn ExpectValidate>>>);

impl ExpectDeferred {
    fn add(&mut self, entity: Entity, expect: Box<dyn ExpectValidate>) {
        self.0.entry(entity).or_default().push(expect);
    }
}

/// Call this to resolve all [`ExpectDeferred`] requirement checks and removes the resource.
///
/// # Usage
///
/// See [`ExpectDeferred`] for usage details.
///
/// If you are using [Moonshine Save](https://crates.io/crates/moonshine-save),
/// [`LoadWorld`](https://docs.rs/moonshine-save/latest/moonshine_save/load/struct.LoadWorld.html)
/// calls this automatically.
pub fn expect_deferred(world: &mut World) {
    let Some(ExpectDeferred(requirements)) = world.remove_resource::<ExpectDeferred>() else {
        return;
    };

    for (entity, expects) in requirements {
        let Ok(entity) = world.get_entity(entity) else {
            continue;
        };

        for expect in expects {
            expect.validate(entity);
        }
    }
}

#[doc(hidden)]
pub struct ExpectFetch<'w, T: WorldQuery> {
    fetch: T::Fetch<'w>,
    matches: bool,
}

impl<T: WorldQuery> Clone for ExpectFetch<'_, T> {
    fn clone(&self) -> Self {
        Self {
            fetch: self.fetch.clone(),
            matches: self.matches,
        }
    }
}

unsafe impl<T: QueryData> QueryData for Expect<T> {
    type ReadOnly = Expect<T::ReadOnly>;

    const IS_READ_ONLY: bool = true;

    type Item<'a> = T::Item<'a>;

    fn shrink<'wlong: 'wshort, 'wshort>(item: Self::Item<'wlong>) -> Self::Item<'wshort> {
        T::shrink(item)
    }

    unsafe fn fetch<'w>(
        fetch: &mut Self::Fetch<'w>,
        entity: Entity,
        table_row: TableRow,
    ) -> Self::Item<'w> {
        let item = fetch
            .matches
            .then(|| T::fetch(&mut fetch.fetch, entity, table_row));
        if let Some(item) = item {
            item
        } else {
            panic!(
                "expected query of type `{}` does not match entity {:?}",
                std::any::type_name::<T>(),
                entity
            );
        }
    }
}

unsafe impl<T: ReadOnlyQueryData> ReadOnlyQueryData for Expect<T> {}

unsafe impl<T: QueryData> WorldQuery for Expect<T> {
    type Fetch<'w> = ExpectFetch<'w, T>;
    type State = T::State;

    fn shrink_fetch<'wlong: 'wshort, 'wshort>(fetch: Self::Fetch<'wlong>) -> Self::Fetch<'wshort> {
        ExpectFetch {
            fetch: T::shrink_fetch(fetch.fetch),
            matches: fetch.matches,
        }
    }

    const IS_DENSE: bool = T::IS_DENSE;

    #[inline]
    unsafe fn init_fetch<'w>(
        world: UnsafeWorldCell<'w>,
        state: &T::State,
        last_run: Tick,
        this_run: Tick,
    ) -> ExpectFetch<'w, T> {
        ExpectFetch {
            fetch: T::init_fetch(world, state, last_run, this_run),
            matches: false,
        }
    }

    #[inline]
    unsafe fn set_archetype<'w>(
        fetch: &mut ExpectFetch<'w, T>,
        state: &T::State,
        archetype: &'w Archetype,
        table: &'w Table,
    ) {
        fetch.matches = T::matches_component_set(state, &|id| archetype.contains(id));
        if fetch.matches {
            T::set_archetype(&mut fetch.fetch, state, archetype, table);
        }
    }

    #[inline]
    unsafe fn set_table<'w>(fetch: &mut ExpectFetch<'w, T>, state: &T::State, table: &'w Table) {
        fetch.matches = T::matches_component_set(state, &|id| table.has_column(id));
        if fetch.matches {
            T::set_table(&mut fetch.fetch, state, table);
        }
    }

    fn update_component_access(state: &T::State, access: &mut FilteredAccess<ComponentId>) {
        let mut intermediate = access.clone();
        T::update_component_access(state, &mut intermediate);
        access.extend_access(&intermediate);
    }

    fn get_state(components: &Components) -> Option<Self::State> {
        T::get_state(components)
    }

    fn init_state(world: &mut World) -> T::State {
        T::init_state(world)
    }

    fn matches_component_set(
        _state: &T::State,
        _set_contains_id: &impl Fn(ComponentId) -> bool,
    ) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use bevy_ecs::system::RunSystemOnce;

    use super::*;

    #[derive(Component)]
    struct A;

    #[derive(Component)]
    struct B;

    #[test]
    #[should_panic]
    fn expected_component() {
        let mut w = World::default();
        w.spawn(A);
        w.run_system_once(|q: Query<(&A, Expect<&B>)>| for _ in q.iter() {})
            .unwrap();
    }
}
