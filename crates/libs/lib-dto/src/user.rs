use std::str::FromStr;
use std::time::SystemTime;

use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tokio_postgres::{Error, Row};
use tokio_postgres::types::ToSql;
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize, Builder)]
pub struct UserForCreate {
    pub phone: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Deserialize, Serialize, Builder)]
pub struct AuthCode {
    pub phone: String,
    pub auth_code: String,
}

impl AuthCode {
    pub fn new(phone: impl Into<String>, auth_code: impl Into<String>) -> Self {
        Self { phone: phone.into(), auth_code: auth_code.into() }
    }
}

impl UserForCreate {
    pub fn new(
        phone: impl Into<String>,
        password: impl Into<String>,
        first_name: impl Into<String>,
        last_name: impl Into<String>,
    ) -> Self {
        UserForCreate {
            phone: phone.into(),
            password: password.into(),
            first_name: first_name.into(),
            last_name: last_name.into(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserForSignIn {
    pub phone: String,
    pub password: String,
}

impl UserForSignIn {
    pub fn new(
        phone: impl Into<String>,
        password: impl Into<String>,

    ) -> Self {
        UserForSignIn {
            phone: phone.into(),
            password: password.into(),
        }
    }
}

#[derive(Clone, FromRow, Debug)]
pub struct UserForAuth {
    pub id: i64,
    pub phone: String,

    // -- token info
    pub token_salt: Uuid,
}

#[derive(Clone, FromRow, Debug)]
pub struct UserForLogin {
    pub id: i64,
    pub phone: String,
    pub pwd: String,

    // -- pwd info
    pub pwd_salt: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserStored {
    pub id: i64,
    pub phone: String,
    pub first_name: String,
    pub last_name: String,
    pub pwd: String, // todo remove
    //pub created_at: DateTime<Utc>,
    //pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub enum Userphone {
    Phone(String),
    Email(String),
}