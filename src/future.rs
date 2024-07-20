use std::sync::{Arc, Mutex};

#[must_use]
pub struct Promise<T>(Arc<Mutex<Option<T>>>);

impl<T> Promise<T> {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }

    pub fn take(&self) -> Promise<T> {
        Self(Arc::new(Mutex::new(self.0.lock().unwrap().take())))
    }

    pub fn done(self, value: T) {
        *self.0.lock().unwrap() = Some(value);
    }
}

impl<T> Default for Promise<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[must_use]
pub struct Future<T>(Arc<Mutex<Option<T>>>);

impl<T> Future<T> {
    pub fn new(promise: &Promise<T>) -> Self {
        Self(Arc::clone(&promise.0))
    }

    pub fn poll(self) -> FutureResponse<T> {
        let value = self.0.lock().unwrap().take();
        match value {
            Some(value) => FutureResponse::Done(value),
            None => FutureResponse::Wait(self),
        }
    }
}

impl<T> Clone for Future<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

pub enum FutureResponse<T> {
    Wait(Future<T>),
    Done(T),
}

impl<T> FutureResponse<T> {
    pub fn is_done(&self) -> bool {
        matches!(self, Self::Done(_))
    }

    pub fn unwrap(self) -> T {
        match self {
            Self::Done(value) => value,
            Self::Wait(_) => panic!("future is not ready"),
        }
    }
}
