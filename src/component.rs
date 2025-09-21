//! Utilities related to [`Component`] management.

use std::marker::PhantomData;
use std::ops::AddAssign;

use bevy_ecs::component::{HookContext, Mutable};
use bevy_ecs::prelude::*;
use bevy_ecs::world::DeferredWorld;

use crate::Static;

/// Any [`Component`] which can be merged with itself.
///
/// This trait is automatically implemented for any [`Component`] which also implements [`AddAssign`].
///
/// See [`Add<T>`] for detailed usage and examples.
pub trait AddComponent: Component<Mutability = Mutable> {
    /// Merges the contents of `other` into this [`Component`].
    fn add(&mut self, other: Self);
}

impl<T: AddAssign + Component<Mutability = Mutable>> AddComponent for T {
    fn add(&mut self, other: Self) {
        *self += other;
    }
}

/// An [`EntityCommand`] which is used to add components.
///
/// # Usage
/// It is impossible to have duplicate components on an [`Entity`] in Bevy.
/// However, in some cases, multiple instances of some components can be "merged" into one.
///
/// If a component implements [`AddComponent`], you can use this command to merge multiple instances
/// of the component into one.
///
/// ```rust
/// use bevy::prelude::*;
/// use moonshine_util::prelude::*;
///
/// #[derive(Component, Default)]
/// struct N(usize);
///
/// impl AddComponent for N {
///     fn add(&mut self, rhs: Self) {
///         self.0 += rhs.0;
///     }
/// }
///
/// let mut world = World::new();
/// let entity = world.spawn_empty().id();
/// world.commands().entity(entity).queue(Add(N(1)));
/// world.commands().entity(entity).queue(Add(N(2)));
/// world.flush();
/// let &N(value) = world.get(entity).unwrap();
/// assert_eq!(value, 3);
/// ```
///
/// This command may also be used as a [`Component`] itself. This can be used in a [`Bundle`] or as a
/// requirement to merge components.
///
/// ```rust
/// use bevy::prelude::*;
/// use moonshine_util::prelude::*;
///
/// #[derive(Component, Default)]
/// struct N(usize);
///
/// impl AddComponent for N {
///     fn add(&mut self, rhs: Self) {
///         self.0 += rhs.0;
///     }
/// }
///
/// let mut world = World::new();
/// let entity = world.spawn((N(1), Add(N(2))));
/// let &N(value) = entity.get().unwrap();
/// assert_eq!(value, 3);
/// ```
///
/// Because [`Add<T>`] is a component itself, it can be used as a component requirement.
/// However, because of the component uniqueness rule, multiple `Add<T>` instances may not exist on the same entity.
///
/// To work around this, you can use [`AddFrom`] and [`AddWith`].
#[derive(Component)]
#[component(on_insert = Self::on_insert)]
pub struct Add<T: AddComponent>(pub T);

impl<T: AddComponent> Add<T> {
    /// Ergonomic alias for [`AddWith::new`].
    pub fn with<F: Static + FnOnce() -> T>(f: F) -> AddWith<T, impl Static + FnOnce() -> T> {
        AddWith::new(f)
    }

    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        world
            .commands()
            .entity(ctx.entity)
            .queue(|mut entity: EntityWorldMut| {
                entity.take::<Self>().unwrap().apply(entity);
            });
    }
}

impl<T: AddComponent> From<T> for Add<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T: AddComponent> EntityCommand for Add<T> {
    fn apply(self, mut entity: EntityWorldMut) -> () {
        let Self(source) = self;
        if let Some(mut target) = entity.get_mut::<T>() {
            target.add(source);
        } else {
            entity.insert(source);
        }
    }
}

/// A [`Component`] which is used to add components as requirements.
///
/// # Usage
/// Because [`Add<T>`] is a component itself, it can be used as a component requirement.
/// This type may instead be used to work around this restriction:
///
/// ```rust
/// use bevy::prelude::*;
/// use moonshine_util::prelude::*;
///
/// #[derive(Component, Default)]
/// struct N(usize);
///
/// impl AddComponent for N {
///     fn add(&mut self, rhs: Self) {
///         self.0 += rhs.0;
///     }
/// }
///
/// #[derive(Component, Default)]
/// #[require(AddFrom<Self, N> = N(1))]
/// struct A;
///
/// #[derive(Component, Default)]
/// #[require(A, AddFrom<Self, N> = N(2))]
/// struct B;
///
/// let mut world = World::new();
/// let entity = world.spawn(B);
/// let &N(value) = entity.get().unwrap();
/// assert_eq!(value, 3);
/// ```
#[derive(Component)]
#[component(on_insert = Self::on_insert)]
pub struct AddFrom<M: Static, T: AddComponent>(Add<T>, PhantomData<M>);

impl<M: Static, T: AddComponent> AddFrom<M, T> {
    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        world
            .commands()
            .entity(ctx.entity)
            .queue(|mut entity: EntityWorldMut| {
                let Self(inner, ..) = entity.take::<Self>().unwrap();
                inner.apply(entity);
            });
    }
}

