use serde_json::{json, Value};
use tracing::info;

use lib_core::bmc::book_info::BookBmc;
use lib_core::context::app_context::ModelManager;
use lib_dto::book::{BookDescription, BookList};

use crate::error::Result;

pub(super) async fn add_books(mm: &ModelManager, params: Value) -> Result<Value> {
    let books = BookBmc::get_all(mm).await?;
    if books.book_list().len() == 5 {
        info!("All books are already inserted.");
        return Ok(Value::Null);
    }

    let book_list: BookList = serde_json::from_value(params)?;
    for book_info in book_list.book_list().iter() {
        BookBmc::create(mm, book_info).await?;
    }
    Ok(Value::Null)
}

pub(super) async fn all_books(mm: &ModelManager) -> Result<Value> {
    Ok(json!(BookBmc::get_all(mm).await?))
}

pub(super) async fn books_by_description(mm: &ModelManager, params: Value) -> Result<Value> {
    let description: BookDescription = serde_json::from_value(params)?;
    Ok(json!(BookBmc::get_by_description(mm, description.description()).await?))
}