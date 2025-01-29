use uuid::Uuid;
use lib_dto::book::BookInfo;
use lib_dto::user::{UserForCreate, UserForLogin, UserForSignIn};
use crate::bmc::scheme::Scheme;
use crate::context::app_context::ModelManager;

use crate::error::{Result, Error};

pub struct BookBmc;
//todo return phone type
const INSERT_BOOK: &str = r#"
INSERT INTO book_info
(title, author, isbn, description,
  --, created_at, updated_at todo
)
VALUES
($1, $2, $3, $4
  -- , $5, $6 todo
)
RETURNING id;
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
        book: BookInfo,
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

    // pub async fn get_by_id(
    //     mm: &ModelManager,
    //     id: i64,
    // ) -> Result<UserStored> {
    //     //let res = db_client.execute(&statement, &[&user.uuid, &user.pass]).await?;
    //     let res = mm.client().query(SELECT_BY_ID, &[&id]).await?;
    //
    //     println!("{:?}", &res);
    //
    //     let v = res.get(0).ok_or(Error::StoreError("not_found".to_string()))?;
    //
    //     UserStored::try_from(v)
    // }
    //
    // pub async fn get_for_auth(
    //     mm: &ModelManager,
    //     phone: &String,
    // ) -> Result<UserForAuth> {
    //     //let res = db_client.execute(&statement, &[&user.uuid, &user.pass]).await?;
    //     let res = mm.client().query(SELECT_BY_phone, &[phone]).await?;
    //
    //     println!("{:?}", &res);
    //
    //     let v = res.get(0).ok_or(Error::StoreError("not_found".to_string()))?;
    //
    //     UserForAuth::try_from(v)
    // }
    //
    // // todo make these two functions generic
    // pub async fn validate(
    //     mm: &ModelManager,
    //     user_for_sign_in: &UserForSignIn,
    // ) -> Result<()> {
    //     let user: UserForLogin = sqlx::query_as(SELECT_BY_PHONE)
    //         .bind(&user_for_sign_in.phone)
    //         .fetch_one(mm.pg_pool())
    //         .await?;
    //
    //     let to_hash = ContentToHash {
    //         content: user_for_sign_in.password.clone(),
    //         salt: Uuid::parse_str(&user.pwd_salt).unwrap(), // todo make it uuid in the database
    //     };
    //
    //     validate_pwd(to_hash, user.pwd).await?;
    //
    //     Ok(())
    // }
}