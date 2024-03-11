# 🛠️ Moonshine Utilities

A collection of utilities for [Bevy](https://github.com/bevyengine/bevy) game engine.

## Features

### `Expect<T>`

A [`QueryData`](https://docs.rs/bevy/latest/bevy/ecs/query/trait.QueryData.html) "decorator" which panics if it doesn't match.

This helps avoid silent failures in systems due to missing components:

```rust
use bevy::prelude::*;
use moonshine_util::expect::Expect;

#[derive(Component)]
struct A;

#[derive(Component)]
struct B;

#[derive(Bundle)]
struct AB {
    a: A, // Every `A` is expected to have a `B`
    b: B,
}

fn bad_system(mut commands: Commands) {
    commands.spawn(A); // BUG: Spawn A witout B!
}

fn unsafe_system(mut query: Query<(&A, &B)>) {
    for (a, b) in query.iter() {
        // An instance of `A` does exist, but this system skips over it silently!
    }
}

fn safe_system(mut query: Query<(&A, Expect<&B>)>) {
    for (a, b) in query.iter() {
        // This system will panic if an `A` instance is missing a `B`!
    }
}
```

### `HierarchyQuery`

A convenient [`SystemParam`](https://docs.rs/bevy/latest/bevy/ecs/system/trait.SystemParam.html) for traversing and querying entity hierarchies:

```rust
use bevy::prelude::*;
use moonshine_util::hierarchy::HierarchyQuery;

#[derive(Component)]
struct Needle;

#[derive(Component)]
struct Haystack;

fn spawn_haystack(mut commands: Commands) {
    // A very complex hierarchy ...
    commands.spawn(Haystack).with_children(|x| {
        x.spawn_empty().with_children(|y| {
            y.spawn_empty().with_children(|z| {
                z.spawn(Needle);
            });
        });
    });
}

fn find_needle(
    haystack: Query<Entity, With<Haystack>>,
    needle_query: Query<Entity, With<Needle>>,
    hierarchy: HierarchyQuery
) {
    let haystack = haystack.single();
    let needle = hierarchy.find_descendant(haystack, &needle_query);
}
```

Some useful functions include:

- `fn parent(&self, entity: Entity) -> Option<Entity>`
- `fn has_parent(&self, entity: Entity) -> bool`
- `fn children(&self, entity: Entity) -> impl Iterator<Item = Entity>`
- `fn has_children(&self, entity: Entity) -> bool`
- `fn root(&self, entity: Entity) -> Entity`
- `fn is_root(&self, entity: Entity) -> bool`
- `fn ancestors(&self, entity: Entity) -> impl Iterator<Item = Entity>`
- `fn descendants(&self, entity: Entity) -> impl Iterator<Item = Entity>`
- `fn is_ancestor_of(&self, entity: Entity, descendant: Entity) -> bool`
- `fn is_descendant_of(&self, entity: Entity, ancestor: Entity) -> bool`
- `fn find_ancestor<T, F>(&self, entity: Entity, query: &Query<T, F>) -> Option<QueryItem<T>`
- `fn find_descendant<T, F>(&self, entity: Entity, query: &Query<T, F>) -> Option<QueryItem<T>`

### `RunSystemLoop`

A trait similar to [`RunSystemOnce`](https://docs.rs/bevy/latest/bevy/ecs/system/trait.RunSystemOnce.html) which allows you to run a system loop for testing purposes:

```rust
use bevy::prelude::*;
use moonshine_util::diagnostics::RunSystemLoop;

let mut world = World::new();
let outputs = world.run_system_loop(2, |mut commands: Commands| {
    commands.spawn_empty().id()
});

assert_eq!(outputs.len(), 2);

assert!(world.get_entity(outputs[0]).is_some());
assert!(world.get_entity(outputs[1]).is_some());
```