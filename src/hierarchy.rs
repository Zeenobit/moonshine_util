//! Utilities related to relationship hierarchy traversal.

use bevy_ecs::component::HookContext;
use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemParam;
use bevy_ecs::world::DeferredWorld;

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

/// A [`SystemParam`] for ergonomic [`Entity`] hierarchy traversal.
#[derive(SystemParam)]
pub struct HierarchyQuery<'w, 's> {
    parent: Query<'w, 's, &'static ChildOf>,
    children: Query<'w, 's, &'static Children>,
}

// TODO: Support for generic relationship hierarchies

impl HierarchyQuery<'_, '_> {
    /// Returns the parent of the given entity, if it has one.
    ///
    /// See [`ChildOf`] for more information.
    pub fn parent(&self, entity: Entity) -> Option<Entity> {
        self.parent.get(entity).ok().map(|parent| parent.0)
    }

    /// Iterates over the children of the given entity.
    ///
    /// See [`Children`] for more information.
    pub fn children(&self, entity: Entity) -> impl Iterator<Item = Entity> + '_ {
        self.children
            .get(entity)
            .ok()
            .into_iter()
            .flat_map(|children| children.into_iter().copied())
    }

    /// Iterates over the ancestors of the given `entity`.
    ///
    /// See [`Query::iter_ancestors`] for more information.
    pub fn ancestors(&self, entity: Entity) -> impl Iterator<Item = Entity> + '_ {
        self.parent.iter_ancestors(entity)
    }

    /// Iterates over the descendants of the given `entity` in a breadth-first order.
    ///
    /// See [`Query::iter_descendants`] for more information.
    pub fn descendants_wide(&self, entity: Entity) -> impl Iterator<Item = Entity> + '_ {
        self.children.iter_descendants(entity)
    }

    /// Iterates over the descendants of the given `entity` in depth-first order.
    ///
    /// See [`Query::iter_descendants_depth_first`] for more information.
    pub fn descendants_deep(&self, entity: Entity) -> impl Iterator<Item = Entity> + '_ {
        self.children.iter_descendants_depth_first(entity)
    }
}

#[cfg(test)]
mod tests {
    use bevy_ecs::system::RunSystemOnce;

    use super::*;

    #[test]
    fn parent_child() {
        let mut w = World::new();
        let e = w
            .spawn_empty()
            .with_children(|parent| {
                parent.spawn_empty();
            })
            .id();

        let pass = w.run_system_once(move |q: HierarchyQuery| {
            assert_eq!(q.children(e).count(), 1);
            true
        });

        assert!(pass.unwrap());
    }

    #[test]
    fn ancestors() {
        #[derive(Component)]
        struct A(usize);

        #[derive(Component)]
        struct X;

        let mut w = World::new();
        w.spawn(A(0)).with_children(|a| {
            a.spawn(A(1)).with_children(|b| {
                b.spawn(A(2)).with_children(|c| {
                    c.spawn(A(3));
                    c.spawn(A(4)).with_children(|d| {
                        d.spawn(A(5));
                    });
                });
                b.spawn(A(6)).with_children(|c| {
                    c.spawn((A(7), X));
                });
            });
        });

        let r = w
            .run_system_once(
                move |q: HierarchyQuery, qa: Query<&A>, x: Single<Entity, With<X>>| {
                    let mut r = Vec::new();
                    for e in q.ancestors(*x) {
                        r.push(qa.get(e).unwrap().0);
                    }
                    r
                },
            )
            .unwrap();

        assert_eq!(r, vec![6, 1, 0]);
    }

    #[test]
    fn descendants_wide() {
        #[derive(Component)]
        struct A(usize);

        let mut w = World::new();
        let entity = w
            .spawn(A(0))
            .with_children(|a| {
                a.spawn(A(1)).with_children(|b| {
                    b.spawn(A(2)).with_children(|c| {
                        c.spawn(A(3));
                        c.spawn(A(4)).with_children(|d| {
                            d.spawn(A(5));
                        });
                    });
                    b.spawn(A(6)).with_children(|c| {
                        c.spawn(A(7));
                    });
                });
            })
            .id();

        let r = w
            .run_system_once(move |q: HierarchyQuery, qa: Query<&A>| {
                let mut r = Vec::new();
                for e in q.descendants_wide(entity) {
                    r.push(qa.get(e).unwrap().0);
                }
                r
            })
            .unwrap();

        assert_eq!(r, vec![1, 2, 6, 3, 4, 7, 5]);
    }

    #[test]
    fn descendants_deep() {
        #[derive(Component)]
        struct A(usize);

        let mut w = World::new();
        let entity = w
            .spawn(A(0))
            .with_children(|a| {
                a.spawn(A(1)).with_children(|b| {
                    b.spawn(A(2)).with_children(|c| {
                        c.spawn(A(3));
                        c.spawn(A(4)).with_children(|d| {
                            d.spawn(A(5));
                        });
                    });
                    b.spawn(A(6)).with_children(|c| {
                        c.spawn(A(7));
                    });
                });
            })
            .id();

        let r = w
            .run_system_once(move |q: HierarchyQuery, qa: Query<&A>| {
                let mut r = Vec::new();
                for e in q.descendants_deep(entity) {
                    r.push(qa.get(e).unwrap().0);
                }
                r
            })
            .unwrap();

        assert_eq!(r, vec![1, 2, 3, 4, 5, 6, 7]);
    }
}
