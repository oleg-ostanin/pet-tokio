#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use crate::common::order::Order;
    use crate::common::processors::check_order_processor::CheckStorageOrderProcessor;
    use crate::common::storage::Storage;

    #[test]
    fn simple() {
        let mut items = HashMap::new();
        items.insert(2u32, 4);
        let order = Order::default_with_items(items);
        let storage: Storage = Storage::default();


        let cop = CheckStorageOrderProcessor::new(storage);
    }
}