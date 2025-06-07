use bevy_ecs::component::HookContext;
use bevy_ecs::prelude::*;
use bevy_ecs::world::DeferredWorld;

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
