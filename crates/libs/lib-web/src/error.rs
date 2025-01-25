use std::collections::HashMap;
use std::sync::{Arc, PoisonError, RwLockReadGuard, RwLockWriteGuard};

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_json::Value;
use serde_with::{DisplayFromStr, serde_as};
use tracing::{debug, warn};

use crate::middleware;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Serialize)]
pub enum Error {
	// -- Extractors
    WebError,
    FailedToWriteCache,
    FailedToReadCache,
	ReqStampNotInReqExt,
}

// region:    --- Axum IntoResponse
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        debug!("{:<12} - model::Error {self:?}", "INTO_RES");

        // Create a placeholder Axum response.
        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

        // Insert the Error into the response.
        response.extensions_mut().insert(Arc::new(self));

        response
    }
}
// endregion: --- Axum IntoResponse

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

impl From<lib_core::error::Error> for Error {
    fn from(value: lib_core::error::Error) -> Self {
        Error::WebError
    }
}

impl From<PoisonError<RwLockWriteGuard<'_, HashMap<String, String>>>> for Error {
    fn from(value: PoisonError<RwLockWriteGuard<'_, HashMap<String, String>>>) -> Self {
        Error::FailedToWriteCache
    }
}

impl From<PoisonError<RwLockReadGuard<'_, HashMap<String, String>>>> for Error {
    fn from(value: PoisonError<RwLockReadGuard<'_, HashMap<String, String>>>) -> Self {
        Error::FailedToWriteCache
    }
}
