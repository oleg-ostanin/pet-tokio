use std::marker::PhantomData;
use std::sync::{Arc, Condvar, Mutex, RwLock};
use crate::channel::mpmc::receiver::Receiver;
use crate::channel::mpmc::sender::Sender;
use crate::common::Types;

pub(crate) struct Shared<T> {
    pub(crate) data: Arc<Mutex<Vec<T>>>,
    pub(crate) condvar: Arc<Condvar>,
}

impl<T> Shared<T> {
    fn new() -> Shared<T> {
        Shared {
            data: Arc::new(Mutex::new(Vec::default())),
            condvar: Arc::new(Condvar::new()),
        }
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Shared {
            data: Arc::clone(&self.data),
            condvar: Arc::clone(&self.condvar),
        }
    }
}

pub(crate) struct Channel {}

impl Channel {
    pub fn new<T: Clone>() -> (Sender<T>, Receiver<T>) {
        let shared = Shared::new();
        let sender = Sender::new(shared.clone());
        let receiver = Receiver::new(shared);
        (sender, receiver)
    }
}
