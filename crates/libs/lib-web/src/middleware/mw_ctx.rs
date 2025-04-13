use axum::body::Body;
use axum::extract::{Request};
use axum::middleware::Next;
use axum::response::Response;
use tower_cookies::Cookies;
use tracing::debug;

use lib_utils::constants::AUTH_TOKEN;
use lib_utils::jwt::phone_from_token;

use crate::ctx::{Ctx, CtxExtError, CtxExtResult, CtxW};
use crate::error::Result;

pub async fn mw_ctx_check(
    ctx: Result<CtxW>,
    req: Request<Body>,
    next: Next,
) -> Result<Response> {
    debug!("{:<12} - ctx: {ctx:#?} req: {req:#?}", "MIDDLEWARE");

    ctx?;

    Ok(next.run(req).await)
}

// IMPORTANT: This resolver must never fail, but rather capture the potential Auth error and put in in the
//            request extension as CtxExtResult.
//            This way it won't prevent downstream middleware to be executed, and will still capture the error
//            for the appropriate middleware (.e.g., mw_ctx_require which forces successful auth) or handler
//            to get the appropriate information.
pub async fn mw_ctx_create(
    cookies: Cookies,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    debug!("{:<12} - mw_ctx_resolve", "MIDDLEWARE");

    let ctx_ext_result = ctx_resolve(&cookies).await;

    // Store the ctx_ext_result in the request extension
    // (for Ctx extractor).
    req.extensions_mut().insert(ctx_ext_result);

    next.run(req).await
}

async fn ctx_resolve(cookies: &Cookies) -> CtxExtResult {
    // -- Get Token String
    let token = cookies
        .get(AUTH_TOKEN)
        .map(|c| c.value().to_string())
        .ok_or(CtxExtError::TokenNotInCookie)?;
    debug!("Token in ctx resolve: {:#?}", token);

    // -- Parse Token
    let token_key = std::env::var("SERVICE_TOKEN_KEY").expect("TOKEN must be set.");
    debug!("Token_key in ctx resolve: {:#?}", token_key);

    let phone = phone_from_token(&token, &token_key);
    debug!("phone in ctx resolve: {:#?}", phone);


    if let Some(phone) = phone {
        Ok(CtxW(Ctx::new(phone)))
    } else {
        Err(CtxExtError::TokenNotInCookie)
    }
}