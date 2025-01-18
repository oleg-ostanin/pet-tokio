use std::collections::HashMap;

#[derive(Default, Debug, Clone)]
pub(crate) struct Order {
    id: u32,
    customer_id: u32,
    pub(crate) items: HashMap<u32, u32>,
    comment: Option<String>,
}

impl Order {
    pub(crate) fn new(id: u32, customer_id: u32, items: HashMap<u32, u32>, comment: Option<String>) -> Self {
        Self { id, customer_id, items, comment }
    }

    pub(crate) fn default_with_items(items: HashMap<u32, u32>) -> Self {
        Order {
            items,
            .. Order::default()
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn customer_id(&self) -> u32 {
        self.customer_id
    }

    pub fn items(&self) -> &HashMap<u32, u32> {
        &self.items
    }

    pub fn comment(&self) -> &Option<String> {
        &self.comment
    }
}