# 🛠️ Moonshine Utilities

[![crates.io](https://img.shields.io/crates/v/moonshine-util)](https://crates.io/crates/moonshine-util)
[![downloads](https://img.shields.io/crates/dr/moonshine-util?label=downloads)](https://crates.io/crates/moonshine-util)
[![docs.rs](https://docs.rs/moonshine-util/badge.svg)](https://docs.rs/moonshine-util)
[![license](https://img.shields.io/crates/l/moonshine-util)](https://github.com/Zeenobit/moonshine_util/blob/main/LICENSE)
[![stars](https://img.shields.io/github/stars/Zeenobit/moonshine_util)](https://github.com/Zeenobit/moonshine_util)

Collection of utilities for [Bevy](https://github.com/bevyengine/bevy).

## Features

### [`Expect<T>`]

A decorator for [`QueryData`](https://docs.rs/bevy/latest/bevy/ecs/query/trait.QueryData.html) which panics if it doesn't match.

This helps avoid silent failures in systems due to missing components:

```rust
use bevy::prelude::*;
use moonshine_util::prelude::*;

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

Normally, expected components would just be added as required components. However, in some cases it may not be possible
to add the required components, such as when dealing with third party crates or generic code.

Note that [`Expect<T>`] may also be used as a required component:

```rust
use bevy::prelude::*;
use moonshine_util::prelude::*;

#[derive(Component)]
struct A;

#[derive(Component)]
#[require(Expect<A>)] // Expect `A` to be present
struct B;
```

In this context, [`Expect<T>`] will panic if `B` is ever inserted into an entity without `A`.

### [`Get<T>`] and [`FromQuery`]

An ergonomic and generic way to process repetitive query patterns:

```rust
use bevy::prelude::*;
use moonshine_util::prelude::*;

struct Height(f32);

impl MapQuery for Height {
    type Query = &'static GlobalTransform;
    type Output = Self;

    fn map(data: &GlobalTransform) -> Self {
        Self(data.translation().y)
    }
}

fn average_height(query: Query<Get<Height>>) -> Height {
    // Transforms are so yesterday!
    let mut total_height = 0.0;
    let mut count = 0;
    for Height(h) in query.iter() {
        total_height += h;
        count += 1;
    }

    Height(total_height / count as f32)
}
```

### [`HierarchyQuery`]

A convenient [`SystemParam`](https://docs.rs/bevy/latest/bevy/ecs/system/trait.SystemParam.html) for traversing and querying entity hierarchies:

```rust
use bevy::prelude::*;
use moonshine_util::hierarchy::HierarchyQuery;

#[derive(Component)]
struct Needle;

#[derive(Component)]
struct Haystack;

fn spawn_haystack(mut commands: Commands) {
    // A complex hierarchy ...
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
    let haystack = haystack.single().unwrap();
    for needle in hierarchy.descendants_deep(haystack) {
        // ...
    }
}
```

Some useful functions include:

- `fn parent(&self, Entity) -> Option<Entity>`
- `fn children(&self, Entity) -> Iterator<Item = Entity>`
- `fn ancestors(&self, Entity) -> Iterator<Item = Entity>`
- `fn descendants_wide(&self, Entity) -> Iterator<Item = Entity>`
- `fn descendants_deep(&self, Entity) -> Iterator<Item = Entity>`

See [documentation][`HierarchyQuery`] for details.

For even more convenient hierarchy traversal, check out [🌴 Moonshine Object](https://github.com/Zeenobit/moonshine_object).

### [`RunSystemLoop`]

A trait similar to [`RunSystemOnce`](https://docs.rs/bevy/latest/bevy/ecs/system/trait.RunSystemOnce.html) which allows you to run a system loop for testing purposes:

```rust
use bevy::prelude::*;
use moonshine_util::diagnostics::RunSystemLoop;

let mut world = World::new();
let outputs = world.run_system_loop(2, |mut commands: Commands| {
    commands.spawn_empty().id()
});

assert_eq!(outputs.len(), 2);

assert!(world.get_entity(outputs[0]).is_ok());
assert!(world.get_entity(outputs[1]).is_ok());
```

### [`SingleEvent`]

A trait designed to behave like standard Bevy events. Unlike standard events, a [`SingleEvent`] may only be handled by a single observer.
This allows the single observer to consume and mutate the event data as needed.

```rust
use bevy::prelude::*;
use moonshine_util::prelude::*;

struct BigEvent(/* ... */);

impl SingleEvent for BigEvent {}

fn big_event_plugin(app: &mut App) {
    // Panics if there is another single observer registered for `BigEvent`:
    app.add_single_observer(on_big_event);
}

fn trigger_big_event(mut commands: Commands) {
    commands.trigger_single(BigEvent(/* ... */));
}

fn on_big_event(trigger: SingleTrigger<BigEvent>) {
    let event: BigEvent = trigger.consume().unwrap();
    /* ... */
}

```

### [Utility Systems](https://docs.rs/moonshine-util/latest/moonshine_util/system/index.html)

A growing collection of simple and generic systems useful for constructing larger system pipelines:

- `has_event<T: Event>`
- `has_resource<T: Resource>`
- `remove_resource<T: Resource>`
- `remove_resource_immediate<T: Resource>`
- `remove_all_components<T: Component>`

See [documentation](https://docs.rs/moonshine-util/latest/moonshine_util/system/index.html) for details and usage examples.

This crate is also included as part of [🍸 Moonshine Core](https://github.com/Zeenobit/moonshine_core).

## Changes

### Version 0.3.2

- Added [`SingleEvent`] feature

## Support

Please [post an issue](https://github.com/Zeenobit/moonshine_util/issues/new) for any bugs, questions, or suggestions.

You may also contact me on the official [Bevy Discord](https://discord.gg/bevy) server as **@Zeenobit**.

[`Expect<T>`]:https://docs.rs/moonshine-util/latest/moonshine_util/expect/struct.Expect.html
[`Get<T>`]:https://docs.rs/moonshine-util/latest/moonshine_util/query/struct.Get.html
[`FromQuery`]:https://docs.rs/moonshine-util/latest/moonshine_util/query/trait.FromQuery.html
[`HierarchyQuery`]:https://docs.rs/moonshine-util/latest/moonshine_util/hierarchy/struct.HierarchyQuery.html
[`RunSystemLoop`]:https://docs.rs/moonshine-util/latest/moonshine_util/diagnostics/trait.RunSystemLoop.html
[`SingleEvent`]:https://docs.rs/moonshine-util/latest/moonshine_util/event/struct.SingleEvent.html