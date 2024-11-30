use bevy_ecs::{prelude::*, system::SystemParam};
use bevy_hierarchy::prelude::*;

/// A [`SystemParam`] for ergonomic [`Entity`] hierarchy traversal.
#[derive(SystemParam)]
pub struct HierarchyQuery<'w, 's> {
    parent: Query<'w, 's, &'static Parent>,
    children: Query<'w, 's, &'static Children>,
}

impl HierarchyQuery<'_, '_> {
    /// Returns the parent of the given entity, if it has one.
    pub fn parent(&self, entity: Entity) -> Option<Entity> {
        self.parent.get(entity).ok().map(|parent| **parent)
    }

    /// Iterates over the children of the given entity.
    pub fn children(&self, entity: Entity) -> impl Iterator<Item = Entity> + '_ {
        self.children
            .get(entity)
            .ok()
            .into_iter()
            .flat_map(|children| children.into_iter().copied())
    }

    /// Iterates over the ancestors of the given `entity`.
    pub fn ancestors(&self, entity: Entity) -> impl Iterator<Item = Entity> + '_ {
        self.parent.iter_ancestors(entity)
    }

    /// Iterates over the descendants of the given `entity`.
    pub fn descendants_wide(&self, entity: Entity) -> impl Iterator<Item = Entity> + '_ {
        self.children.iter_descendants(entity)
    }

    pub fn descendants_deep(&self, entity: Entity) -> impl Iterator<Item = Entity> + '_ {
        DeepDescendantIter::new(&self.children, entity)
    }
}

/// An [`Iterator`] of [`Entity`]s over the descendants of an [`Entity`].
///
/// Traverses the hierarchy breadth-first.
struct DeepDescendantIter<'w, 's, 'a> {
    query: &'a Query<'w, 's, &'static Children>,
    stack: Vec<Entity>,
}

impl<'w, 's, 'a> DeepDescendantIter<'w, 's, 'a> {
    fn new(query: &'a Query<'w, 's, &'static Children>, entity: Entity) -> Self {
        DeepDescendantIter {
            query,
            stack: query
                .get(entity)
                .into_iter()
                .flatten()
                .copied()
                .rev()
                .collect(),
        }
    }
}

impl Iterator for DeepDescendantIter<'_, '_, '_> {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.stack.pop()?;
        let children = self.query.get(current).into_iter().flatten().copied().rev();
        self.stack.extend(children);
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
                move |q: HierarchyQuery, qa: Query<&A>, qx: Query<Entity, With<X>>| {
                    let entity = qx.single();
                    let mut r = Vec::new();
                    for e in q.ancestors(entity) {
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
