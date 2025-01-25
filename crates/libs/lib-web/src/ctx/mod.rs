use std::sync::Arc;
use axum::async_trait;
use axum::http::request::Parts;
use axum::extract::{FromRequestParts, State};
use serde::Serialize;
use tracing::info;

use crate::error::{Error, Result};

#[cfg_attr(feature = "with-rpc", derive(rpc_router::RpcResource))]
#[derive(Clone, Debug)]
pub struct Ctx {
    phone: String,
}

// Constructors.
impl Ctx {
    pub fn new(phone: String) -> Self {
        Self {
            phone,
        }
    }

    pub fn phone(&self) -> &String {
        &self.phone
    }
}

// region:    --- Ctx Extractor
#[derive(Debug, Clone)]
pub struct CtxW(pub Ctx);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for CtxW {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        info!("{:<12} - Ctx", "EXTRACTOR");

        let res = parts
            .extensions
            .get::<CtxExtResult>()
            //.ok_or(Error::CtxExt(CtxExtError::CtxNotInRequestExt))?
            .ok_or(Error::WebError)?
            .clone();
            //.map_err(Error::CtxExt)

        match res {
            Ok(res) => Ok(res),
            Err(_) => Err(Error::WebError)
        }
    }
}
// endregion: --- Ctx Extractor

// region:    --- Ctx Extractor Result/Error
pub type CtxExtResult = core::result::Result<CtxW, CtxExtError>;

#[derive(Clone, Serialize, Debug)]
pub enum CtxExtError {
    TokenNotInCookie,
    TokenWrongFormat,

    UserNotFound,
    ModelAccessError(String),
    FailValidate,
    CannotSetTokenCookie,

    CtxNotInRequestExt,
    CtxCreateFail(String),
}
// endregion: --- Ctx Extractor Result/Error


