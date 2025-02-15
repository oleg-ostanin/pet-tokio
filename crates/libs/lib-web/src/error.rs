use std::collections::HashMap;
use std::sync::{Arc, PoisonError, RwLockReadGuard, RwLockWriteGuard};

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_json::Value;
use serde_with::{DisplayFromStr, serde_as};
use strum_macros;
use tracing::{debug, error, info, warn};
use crate::ctx::CtxExtError;
use crate::middleware;

pub type Result<T> = core::result::Result<T, Error>;

//#[serde_as]
#[derive(Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error {
    WebError,
    Anyhow,

    UnauthorizedAccess,

    RpcRequestParsing,
    RpcNoParams,
    UnknownRpcMethod,

    // -- CtxExtError
    CtxExt(CtxExtError),

    FailedToConvertJson,
    FailedToSendRequest,
    FailedToWriteCache,
    FailedToReadCache,
	ReqStampNotInReqExt,
}

// region:    --- Axum IntoResponse
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        info!("{:<12} - model::Error {self:?}", "INTO_RES");

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

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        error!("{:?}", value);
        Error::FailedToConvertJson
    }
}

impl From<hyper_util::client::legacy::Error> for Error {
    fn from(value: hyper_util::client::legacy::Error) -> Self {
        error!("{:?}", value);
        Error::FailedToSendRequest
    }
}

impl From<anyhow::Error> for Error {
    fn from(value: anyhow::Error) -> Self {
        error!("{:?}", &value);
        Error::Anyhow
    }
}

impl From<CtxExtError> for Error {
    fn from(value: CtxExtError) -> Self {
        Error::CtxExt(value)
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

/// From the root error to the http status code and ClientError
impl Error {
    pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
        use Error::*; // TODO: should change to `use web::Error as E`

        match self {
            // -- Login
            WebError => {
                (StatusCode::FORBIDDEN, ClientError::LOGIN_FAIL)
            }

            // -- Auth
            CtxExt(_) => (StatusCode::FORBIDDEN, ClientError::LOGIN_FAIL),

            // -- Auth
            FailedToConvertJson => (StatusCode::BAD_REQUEST, ClientError::LOGIN_FAIL),
            //
            // // -- Model
            // Model(model::Error::EntityNotFound { entity, id }) => (
            //     StatusCode::BAD_REQUEST,
            //     ClientError::ENTITY_NOT_FOUND { entity, id: *id },
            // ),
            //
            // // -- Rpc
            // RpcRequestParsing(req_parsing_err) => (
            //     StatusCode::BAD_REQUEST,
            //     ClientError::RPC_REQUEST_INVALID(req_parsing_err.to_string()),
            // ),
            // RpcRouter {
            //     error: rpc_router::Error::MethodUnknown,
            //     method,
            //     ..
            // } => (
            //     StatusCode::BAD_REQUEST,
            //     ClientError::RPC_REQUEST_METHOD_UNKNOWN(format!(
            //         "rpc method '{method}' unknown"
            //     )),
            // ),
            // RpcRouter {
            //     error: rpc_router::Error::ParamsParsing(params_parsing_err),
            //     ..
            // } => (
            //     StatusCode::BAD_REQUEST,
            //     ClientError::RPC_PARAMS_INVALID(params_parsing_err.to_string()),
            // ),
            // RpcRouter {
            //     error: rpc_router::Error::ParamsMissingButRequested,
            //     method,
            //     ..
            // } => (
            //     StatusCode::BAD_REQUEST,
            //     ClientError::RPC_PARAMS_INVALID(format!(
            //         "Params missing. Method '{method}' requires params"
            //     )),
            // ),

            // -- Fallback.
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ClientError::SERVICE_ERROR,
            ),
        }
    }
}


#[derive(Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "message", content = "detail")]
#[allow(non_camel_case_types)]
pub enum ClientError {
    LOGIN_FAIL,
    NO_AUTH,
    ENTITY_NOT_FOUND { entity: &'static str, id: i64 },

    RPC_REQUEST_INVALID(String),
    RPC_REQUEST_METHOD_UNKNOWN(String),
    RPC_PARAMS_INVALID(String),

    SERVICE_ERROR,
}