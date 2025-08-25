use std::marker::PhantomData;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_ecs::schedule::ScheduleLabel;
use bevy_ecs::system::SystemId;
use bevy_log::prelude::*;

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

pub trait RunDeferredSystem {
    fn run_deferred_system_with<S: ScheduleLabel, I, M>(
        &mut self,
        schedule: S,
        system: impl 'static + IntoSystem<In<I>, (), M>,
        input: I,
    ) where
        I: 'static + Send + Sync;
}

impl RunDeferredSystem for World {
    fn run_deferred_system_with<S: ScheduleLabel, I, M>(
        &mut self,
        _schedule: S,
        system: impl 'static + IntoSystem<In<I>, (), M>,
        input: I,
    ) where
        I: 'static + Send + Sync,
    {
        let system = self.register_system_cached(system);
        self.get_resource_or_init::<DeferredSystems<S>>()
            .0
            .push(Box::new(DeferredSystem(system, input)));
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

trait AnyDeferredSystem: 'static + Send + Sync {
    fn run(self: Box<Self>, world: &mut World);
}

struct DeferredSystem<I: 'static + Send + Sync>(SystemId<In<I>>, I);

impl<I: 'static + Send + Sync> AnyDeferredSystem for DeferredSystem<I> {
    fn run(self: Box<Self>, world: &mut World) {
        if let Err(why) = world.run_system_with(self.0, self.1) {
            error!("deferred system error: {why}");
        }
    }
}

pub fn run_deferred_systems<S: ScheduleLabel>(world: &mut World) {
    let Some(mut systems) = world.get_resource_mut::<DeferredSystems<S>>() else {
        return;
    };

    systems.take().run(world);
}
