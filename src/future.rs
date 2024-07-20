use std::sync::{Arc, Mutex};

#[must_use]
pub struct Promise<T>(Mutex<Option<Arc<Mutex<FutureValue<T>>>>>);

impl<T> Promise<T> {
    pub fn new() -> Self {
        Self(Mutex::new(Some(Arc::new(Mutex::new(FutureValue::Wait)))))
    }

    pub fn take(&self) -> Promise<T> {
        let arc = self.0.lock().unwrap().take().expect("promise is expired");
        Self(Mutex::new(Some(arc)))
    }

    pub fn done(self, value: T) {
        let arc = self.0.lock().unwrap().take().expect("promise is expired");
        *arc.lock().unwrap() = FutureValue::Ready(value);
    }

    pub fn is_expired(&self) -> bool {
        self.0.lock().unwrap().is_none()
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
        Self(
            promise
                .0
                .lock()
                .unwrap()
                .clone()
                .expect("promise is expired"),
        )
    }

    pub fn poll(&self) -> Option<T> {
        let mut value = self.0.lock().unwrap();

        if matches!(*value, FutureValue::Expired) {
            panic!("future is expired");
        }

        if matches!(*value, FutureValue::Wait) {
            return None;
        }

        let value = std::mem::replace(&mut *value, FutureValue::Expired);
        if let FutureValue::Ready(value) = value {
            Some(value)
        } else {
            unreachable!()
        }
    }

    pub fn is_expired(&self) -> bool {
        matches!(*self.0.lock().unwrap(), FutureValue::Expired)
    }
}

impl<T> Clone for Future<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

enum FutureValue<T> {
    Wait,
    Expired,
    Ready(T),
}

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

        assert!(w.run_system_once(move |q: Query<&Client>| { q.single().0.poll().is_none() }));
        assert!(w.run_system_once(move |q: Query<&Client>| { q.single().0.poll().is_none() }));

        w.run_system_once(move |q: Query<&Server>| q.single().0.take().done(()));

        assert!(w.run_system_once(move |q: Query<&Server>| { q.single().0.is_expired() }));
        assert!(w.run_system_once(move |q: Query<&Client>| { q.single().0.poll().is_some() }));
        assert!(w.run_system_once(move |q: Query<&Client>| { q.single().0.is_expired() }));
    }
}
