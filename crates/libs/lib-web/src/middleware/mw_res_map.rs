//use crate::log::log_request;
//use crate::middleware::mw_auth::CtxW;
use crate::middleware::mw_req_stamp::ReqStamp;
//use crate::handlers::handlers_rpc::RpcInfo;
use crate::error::Error;

use axum::http::{Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, to_value};
use std::sync::Arc;
use tower_cookies::Cookies;
use tracing::debug;
use uuid::Uuid;

pub async fn mw_response_map(
	//ctx: Option<CtxW>,
	uri: Uri,
	req_method: Method,
	req_stamp: ReqStamp,
	res: Response,
) -> Response {
	//let ctx = ctx.map(|ctx| ctx.0);

	debug!("{:<12} - mw_response_map", "RES_MAPPER");
	let uuid = Uuid::new_v4();

	//let rpc_info = res.extensions().get::<Arc<RpcInfo>>().map(Arc::as_ref);

	// -- Get the eventual response error.
	let web_error = res.extensions().get::<Arc<Error>>().map(Arc::as_ref);

    if let Some(web_error) = res.extensions().get::<Arc<Error>>().map(Arc::as_ref) {
		// -- If client error, build the new response.
		return  StatusCode::INTERNAL_SERVER_ERROR.into_response();
	}

	res
}
