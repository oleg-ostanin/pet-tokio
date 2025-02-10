use chrono::prelude::*;
use uuid::Uuid;
use lib_dto::order::{OrderForCreate, OrderId, OrderStatus};
use lib_dto::user::{UserExists, UserForCreate, UserForLogin, UserForSignIn};

use crate::bmc::scheme::Scheme;
use crate::context::app_context::ModelManager;
use crate::error::{Error, Result};

pub struct OrderBmc;
const INSERT_USER: &str = r#"
INSERT INTO user
(user_id, content, status, created_at, updated_at)
VALUES
($1, $2, $3, $4, $5)
RETURNING order_id;
"#;

const SELECT_BY_ID: &str = r#"
SELECT * FROM order WHERE id=$1;
"#;

const SELECT_BY_PHONE: &str = r#"
SELECT * FROM users WHERE phone=$1;
"#;

impl OrderBmc {
    pub async fn create(
        mm: &ModelManager,
        order: OrderForCreate,
    ) -> Result<OrderId> {

        let json = sqlx::types::Json::from(order.content());
        let order_id = sqlx::query_as(INSERT_USER)
            .bind(&order.user_id())
            .bind(json)
            .bind(OrderStatus::New)
            .bind(Utc::now())
            .bind(Utc::now())
            .fetch_one(mm.pg_pool())
            .await?;

        Ok(order_id)
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


    pub async fn check_if_exists(
        mm: &ModelManager,
        phone: String,
    ) -> Result<UserExists> {
        let users: Vec<UserForLogin> = sqlx::query_as(SELECT_BY_PHONE)
            .bind(&phone)
            .fetch_all(mm.pg_pool())
            .await?;

        let user_exists = UserExists::new(!users.is_empty());

        Ok(user_exists)
    }
}


