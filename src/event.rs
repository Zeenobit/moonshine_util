//! Utilities related to Bevy [`Event`] system.

use bevy_app::{App, Plugin};
use bevy_ecs::prelude::*;
use bevy_ecs::system::IntoObserverSystem;
use std::marker::PhantomData;
use std::sync::Mutex;

use crate::Static;

/// An [`Event`]-like trait for events that may only trigger a single observer.
///
/// # Usage
///
/// Standard Bevy events are always read-only and accessible by by reference because
/// each event can trigger many observers.
///
/// However, sometimes you may need to consume the event data to avoid unnecessary cloning and
/// you know that you will only have a single observer for that event.
///
/// For these cases, you can use the [`SingleEvent`] trait which.
///
/// See also:
/// - [`OnSingle`]
/// - [`add_single_observer`](AddSingleObserver::add_single_observer)
/// - [`trigger_single`](TriggerSingle::trigger_single)
pub trait SingleEvent: Static {}

/// Trait used to register single observers for [`SingleEvent`]s.
pub trait AddSingleObserver {
    /// Checks if an observer is registered for a given [`SingleEvent`].
    fn has_single_observer<E: SingleEvent>(&self) -> bool;

    /// Adds a single observer for a given [`SingleEvent`] and guarantees that it's the only one registered.
    ///
    /// # Panic
    /// This will panic if an observer for the same event is already registered.
    fn add_single_observer<E: SingleEvent, B: Bundle, M>(
        self,
        observer: impl IntoSingleObserverSystem<E, B, M>,
    ) -> Self;
}

impl AddSingleObserver for &mut App {
    fn has_single_observer<E: SingleEvent>(&self) -> bool {
        self.is_plugin_added::<SingleEventObserverPlugin<E>>()
    }

    fn add_single_observer<E: SingleEvent, B: Bundle, M>(
        self,
        observer: impl IntoSingleObserverSystem<E, B, M>,
    ) -> Self {
        if !self.is_plugin_added::<SingleEventObserverPlugin<E>>() {
            self.add_plugins(SingleEventObserverPlugin::<E>::new());
        } else {
            panic!(
                "a single observer is already registered for event: {}",
                std::any::type_name::<E>()
            );
        }

        self.add_observer(observer)
    }
}

/// Trait used to trigger a [`SingleEvent`] via [`Commands`] or [`World`].
pub trait TriggerSingle {
    /// Triggers a [`SingleEvent`], similar to [`Commands::trigger`].
    ///
    // TODO: Support targets
    fn trigger_single<E: SingleEvent>(self, event: E);
}

impl TriggerSingle for &mut Commands<'_, '_> {
    fn trigger_single<E: SingleEvent>(self, event: E) {
        self.trigger(SingleEventWrapper::new(event));
    }
}

impl TriggerSingle for &mut World {
    fn trigger_single<E: SingleEvent>(self, event: E) {
        self.trigger(SingleEventWrapper::new(event));
    }
}

#[doc(hidden)]
pub trait IntoSingleObserverSystem<E: SingleEvent, B: Bundle, M>:
    IntoObserverSystem<SingleEventWrapper<E>, B, M>
{
}

impl<E: SingleEvent, B: Bundle, M, S> IntoSingleObserverSystem<E, B, M> for S where
    S: IntoObserverSystem<SingleEventWrapper<E>, B, M>
{
}

/// A standard [`Event`] which contains a [`SingleEvent`].
///
/// You should avoid using this directly and instead use [`OnSingle`] for better ergonomics.
#[derive(Event)]
pub struct SingleEventWrapper<E: SingleEvent>(Mutex<Option<E>>);

impl<E: SingleEvent> SingleEventWrapper<E> {
    fn new(event: E) -> Self {
        Self(Mutex::new(Some(event)))
    }

    /// Consumes the [`SingleEvent`] and returns it.
    ///
    /// Returns [`None`] if the event has already been consumed.
    pub fn consume(&self) -> Option<E> {
        self.0.lock().unwrap().take()
    }
}

/// Trigger for [`SingleEvent`] types.
///
/// Usage is identical to [`On`] but with the addition of the [`consume`](SingleEventWrapper::consume) method.
pub type OnSingle<'w, 't, E, B = ()> = On<'w, 't, SingleEventWrapper<E>, B>;

#[doc(hidden)]
pub struct SingleEventObserverPlugin<E: SingleEvent>(PhantomData<E>);

impl<E: SingleEvent> SingleEventObserverPlugin<E> {
    fn new() -> Self {
        Self(PhantomData)
    }
}

impl<E: SingleEvent> Plugin for SingleEventObserverPlugin<E> {
    fn build(&self, _: &mut App) {}
}
