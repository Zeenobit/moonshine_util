//! Utilities related to spawning entities.

use bevy_ecs::component::HookContext;
use bevy_ecs::prelude::*;
use bevy_ecs::world::DeferredWorld;

/// This [`Component`] is similar to [`SpawnWith`](bevy_ecs::spawn::SpawnWith), but it spawns the
/// associated entity without any [`Relationship`](bevy_ecs::relationship::Relationship).
#[derive(Component)]
#[component(storage = "SparseSet")]
#[component(on_add = Self::on_add)]
pub struct SpawnUnrelated<B: Bundle, F: FnOnce(Entity) -> B>(pub F)
where
    F: 'static + Send + Sync,
    B: 'static + Send + Sync;

impl<B: Bundle, F: FnOnce(Entity) -> B> SpawnUnrelated<B, F>
where
    F: 'static + Send + Sync,
    B: 'static + Send + Sync,
{
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let entity = ctx.entity;
        world.commands().queue(move |world: &mut World| {
            let mut entity = world.entity_mut(entity);
            let SpawnUnrelated(f) = entity.take::<Self>().unwrap();
            let source = entity.id();
            world.spawn(f(source));
        });
    }
}

/// A [`Component`] which spawns a child [`Entity`] when inserted into some parent.
///
/// Unlike [`Children::spawn`] (and by extension [`children!`]), each instance of this component is unique.
/// This allows you to have multiple instances of it within the same bundle to spawn multiple
/// children independently of each other.
#[derive(Component)]
#[component(storage = "SparseSet")]
#[component(on_add = Self::on_add)]
pub struct WithChild<B: Bundle, F: FnOnce(Entity) -> B>(pub F)
where
    F: 'static + Send + Sync,
    B: 'static + Send + Sync;

impl<B: Bundle, F: FnOnce(Entity) -> B> WithChild<B, F>
where
    F: 'static + Send + Sync,
    B: 'static + Send + Sync,
{
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let entity = ctx.entity;
        world.commands().queue(move |world: &mut World| {
            let mut entity = world.entity_mut(entity);
            let WithChild(f) = entity.take::<Self>().unwrap();
            entity.with_child(f(entity.id()));
        });
    }
}
