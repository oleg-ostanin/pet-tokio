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
    pub identity: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Debug, Deserialize, Serialize, Builder)]
pub struct AuthCode {
    pub auth_code: String,
}

impl UserForCreate {
    pub fn new(
        identity: impl Into<String>,
        password: impl Into<String>,
        first_name: impl Into<String>,
        last_name: impl Into<String>,
    ) -> Self {
        UserForCreate {
            identity: identity.into(),
            password: password.into(),
            first_name: first_name.into(),
            last_name: last_name.into(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserForSignIn {
    pub identity: String,
    pub password: String,
}

impl UserForSignIn {
    pub fn new(
        identity: impl Into<String>,
        password: impl Into<String>,

    ) -> Self {
        UserForSignIn {
            identity: identity.into(),
            password: password.into(),
        }
    }
}

#[derive(Clone, FromRow, Debug)]
pub struct UserForAuth {
    pub id: i64,
    pub identity: String,

    // -- token info
    pub token_salt: Uuid,
}

#[derive(Clone, FromRow, Debug)]
pub struct UserForLogin {
    pub id: i64,
    pub identity: String,
    pub pwd: Option<String>,

    // -- pwd info
    pub pwd_salt: Uuid,
    // -- token info
    pub token_salt: Uuid,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserStored {
    pub id: i64,
    pub identity: String,
    pub first_name: String,
    pub last_name: String,
    pub pwd: String, // todo remove
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub enum UserIdentity {
    Phone(String),
    Email(String),
}