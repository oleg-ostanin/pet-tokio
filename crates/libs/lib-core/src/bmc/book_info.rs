use lib_dto::book::{BookInfo, BookList, BookStorageInfo};
use lib_dto::order::OrderItem;
use crate::context::app_context::ModelManager;
use crate::error::Result;

pub struct BookBmc;
const INSERT_BOOK: &str = r#"
INSERT INTO book_info
(title, author, isbn, description
  --, created_at, updated_at todo
)
VALUES
($1, $2, $3, $4
  -- , $5, $6 todo
)
RETURNING id;
"#;

const SELECT_ALL: &str = r#"
SELECT * FROM book_info;
"#;

const SELECT_BY_ID: &str = r#"
SELECT * FROM book_info WHERE id=$1;
"#;

const SELECT_BY_TITLE: &str = r#"
SELECT * FROM book_info WHERE title=$1;
"#;

impl BookBmc {
    pub async fn create(
        mm: &ModelManager,
        book: &BookInfo,
    ) -> Result<()> {
        sqlx::query_as(INSERT_BOOK)
            .bind(&book.title)
            .bind(&book.author)
            .bind(&book.isbn)
            .bind(&book.description)
            .fetch_one(mm.pg_pool())
            .await?;

        Ok(())
    }

    pub async fn get_all(
        mm: &ModelManager,
    ) -> Result<BookList> {
        let books: Vec<BookInfo> = sqlx::query_as(SELECT_ALL)
            .fetch_all(mm.pg_pool())
            .await?;

        Ok(BookList::new(books))
    }
}