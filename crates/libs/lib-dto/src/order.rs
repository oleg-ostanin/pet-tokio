use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, Deserialize, Serialize)]
#[sqlx(type_name = "order_status", rename_all = "lowercase")]
pub enum OrderStatus {
    New,
    InProgress,
    ReadyToDeliver,
    Delivered
}

pub struct Order {
    id: i64,
    user_id: i64,
    content: OrderContent,
    status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct OrderItem {
    book_id: i64,
    quantity: i64,
}

pub struct OrderContent {
    content: Vec<OrderItem>,
}