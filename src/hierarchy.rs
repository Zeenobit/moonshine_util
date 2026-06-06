//! Utilities related to relationship hierarchy traversal.

use std::collections::VecDeque;
use std::marker::PhantomData;

use bevy_ecs::prelude::*;
use bevy_ecs::relationship::{Relationship, SourceIter};
use bevy_ecs::system::SystemParam;

/// A [`SystemParam`] for ergonomic [`Entity`] hierarchy traversal.
#[derive(SystemParam)]
pub struct HierarchyQuery<'w, 's, R: Relationship = ChildOf> {
    parent: Query<'w, 's, &'static R>,
    children: Query<'w, 's, &'static <R as Relationship>::RelationshipTarget>,
}

impl<R: Relationship> HierarchyQuery<'_, '_, R> {
    /// Returns the parent of the given entity, if it has one.
    ///
    /// See [`ChildOf`] for more information.
    pub fn parent(&self, entity: Entity) -> Option<Entity> {
        self.parent.get(entity).ok().map(|parent| parent.get())
    }

    /// Iterates over the children of the given entity.
    ///
    /// See [`Children`] for more information.
    pub fn children(&self, entity: Entity) -> impl Iterator<Item = Entity> + '_ {
        self.children
            .get(entity)
            .ok()
            .into_iter()
            .flat_map(|children| children.iter())
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
    pub fn descendants_deep<'a>(&'a self, entity: Entity) -> impl Iterator<Item = Entity> + 'a
    where
        SourceIter<'a, R::RelationshipTarget>: DoubleEndedIterator,
    {
        self.children.iter_descendants_depth_first(entity)
    }
}

/// Iterator for breadth-first traversal of descendants.
pub struct WorldDescendantsWideIter<'w, R: Relationship = ChildOf> {
    world: &'w World,
    queue: VecDeque<Entity>,
    _marker: PhantomData<R>,
}

impl<'w, R: Relationship> WorldDescendantsWideIter<'w, R> {
    /// Creates a new [`WorldDescendantsWideIter`] to iterate over all descendants of the given
    /// [`Entity`] in breadth-first order.
    pub fn new(world: &'w World, root: Entity) -> Self {
        let mut queue = VecDeque::new();

        if let Some(children) = world.get::<R::RelationshipTarget>(root) {
            queue.extend(children.iter());
        }

        Self {
            world,
            queue,
            _marker: PhantomData,
        }
    }
}

impl<R: Relationship> Iterator for WorldDescendantsWideIter<'_, R> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.queue.pop_front()?;

        if let Some(children) = self.world.get::<R::RelationshipTarget>(current) {
            self.queue.extend(children.iter());
        }

        Some(current)
    }
}

/// Iterator for depth-first traversal of descendants.
pub struct WorldDescendantsDeepIter<'w, R: Relationship = ChildOf> {
    world: &'w World,
    stack: Vec<Entity>,
    _marker: PhantomData<R>,
}

impl<'w, R: Relationship> WorldDescendantsDeepIter<'w, R> {
    /// Creates a new [`WorldDescendantsDeepIter`] to iterate over all descendants of the given
    /// [`Entity`] in depth-first order.
    pub fn new(world: &'w World, root: Entity) -> Self {
        let mut stack = Vec::new();

        if let Some(children) = world.get::<R::RelationshipTarget>(root) {
            let children: Vec<Entity> = children.iter().collect();
            stack.extend(children.into_iter().rev());
        }

        Self {
            world,
            stack,
            _marker: PhantomData,
        }
    }
}

impl<R: Relationship> Iterator for WorldDescendantsDeepIter<'_, R> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.stack.pop()?;

        if let Some(children) = self.world.get::<R::RelationshipTarget>(current) {
            let children: Vec<Entity> = children.iter().collect();
            self.stack.extend(children.into_iter().rev());
        }

        Some(current)
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
    fn world_descendants_wide() {
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

        let r: Vec<_> = WorldDescendantsWideIter::<ChildOf>::new(&w, entity)
            .filter_map(|entity| w.get::<A>(entity))
            .map(|&A(v)| v)
            .collect();

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

    #[test]
    fn world_descendants_deep() {
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

        let r: Vec<_> = WorldDescendantsDeepIter::<ChildOf>::new(&w, entity)
            .filter_map(|entity| w.get::<A>(entity))
            .map(|&A(v)| v)
            .collect();

        assert_eq!(r, vec![1, 2, 3, 4, 5, 6, 7]);
    }
}
