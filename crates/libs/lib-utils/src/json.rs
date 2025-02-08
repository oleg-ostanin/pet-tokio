use axum::response::Response;
use http_body_util::BodyExt;
use hyper::body::Buf;
use hyper::body::Incoming;
use serde::Deserialize;
use serde_json::Value;
use tower::Service;
use anyhow::{anyhow, Context, Result};
use tracing::error;
use tracing_subscriber::fmt::format;

pub async fn value(response: Response<Incoming>) -> Result<Value> {
    let body = response.collect().await
        .map_err(|e| {
            error!("Failed to collect: {}", e.to_string());
            anyhow!(e)
        })?
        .aggregate();
    let json_value: Value = serde_json::from_reader(body.reader())
        .map_err(|e| {
            error!("Failed to parse: {}", e.to_string());
            anyhow!(e)
        })?;
    Ok(json_value)
}

//pub(crate) fn body<T: Deserialize>(json: Value) -> T { // todo investigate why does not work
pub fn body<T: for<'a> Deserialize<'a>>(json: Value) -> Option<T> {
    serde_json::from_value::<Option<T>>(json)
        .map_err(|e| error!("Failed to parse value: {}", e.to_string()))
        .ok()?
}

// pub async fn value0(response: Response<Incoming>) -> Result<Value> {
//     let body = response.collect().await
//         .and_then(|collected| collected.aggregate())
//         .and_then(z);
//     serde_json::from_reader(body.reader())?
// }