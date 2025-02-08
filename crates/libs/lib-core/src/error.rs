use std::env::VarError;
use serde::Serialize;
use thiserror::Error;
use tracing::{error, info};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Sqlx error: {0}")]
    Sqlx(sqlx::Error),
    #[error("Core error")]
    CoreError,
    #[error("Wrong password")]
    WrongPassword,
    #[error("Var error: {0}")]
    VarError(#[from] VarError),
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        error!("Sqlx error: {:?}", &value);
        Error::Sqlx(value)
    }
}