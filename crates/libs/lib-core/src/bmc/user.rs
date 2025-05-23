use chrono::prelude::*;
use tracing::info;
use uuid::Uuid;

use lib_dto::user::{UserExists, UserForCreate, UserForLogin, UserForSignIn, UserStored};

use crate::bmc::scheme::Scheme;
use crate::context::app_context::ModelManager;
use crate::error::{Error, Result};

pub struct UserBmc;
const INSERT_USER: &str = r#"
INSERT INTO users
(phone, first_name, last_name, pwd, pwd_salt, token_salt, created_at, updated_at)
VALUES
($1, $2, $3, $4, $5, $6, $7, $8)
RETURNING id;
"#;

const SELECT_BY_ID: &str = r#"
SELECT * FROM users WHERE id=$1;
"#;

const SELECT_BY_PHONE: &str = r#"
SELECT * FROM users WHERE phone=$1;
"#;

impl UserBmc {
    pub async fn create(
        mm: &ModelManager,
        user: UserForCreate,
    ) -> Result<String> {
        let pwd_salt = Uuid::new_v4();
        let to_hash = ContentToHash {
            content: user.password.clone(),
            salt: pwd_salt,
        };

        let pwd_hashed = hash_pwd(to_hash).await?;
        let token_salt = Uuid::new_v4();

        sqlx::query(INSERT_USER)
            .bind(&user.phone)
            .bind(&user.first_name)
            .bind(&user.last_name)
            .bind(&pwd_hashed)
            .bind(pwd_salt)
            .bind(token_salt)
            .bind(Utc::now())
            .bind(Utc::now())
            .execute(mm.pg_pool())
            .await?;

        let res = Uuid::new_v4().to_string();
        if let Ok(mut map) = mm.cache().write() {
            map.insert(user.phone, res.clone());
        }

        Ok(res)
    }

    pub async fn get_by_phone(
        mm: &ModelManager,
        phone: &str,
    ) -> Result<UserStored> {
        let user: UserStored = sqlx::query_as(SELECT_BY_PHONE)
            .bind(phone)
            .fetch_one(mm.pg_pool())
            .await?;

        Ok(user)
    }

    pub async fn get_by_id(
        mm: &ModelManager,
        id: i64,
    ) -> Result<UserStored> {
        info!("Trying to get user by id: {:#?}", id);

        let user: UserStored = sqlx::query_as(SELECT_BY_ID)
            .bind(id)
            .fetch_one(mm.pg_pool())
            .await?;

        Ok(user)
    }
    //
    // pub async fn get_for_auth(
    //     mm: &ModelManager,
    //     phone: &String,
    // ) -> Result<UserForAuth> {
    //     //let res = db_client.execute(&statement, &[&user.uuid, &user.pass]).await?;
    //     let res = mm.client().query(SELECT_BY_phone, &[phone]).await?;
    //
    //     info!("{:#?}", &res);
    //
    //     let v = res.get(0).ok_or(Error::StoreError("not_found".to_string()))?;
    //
    //     UserForAuth::try_from(v)
    // }
    //
    pub async fn validate(
        mm: &ModelManager,
        user_for_sign_in: &UserForSignIn,
    ) -> Result<()> {
        let user: UserForLogin = sqlx::query_as(SELECT_BY_PHONE)
            .bind(&user_for_sign_in.phone)
            .fetch_one(mm.pg_pool())
            .await?;

        let to_hash = ContentToHash {
            content: user_for_sign_in.password.clone(),
            salt: user.pwd_salt,
        };

        validate_pwd(to_hash, user.pwd).await?;

        Ok(())
    }

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

/// The clean content to hash, with the salt.
///
/// Notes:
///    - Since content is sensitive information, we do NOT implement default debug for this struct.
///    - The clone is only implement for testing
#[cfg_attr(test, derive(Clone))]
pub struct ContentToHash {
    pub content: String, // Clear content.
    pub salt: Uuid,
}

// endregion: --- Types

// region:    --- Public Functions

/// Hash the password with the default scheme.
pub async fn hash_pwd(to_hash: ContentToHash) -> Result<String> {
    let s = Scheme;

    tokio::task::spawn_blocking(move || s.hash(&to_hash))
        .await
        .map_err(|_| Error::CoreError)?
}

pub async fn validate_pwd(to_hash: ContentToHash, pwd: String) -> Result<()> {
    let s = Scheme;

    tokio::task::spawn_blocking(move || s.validate(&to_hash, pwd))
        .await
        .map_err(|_| Error::CoreError)?
}