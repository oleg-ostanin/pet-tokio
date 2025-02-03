use serde::Serialize;
use tracing::info;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Serialize)]
pub enum Error {
    CoreError,
}

// region:    --- Error Boilerplate

impl core::fmt::Display for Error {
    fn fmt(
        &self,
        fmt: &mut core::fmt::Formatter,
    ) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

// endregion: --- Error Boilerplate

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        info!("sqlx error: {:?}", value);
        Error::CoreError
    }
}