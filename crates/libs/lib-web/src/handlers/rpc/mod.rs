use std::ops::Deref;
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde_json::{json, Value};

use book::*;
use lib_core::context::app_context::ModelManager;
use order::create_order;

use crate::ctx::CtxW;

pub mod book;
pub mod order;
pub mod rpc;

/// RPC ID and Method Capture
/// Note: This will be injected into the Axum Response extensions so that
///       it can be used downstream by the `mw_res_map` for logging and eventual
///       error client JSON-RPC serialization
#[derive(Debug)]
pub struct RpcInfo {
    pub id: Option<Value>,
    pub method: String,
}

pub async fn rpc(
    State(app_context): State<Arc<ModelManager>>,
    ctx: CtxW,
    Json(rpc_req): Json<Value>,
) -> Response {
    let ctx = ctx.0;

    // // -- Parse and RpcRequest validate the rpc_request
    let rpc_req = match rpc_router::Request::try_from(rpc_req) {
        Ok(rpc_req) => rpc_req,
        Err(rpc_req_error) => {
            let res = StatusCode::INTERNAL_SERVER_ERROR.into_response();
            //crate::error::Error::RpcRequestParsing(rpc_req_error).into_response();
            return res;
        }
    };

    println!("rpc_req: {:?}", &rpc_req.params);

    // -- Create the RPC Info
    //    (will be set to the response.extensions)
    let rpc_info = RpcInfo {
        id: Some(rpc_req.id.clone()),
        method: rpc_req.method.clone(),
    };

    let call_res = match rpc_req.method.as_str() {
        "add_books" => add_books(app_context.deref(), rpc_req.params.expect("must be")).await,
        "all_books" => all_books(app_context.deref()).await,
        "create_order" => create_order(app_context.deref(), rpc_req.params.expect("must be"), ctx).await,
        _ => unreachable!(),
    };

    //
    // // -- Add the request specific resources
    // // Note: Since Ctx is per axum request, we construct additional RPC resources.
    // //       These additional resources will be "overlayed" on top of the base router services,
    // //       meaning they will take precedence over the base router ones, but won't replace them.
    // let additional_resources = resources_builder![ctx].build();
    //
    // // -- Exec Rpc Route
    // let rpc_call_result = rpc_router
    // 	.call_with_resources(rpc_req, additional_resources)
    // 	.await;
    //
    // -- Build Json Rpc Success Response
    // Note: Error Json response will be generated in the mw_res_map as wil other error.
    let res = call_res.map(|rpc_call_response| {
        let body_response = json!({
			"jsonrpc": "2.0",
			"id": rpc_info.id.clone(),
			"result": rpc_call_response
		});
        Json(body_response)
    });

    // -- Create and Update Axum Response
    // Note: We store data in the Axum Response extensions so that
    //       we can unpack it in the `mw_res_map` for client-side rendering.
    //       This approach centralizes error handling for the client at the `mw_res_map` module
    let res: crate::error::Result<_> = res.map_err(crate::error::Error::from);
    let mut res = res.into_response();
    // Note: Here, add the capture RpcInfo (RPC ID and method) into the Axum response to be used
    //       later in the `mw_res_map` for RequestLineLogging, and eventual JSON-RPC error serialization.
    res.extensions_mut().insert(Arc::new(rpc_info));

    res

    //StatusCode::OK.into_response()
}
