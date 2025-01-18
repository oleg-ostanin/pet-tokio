use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::common::error::{Result, Error};


pub(crate) struct Storage {
    items: Arc<RwLock<HashMap<u32, u32>>>,
}

impl Storage {
    pub fn new() -> Self {
        Self { items: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub fn items(&self) -> &Arc<RwLock<HashMap<u32, u32>>> {
        &self.items
    }

    pub fn check_items(&self, item_id: u32) -> Result<u32> {
        if let Ok(inner_map) = self.items.read() {
            if let Some(quantity) = inner_map.get(&item_id) {
                return Ok(*quantity);
            }
            return Err(Error::NoSuchItem);
        }
        return Err(Error::FailedToCheck);
    }

    pub fn add_items(&self, item_id: u32, quantity: u32) -> Result<u32> {
        if let Ok(mut inner_map) = self.items.write() {
            let current_quantity = inner_map.entry(item_id).or_insert(0);
            let updated_quantity = *current_quantity + quantity;
            inner_map.insert(item_id, updated_quantity);
            return Ok(updated_quantity);
        };
        return Err(Error::FailedToCheck);
    }
}

impl Default for Storage {
    fn default() -> Self {
        let mut storage = Storage::new();
        {
            for i in 0..7 {
                storage.add_items(i, i * 2).expect("TODO: panic message");
            }
        }
        storage
    }
}