impl<M: Static, T: AddComponent> From<T> for AddFrom<M, T> {
    fn from(value: T) -> Self {
        Self(Add(value), PhantomData)
    }
}
/// A [`Component`] which is used to add components as requirements.
///
/// # Usage
/// Because [`Add<T>`] is a component itself, it can be used as a component requirement.
/// This type may instead be used to work around this restriction:
///
/// ```rust
/// use bevy::prelude::*;
/// use moonshine_util::prelude::*;
///
/// #[derive(Component, Default)]
/// struct N(usize);
///
/// impl AddComponent for N {
///     fn add(&mut self, rhs: Self) {
///         self.0 += rhs.0;
///     }
/// }
///
/// let mut world = World::new();
/// let entity = world.spawn((Add::with(|| N(1)), Add::with(|| N(2))));
/// let &N(value) = entity.get().unwrap();
/// assert_eq!(value, 3);
/// ```
#[derive(Component)]
#[component(on_insert = Self::on_insert)]
pub struct AddWith<T: AddComponent, F: Static + FnOnce() -> T>(F, PhantomData<T>);

impl<F: Static + FnOnce() -> T, T: AddComponent> AddWith<T, F> {
    /// Creates a new [`AddWith`] [`Component`] for the given [`FnOnce`].
    ///
    /// See [`Add::with`] for a more ergonomic constructor.
    pub fn new(f: F) -> Self {
        Self(f, PhantomData)
    }

    fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
        world
            .commands()
            .entity(ctx.entity)
            .queue(|mut entity: EntityWorldMut| {
                let Self(f, ..) = entity.take::<Self>().unwrap();
                Add(f()).apply(entity);
            });
    }
}

impl<F: Static + FnOnce() -> T, T: AddComponent> From<F> for AddWith<T, F> {
    fn from(f: F) -> Self {
        Self::new(f)
    }
}

/// A convenient macro for defining a pair of [`Relationship`] and [`RelationshipTarget`] component.
///
/// ```rust
/// use bevy::prelude::*;
/// use moonshine_util::prelude::*;
///
/// relationship! {
///     #[derive(Default)]
///     pub Friends(Vec<Entity>) -> { pub FriendOf(pub Entity) }
/// }
///
/// let mut w = World::new();
/// let a = w.spawn_empty().id();
/// let b = w.spawn(FriendOf(a)).id();
///
/// assert!(w
///     .get::<Friends>(a)
///     .is_some_and(|Friends(friends)| friends[0] == b));
/// ```
///
/// [`Relationship`]: bevy_ecs::relationship::Relationship
/// [`RelationshipTarget`]: bevy_ecs::relationship::RelationshipTarget
#[macro_export]
macro_rules! relationship {
    {
        $(#[$target_attr:meta])*
        $target_vis:vis $target:ident($target_inner_vis:vis $target_inner:ty)
        -> {
            $(#[$source_attr:meta])*
            $source_vis:vis $source:ident($source_inner_vis:vis $source_inner:ty)
        }
    } => {
        relationship! {
            $(#[$target_attr])* $target_vis $target($target_inner_vis $target_inner) -> [] {
                $(#[$source_attr])* $source_vis $source($source_inner_vis $source_inner)
            }
        }
    };

    {
        $(#[$target_attr:meta])*
        $target_vis:vis $target:ident($target_inner_vis:vis $target_inner:ty)
        -> [$($options:expr),*] {
            $(#[$source_attr:meta])*
            $source_vis:vis $source:ident($source_inner_vis:vis $source_inner:ty)
        }
    } => {
        $(#[$target_attr])*
        #[derive(Component)]
        #[relationship_target(relationship = $source, $($options),*)]
        $target_vis struct $target($target_inner_vis $target_inner);

        $(#[$source_attr])*
        #[derive(Component)]
        #[relationship(relationship_target = $target)]
        $source_vis struct $source($source_inner_vis $source_inner);
    };
}

#[test]
fn test_add_component() {
    #[derive(Component, Default)]
    struct N(usize);

    impl AddAssign for N {
        fn add_assign(&mut self, rhs: Self) {
            self.0 += rhs.0;
        }
    }

    #[derive(Component, Default)]
    #[require(AddFrom<Self, N> = N(1))]
    struct A;

    #[derive(Component, Default)]
    #[require(A, AddFrom<Self, N> = N(2))]
    struct B;

    #[derive(Component, Default)]
    #[require(A, B)]
    struct C;

    let mut w = World::new();
    let e = w.spawn(C);
    let &N(v) = e.get().unwrap();

    assert_eq!(v, 3);
}

#[test]
fn test_relationship_linked_spawn() {
    relationship! {
        pub Owner(Vec<Entity>) -> [linked_spawn] {
            pub OwnedBy(pub Entity)
        }
    }

    let mut w = World::new();
    let a = w.spawn_empty().id();
    let b = w.spawn(OwnedBy(a)).id();
    assert_eq!(w.get::<Owner>(a).unwrap().0[0], b);
    assert!(w.entities().contains(b));

    w.entity_mut(a).despawn();
    assert!(!w.entities().contains(b));
}
