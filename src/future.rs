use std::sync::{Arc, Mutex};

/// A value that will be written to in the future.
///
/// # Usage
///
/// This type is designed to work with [`Future`]. It is a very simplified implementation of
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
/// This pattern can be used to synchronize data between systems:
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
///
/// # bevy_ecs::system::assert_is_system(add);
/// # bevy_ecs::system::assert_is_system(request_add);
/// # bevy_ecs::system::assert_is_system(process_add_result);
/// ```
#[deprecated(since = "0.2.6", note = "use sparse components instead")]
#[must_use]
pub struct Promise<T>(Arc<Mutex<FutureValue<T>>>);

impl<T> Promise<T> {
    /// Creates a new promise.
    ///
    /// See [`Promise::start()`] for a more convenient way to create a promise and its associated future.
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(Wait)))
    }

    /// Creates a new promise and its associated future.
    pub fn start() -> (Self, Future<T>) {
        let promise = Self::new();
        let future = Future::new(&promise);
        (promise, future)
    }

    /// Sets the promised future value.
    /// This will notify the associated [`Future`] that it is [`Ready`].
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
#[deprecated(since = "0.2.6", note = "use sparse components instead")]
#[must_use]
pub struct Future<T>(Arc<Mutex<FutureValue<T>>>);

impl<T> Future<T> {
    /// Creates a new future that is already expired.
    pub fn expired() -> Self {
        Self(Arc::new(Mutex::new(Expired)))
    }

    /// Creates a new future associated with a given [`Promise`].
    pub fn new(promise: &Promise<T>) -> Self {
        Self(promise.0.clone())
    }

    /// Polls the future for its value.
    ///
    /// # Usage
    ///
    /// This function may return one of three values:
    ///
    /// 1. If the promised value is ready, this will return [`Ready`] and the future will be expired.
    /// 2. If the promised value is not ready yet, this will return [`Wait`].
    /// 3. If this future is expired, this will return [`Expired`].
    ///    You should not handle this case explicitly, and instead avoid polling the future twice.
    pub fn poll(&self) -> FutureValue<T> {
        let Ok(mut value) = self.0.try_lock() else {
            return Wait;
        };
        if matches!(*value, Wait) {
            return Wait;
        }
        if matches!(*value, Expired) {
            return Expired;
        }
        std::mem::replace(&mut *value, Expired)
    }

    pub fn forget(self) {}
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
            w.run_system_once(move |q: Query<&Client>| { q.single().0.poll() })
                .unwrap(),
            Wait
        );
        assert_eq!(
            w.run_system_once(move |q: Query<&Client>| { q.single().0.poll() })
                .unwrap(),
            Wait
        );

        w.run_system_once(move |q: Query<(Entity, &Server)>, mut commands: Commands| {
            let (entity, server) = q.single();
            server.0.set(());
            commands.entity(entity).remove::<Server>();
        })
        .unwrap();

        assert_eq!(
            w.run_system_once(move |q: Query<&Client>| { q.single().0.poll() })
                .unwrap(),
            Ready(())
        );
        assert_eq!(
            w.run_system_once(move |q: Query<&Client>| { q.single().0.poll() })
                .unwrap(),
            Expired
        );
    }
}
