use std::sync::{Arc, Mutex};

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

#[must_use]
pub struct Future<T>(Arc<Mutex<FutureValue<T>>>);

impl<T> Future<T> {
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
