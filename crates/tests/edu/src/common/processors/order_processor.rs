use std::sync::Arc;
use crate::common::order::Order;
use crate::common::error::{Result, Error};
use crate::common::storage::Storage;

pub trait OrderProcessor {
    fn process_order(&self, order: Order) -> Result<Order>;
}