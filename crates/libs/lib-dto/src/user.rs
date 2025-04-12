use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
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

#[derive(Debug, Deserialize, Serialize)]
pub struct UserExists {
    pub exists: bool,
}

impl UserExists {
    pub fn new(exists: bool) -> Self {
        Self { exists }
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
    pub pwd_salt: Uuid,
}

#[derive(Clone, Debug, FromRow, Deserialize, Serialize)]
pub struct UserStored {
    id: i64,
    phone: String,
    first_name: String,
    last_name: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl UserStored {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn phone(&self) -> &str {
        &self.phone
    }

    pub fn first_name(&self) -> &str {
        &self.first_name
    }

    pub fn last_name(&self) -> &str {
        &self.last_name
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}