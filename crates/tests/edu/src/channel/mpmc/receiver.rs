use std::sync::Arc;
use crate::channel::mpmc::channel::Shared;
use super::error::{Result, Error};


pub(crate) struct Receiver<T> {
    shared: Shared<T>,
    last_received: Option<usize>,
}

impl<T: Clone> Receiver<T> {
    pub(crate) fn new(shared: Shared<T>) -> Self {
        Self {
            shared,
            last_received: None,
        }
    }

    pub fn receive(&mut self) -> Result<T> {
        if let Ok(mut items) = self.shared.data.lock() {
            loop {
                let index_to_get = match self.last_received {
                    None => { 0 }
                    Some(index) => { index + 1 }
                };
                if let Some(item) = items.get(index_to_get) {
                    self.last_received = Some(index_to_get);
                    return Ok(item.clone());
                } else {
                    items = self.shared.condvar.wait(items).expect("Should never be poisoned");
                }
            }
        } else {
            return Err(Error::FailedToReceive);
        }
    }
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        Receiver {
            shared: self.shared.clone(),
            last_received: None,
        }
    }
}