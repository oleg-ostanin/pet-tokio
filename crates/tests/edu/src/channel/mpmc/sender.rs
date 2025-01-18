use crate::channel::mpmc::channel::Shared;
use super::error::{Result, Error};

pub(crate) struct Sender<T> {
    shared: Shared<T>,
}

impl<T> Sender<T> {
    pub(crate) fn new(shared: Shared<T>) -> Self {
        Self { shared }
    }

    pub fn send(&self, item: T) -> Result<()> {
        if let Ok(mut items) = self.shared.data.lock() {
            let res = Ok(items.push(item));
            drop(items);
            self.shared.condvar.notify_all();
            res
        } else {
            Err(Error::FailedToSend)
        }
    }
}