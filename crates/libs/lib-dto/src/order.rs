use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, PartialEq, PartialOrd, sqlx::Type, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "order_status", rename_all = "snake_case")]
pub enum OrderStatus {
    New,
    InProgress,
    ReadyToDeliver,
    Delivered
}

#[derive(Clone, FromRow, Debug, Serialize, Deserialize)]
pub struct OrderStored {
    order_id: i64,
    user_id: i64,
    content: sqlx::types::Json<OrderContent>,
    status: OrderStatus,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
 impl OrderStored {
     pub fn order_id(&self) -> i64 {
         self.order_id
     }

     pub fn user_id(&self) -> i64 {
         self.user_id
     }

     pub fn content(&self) -> &sqlx::types::Json<OrderContent> {
         &self.content
     }

     pub fn status(&self) -> &OrderStatus {
         &self.status
     }

     pub fn created_at(&self) -> DateTime<Utc> {
         self.created_at
     }

     pub fn updated_at(&self) -> DateTime<Utc> {
         self.updated_at
     }
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderItem {
    book_id: i64,
    quantity: i64,
}

impl OrderItem {
    pub fn new(book_id: i64, quantity: i64) -> Self {
        Self { book_id, quantity }
    }

    pub fn book_id(&self) -> i64 {
        self.book_id
    }

    pub fn quantity(&self) -> i64 {
        self.quantity
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderItemExt {
    order_id: i64,
    book_id: i64,
    quantity: i64,
}

impl OrderItemExt {
    pub fn new(order_id: i64, book_id: i64, quantity: i64) -> Self {
        Self { order_id, book_id, quantity }
    }

    pub fn order_id(&self) -> i64 {
        self.order_id
    }

    pub fn book_id(&self) -> i64 {
        self.book_id
    }

    pub fn quantity(&self) -> i64 {
        self.quantity
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrderContent {
    content: Vec<OrderItem>,
}

impl OrderContent {
    pub fn new(content: Vec<OrderItem>) -> Self {
        Self { content }
    }

    pub fn content(&self) -> &Vec<OrderItem> {
        &self.content
    }
}

#[derive(Clone, Debug, FromRow, Serialize, Deserialize)]
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