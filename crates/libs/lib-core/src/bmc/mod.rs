pub mod scheme;
pub mod user;

use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Serialize)]
pub enum Error {
    SomeError,
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
        println!("{:?}", value);
        Error::SomeError
    }
}