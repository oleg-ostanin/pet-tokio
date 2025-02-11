use serde_json::{json, Value};

use lib_core::bmc::book_info::BookBmc;
use lib_core::context::app_context::ModelManager;
use lib_dto::book::BookList;

use crate::error::Result;

pub(super) async fn add_books(mm: &ModelManager, params: Value) -> Result<Value> {
    let book_list: BookList = serde_json::from_str(&params.to_string()).unwrap();
    for book_info in book_list.book_list.into_iter() {
        BookBmc::create(mm, book_info).await.unwrap();
    }
    Ok(Value::Null)
}

pub(super) async fn all_books(mm: &ModelManager) -> Result<Value> {
    Ok(json!(BookBmc::get_all(mm).await?))
}