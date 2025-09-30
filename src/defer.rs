//! In Bevy it is possible to [register] and [run] systems manually using [`SystemId`].
//!
//! While this is useful enough, sometimes it may be necessary to defer a manual system
//! execution until later during the update cycle.
//!
//! To solve this problem, you may use [`run_deferred_system`] to run a system manually in
//! any [`Schedule`]. See [`RunDeferredSystem`] for more details and examples.
//!
//! Internally, this works by managing a queue of system IDs to be executed using
//! [`run_deferred_systems`].
//!
//! [register]: bevy_ecs::world::World::register_system
//! [run]: bevy_ecs::world::World::run_system
//! [`run_deferred_system`]: RunDeferredSystem::run_deferred_system

use std::marker::PhantomData;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_ecs::schedule::ScheduleLabel;
use bevy_ecs::system::SystemId;
use bevy_ecs::world::DeferredWorld;
use bevy_log::prelude::*;

use crate::Static;

/// A [`Plugin`] which adds the [`run_deferred_systems`]
pub struct DefaultDeferredSystemsPlugin;

impl Plugin for DefaultDeferredSystemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(First, run_deferred_systems::<First>)
            .add_systems(PreUpdate, run_deferred_systems::<PreUpdate>)
            .add_systems(Update, run_deferred_systems::<Update>)
            .add_systems(PostUpdate, run_deferred_systems::<PostUpdate>)
            .add_systems(Last, run_deferred_systems::<Last>);
    }
}

/// Trait used to run deferred systems via [`World`].
pub trait RunDeferredSystem {
    /// Queues the given [`System`] for a single execution in the given [`Schedule`].
    ///
    /// # Usage
    ///
    /// You must add [`DefaultDeferredSystemsPlugin`] for deferred system execution to work
    /// in standard Bevy schedules. You may also add [`run_deferred_systems`] manually to any
    /// [`Schedule`] to provide deferred system execution support for it.
    ///
    /// # Example
    /// ```
    /// use bevy::prelude::*;
    /// use bevy::ecs::world::DeferredWorld;
    /// use bevy::ecs::component::HookContext;
    /// use moonshine_util::prelude::*;
    ///
    /// #[derive(Component)]
    /// #[component(on_insert = on_insert_foo)]
    /// struct Foo;
    ///
    /// fn on_insert_foo(mut world: DeferredWorld, ctx: HookContext) {
    ///     world.run_deferred_system(PostUpdate, |query: Query<&Foo>| {
    ///         // ...
    ///     });
    /// }
    /// ```
    fn run_deferred_system<S: ScheduleLabel, M>(
        &mut self,
        schedule: S,
        system: impl Static + IntoSystem<(), (), M>,
    );

    /// Same as [`run_deferred_system`](RunDeferredSystem::run_deferred_system), but for systems with
    /// input parameters.
    fn run_deferred_system_with<S: ScheduleLabel, I: Static, M>(
        &mut self,
        schedule: S,
        system: impl Static + IntoSystem<In<I>, (), M>,
        input: I,
    );
}

impl RunDeferredSystem for World {
    fn run_deferred_system<S: ScheduleLabel, M>(
        &mut self,
        _schedule: S,
        system: impl Static + IntoSystem<(), (), M>,
    ) {
        let system = self.register_system_cached(system);
        self.get_resource_or_init::<DeferredSystems<S>>()
            .0
            .push(Box::new(DeferredSystem(system)));
    }

    fn run_deferred_system_with<S: ScheduleLabel, I: Static, M>(
        &mut self,
        _schedule: S,
        system: impl 'static + IntoSystem<In<I>, (), M>,
        input: I,
    ) {
        let system = self.register_system_cached(system);
        self.get_resource_or_init::<DeferredSystems<S>>()
            .0
            .push(Box::new(DeferredSystemWith(system, input)));
    }
}

impl RunDeferredSystem for DeferredWorld<'_> {
    fn run_deferred_system<S: ScheduleLabel, M>(
        &mut self,
        schedule: S,
        system: impl Static + IntoSystem<(), (), M>,
    ) {
        self.commands()
            .queue(move |world: &mut World| world.run_deferred_system(schedule, system));
    }

    fn run_deferred_system_with<S: ScheduleLabel, I: Static, M>(
        &mut self,
        schedule: S,
        system: impl Static + IntoSystem<In<I>, (), M>,
        input: I,
    ) {
        self.commands().queue(move |world: &mut World| {
            world.run_deferred_system_with(schedule, system, input)
        });
    }
}

#[derive(Resource)]
struct DeferredSystems<S: ScheduleLabel>(Vec<Box<dyn AnyDeferredSystem>>, PhantomData<S>);

impl<S: ScheduleLabel> Default for DeferredSystems<S> {
    fn default() -> Self {
        Self(Vec::new(), PhantomData)
    }
}

impl<S: ScheduleLabel> DeferredSystems<S> {
    fn take(&mut self) -> Self {
        Self(self.0.drain(..).collect(), PhantomData)
    }

    fn run(self, world: &mut World) {
        for system in self.0 {
            system.run(world);
        }
    }
}

trait AnyDeferredSystem: Static {
    fn run(self: Box<Self>, world: &mut World);
}

struct DeferredSystem(SystemId);

impl AnyDeferredSystem for DeferredSystem {
    fn run(self: Box<Self>, world: &mut World) {
        if let Err(why) = world.run_system(self.0) {
            error!("deferred system error: {why}");
        }
    }
}

struct DeferredSystemWith<I: Static>(SystemId<In<I>>, I);

impl<I: Static> AnyDeferredSystem for DeferredSystemWith<I> {
    fn run(self: Box<Self>, world: &mut World) {
        if let Err(why) = world.run_system_with(self.0, self.1) {
            error!("deferred system error: {why}");
        }
    }
}

/// A [`System`] which executes all deferred systems in the given [`Schedule`].
pub fn run_deferred_systems<S: ScheduleLabel>(world: &mut World) {
    let Some(mut systems) = world.get_resource_mut::<DeferredSystems<S>>() else {
        return;
    };

    systems.take().run(world);
}

#[test]
fn test_deferred_system() {
    use bevy::prelude::*;
    use bevy::MinimalPlugins;

    #[derive(Resource)]
    struct Success;

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, DefaultDeferredSystemsPlugin));

    app.world_mut()
        .run_deferred_system(Update, |mut commands: Commands| {
            commands.insert_resource(Success)
        });

    app.world_mut().flush(); // Should be redundant, but just to be sure ...
    assert!(!app.world().contains_resource::<Success>());

    app.update();
    assert!(app.world_mut().remove_resource::<Success>().is_some());

    // Ensure the system does not run again
    app.update();
    assert!(!app.world().contains_resource::<Success>());
}
