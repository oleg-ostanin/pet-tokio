use std::sync::Arc;
use crate::common::order::Order;
use crate::common::error::{Result, Error};
use crate::common::processors::order_processor::OrderProcessor;
use crate::common::storage::Storage;

pub(crate) struct CheckStorageOrderProcessor {
    storage: Storage,
}

impl CheckStorageOrderProcessor {
    pub(crate) fn new(storage: Storage) -> Self {
        Self { storage }
    }
}

impl OrderProcessor for CheckStorageOrderProcessor {
    fn process_order(&self, order: Order) -> Result<Order> {
        let order_items = order.items();

        for (item_id, order_quantity) in order_items.iter() {
            let storage_quantity = self.storage.check_items(*item_id)?;
            if storage_quantity < *order_quantity {
                return Err(Error::NotEnoughItems);
            }
        }
        return Ok(order);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use super::*;

    fn check_storage() -> CheckStorageOrderProcessor {
        let storage = Storage::default();
        CheckStorageOrderProcessor::new(storage)
    }

    #[test]
    fn enough() {
        let mut items = HashMap::new();
        items.insert(2u32, 4);
        let order = Order::default_with_items(items);
        let res = check_storage().process_order(order);
        assert!(res.is_ok())
    }

    #[test]
    fn not_enough() {
        let mut items = HashMap::new();
        items.insert(2u32, 5);
        let order = Order::default_with_items(items);
        let res = check_storage().process_order(order);
        assert!(res.is_err_and(|e| e == Error::NotEnoughItems ))
    }
}