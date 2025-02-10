use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, Deserialize, Serialize)]
#[sqlx(type_name = "order_status", rename_all = "lowercase")]
pub enum OrderStatus {
    New,
    InProgress,
    ReadyToDeliver,
    Delivered
}

#[derive(Clone, FromRow, Debug)]
pub struct OrderStored {
    id: i64,
    user_id: i64,
    content: sqlx::types::Json<OrderContent>,
    status: OrderStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub struct OrderForCreate {
    user_id: i64,
    content: OrderContent,
}

impl OrderForCreate {
    pub fn new(user_id: i64, content: OrderContent) -> Self {
        Self { user_id, content }
    }

    pub fn user_id(&self) -> i64 {
        self.user_id
    }

    pub fn content(&self) -> &OrderContent {
        &self.content
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct OrderItem {
    book_id: i64,
    quantity: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct OrderContent {
    content: Vec<OrderItem>,
}

#[derive(Clone, Debug, FromRow)]
pub struct OrderId {
    order_id: i64,
}

impl OrderId {
    pub fn new(order_id: i64) -> Self {
        Self { order_id }
    }

    pub fn order_id(&self) -> i64 {
        self.order_id
    }
}