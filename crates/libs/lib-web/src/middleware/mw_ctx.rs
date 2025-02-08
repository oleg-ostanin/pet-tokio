use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use tower_cookies::Cookies;
use tracing::info;

use lib_core::context::app_context::ModelManager;
use lib_utils::constants::AUTH_TOKEN;
use lib_utils::jwt::phone_from_token;

use crate::ctx::{Ctx, CtxExtError, CtxExtResult, CtxW};
use crate::error::{Error, Result};

pub async fn mw_ctx_check(
    ctx: Result<CtxW>,
    req: Request<Body>,
    next: Next,
) -> Result<Response> {
    info!("{:<12} - mw_ctx_require - {ctx:?}", "MIDDLEWARE");

    ctx?;

    Ok(next.run(req).await)
}

// IMPORTANT: This resolver must never fail, but rather capture the potential Auth error and put in in the
//            request extension as CtxExtResult.
//            This way it won't prevent downstream middleware to be executed, and will still capture the error
//            for the appropriate middleware (.e.g., mw_ctx_require which forces successful auth) or handler
//            to get the appropriate information.
pub async fn mw_ctx_create(
    State(mm): State<Arc<ModelManager>>,
    cookies: Cookies,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    info!("{:<12} - mw_ctx_resolve", "MIDDLEWARE");

    let ctx_ext_result = ctx_resolve(mm, &cookies).await;

    // Store the ctx_ext_result in the request extension
    // (for Ctx extractor).
    req.extensions_mut().insert(ctx_ext_result);

    next.run(req).await
}

async fn ctx_resolve(mm: Arc<ModelManager>, cookies: &Cookies) -> CtxExtResult {
    // -- Get Token String
    let token = cookies
        .get(AUTH_TOKEN)
        .map(|c| c.value().to_string())
        .ok_or(CtxExtError::TokenNotInCookie)?;
    info!("Token in ctx resolve: {:?}", token);

    // -- Parse Token
    let token_key = std::env::var("SERVICE_TOKEN_KEY").expect("TOKEN must be set.");
    info!("Token_key in ctx resolve: {:?}", token_key);

    let phone = phone_from_token(token, &token_key);
    info!("phone in ctx resolve: {:?}", phone);


    if let Some(phone) = phone {
        Ok(CtxW(Ctx::new(phone)))
    } else {
        Err(CtxExtError::TokenNotInCookie)
    }
}