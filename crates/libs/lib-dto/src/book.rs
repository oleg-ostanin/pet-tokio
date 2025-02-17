
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Deserialize, Serialize, Builder, FromRow)]
pub struct BookInfo {
    pub title: String,
    pub author: Option<String>,
    pub isbn: String,
    pub description: String,
}

impl BookInfo {
    pub fn new(title: String, author: Option<String>, isbn: String, description: String) -> Self {
        Self { title, author, isbn, description }
    }
}

#[derive(Debug, Deserialize, Serialize,)]
pub struct BookList {
    book_list: Vec<BookInfo>,
}

impl BookList {
    pub fn new(book_list: Vec<BookInfo>) -> Self {
        Self { book_list }
    }

    pub fn book_list(&self) -> &Vec<BookInfo> {
        &self.book_list
    }
}