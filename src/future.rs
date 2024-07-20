use std::sync::{Arc, Mutex};

/// A value that will be written to in the future.
///
/// # Usage
///
/// This type is designed to work with the [`Future`] type. It is a simplified implementation of
/// the [`std::future::Future`] pattern specifically design for use in Bevy ECS.
///
/// # Example
///
/// ```
/// use moonshine_util::future::*;
///
/// let promise = Promise::new();
/// let future = Future::new(&promise);
///
/// assert_eq!(future.poll(), Wait);
///
/// promise.set(42);
///
/// assert_eq!(future.poll(), Ready(42));
/// assert_eq!(future.poll(), Expired);
/// ```
///
/// This simple pattern can be used to synchronize data between systems:
///
/// ```
/// use bevy::prelude::*;
/// use moonshine_util::future::*;
///
/// #[derive(Component)]
/// struct Add(u32, u32, Promise<u32>);
///
/// fn add(query: Query<(Entity, &Add)>, mut commands: Commands) {
///     for (entity, Add(a, b, promise)) in query.iter() {
///         promise.set(a + b);
///
///         // Typically, this would be done to ensure we don't set the promise twice:
///         commands.entity(entity).remove::<Add>();
///     }
/// }
///
/// #[derive(Component)]
/// struct AddResult(Future<u32>);
///
/// fn request_add(mut commands: Commands) {
///     let promise = Promise::new();
///     let future = Future::new(&promise);
///     commands.spawn(Add(2, 2, promise));
///     commands.spawn(AddResult(future));
/// }
///
/// fn process_add_result(query: Query<(Entity, &AddResult)>, mut commands: Commands) {
///     for (entity, AddResult(future)) in query.iter() {
///         match future.poll() {
///             Ready(result) => {
///                 println!("2 + 2 = {}", result);
///
///                 // Ensure future is not polled again:
///                 commands.entity(entity).remove::<AddResult>();
///             },
///             Wait => {
///                 continue;
///             },
///             _ => {
///                 // If the future is used correctly, this case should never happen.
///                 // Do *NOT* poll the future again after it returns ready.
///                 unreachable!();
///             },
///         }
///     }
/// }
/// ```
#[must_use]
pub struct Promise<T>(Arc<Mutex<FutureValue<T>>>);

impl<T> Promise<T> {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(Wait)))
    }

    pub fn set(&self, value: T) {
        let mut future = self.0.lock().unwrap();
        *future = Ready(value);
    }
}

impl<T> Default for Promise<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// A value that will be read from in the future, when ready.
///
/// See [`Promise`] for more usage information and examples.
#[must_use]
pub struct Future<T>(Arc<Mutex<FutureValue<T>>>);

impl<T> Future<T> {
    pub fn expired() -> Self {
        Self(Arc::new(Mutex::new(Expired)))
    }

    pub fn new(promise: &Promise<T>) -> Self {
        Self(promise.0.clone())
    }

    pub fn poll(&self) -> FutureValue<T> {
        let mut value = self.0.lock().unwrap();
        if matches!(*value, Wait) {
            return Wait;
        }
        if matches!(*value, Expired) {
            return Expired;
        }
        std::mem::replace(&mut *value, Expired)
    }
}

impl<T> Default for Future<T> {
    fn default() -> Self {
        Self::expired()
    }
}

#[must_use]
#[derive(Debug, PartialEq)]
pub enum FutureValue<T> {
    Wait,
    Ready(T),
    Expired,
}

impl<T> FutureValue<T> {
    pub fn is_ready(&self) -> bool {
        matches!(self, Ready(_))
    }

    pub fn is_expired(&self) -> bool {
        matches!(self, Expired)
    }

    pub fn unwrap(self) -> T {
        if let Ready(value) = self {
            value
        } else {
            panic!("future is not ready")
        }
    }
}

pub use FutureValue::{Expired, Ready, Wait};

#[cfg(test)]
mod test {
    use super::*;

    use bevy_ecs::{prelude::*, system::RunSystemOnce};

    #[test]
    fn future_value() {
        #[derive(Component)]
        struct Server(Promise<()>);

        #[derive(Component)]
        struct Client(Future<()>);

        let mut w = World::new();

        let p = Promise::new();
        let f = Future::new(&p);
        w.spawn(Server(p));
        w.spawn(Client(f));

        assert_eq!(
            w.run_system_once(move |q: Query<&Client>| { q.single().0.poll() }),
            Wait
        );
        assert_eq!(
            w.run_system_once(move |q: Query<&Client>| { q.single().0.poll() }),
            Wait
        );

        w.run_system_once(move |q: Query<(Entity, &Server)>, mut commands: Commands| {
            let (entity, server) = q.single();
            server.0.set(());
            commands.entity(entity).remove::<Server>();
        });

        assert_eq!(
            w.run_system_once(move |q: Query<&Client>| { q.single().0.poll() }),
            Ready(())
        );
        assert_eq!(
            w.run_system_once(move |q: Query<&Client>| { q.single().0.poll() }),
            Expired
        );
    }
}
