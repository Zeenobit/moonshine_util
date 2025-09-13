use std::marker::PhantomData;
use std::ops::AddAssign;

use bevy_ecs::component::{HookContext, Mutable};
use bevy_ecs::prelude::*;
use bevy_ecs::world::DeferredWorld;

use crate::Static;

pub trait AddComponent: Component<Mutability = Mutable> {
    fn add(&mut self, other: Self);
}

impl<T: AddAssign + Component<Mutability = Mutable>> AddComponent for T {
    fn add(&mut self, other: Self) {
        *self += other;
    }
}

#[derive(Component)]
#[component(on_insert = Self::on_insert)]
pub struct Add<T: AddComponent>(pub T);

impl<T: AddComponent> Add<T> {
    pub fn with<F: Static + FnOnce() -> R, R: Into<T>>(
        f: F,
    ) -> AddWith<T, impl Static + FnOnce() -> T> {
        AddWith::new(|| f().into())
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

#[derive(Component)]
#[component(on_insert = Self::on_insert)]
pub struct AddWith<T: AddComponent, F: Static + FnOnce() -> T>(F, PhantomData<T>);

impl<F: Static + FnOnce() -> T, T: AddComponent> AddWith<T, F> {
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
