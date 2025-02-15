use axum::response::Response;
use http_body_util::BodyExt;
use hyper::body::Buf;
use hyper::body::Incoming;
use serde::Deserialize;
use serde_json::Value;
use tower::Service;
use anyhow::{anyhow, bail, Context, Result};
use tracing::{error, info};
use tracing_subscriber::fmt::format;

pub async fn value(response: Response<Incoming>) -> Result<Value> {
    let body = response.collect().await
        .map_err(|e| {
            error!("Failed to collect bytes from response: {}", e.to_string());
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

pub fn body<T: for<'a> Deserialize<'a>>(json: Value) -> Result<T> {
    serde_json::from_value::<T>(json)
        .map_err(|e| {
            error!("Failed to parse value: {}", e.to_string());
            anyhow!(e)
        })
}

pub async fn result<T: for<'a> Deserialize<'a>>(response: Response<Incoming>) -> Result<T> {
    let result = value(response).await
        .map_err(|e| {
            error!("Failed to get value from response: {}", e.to_string());
            anyhow!(e)
        })?
        .get("result")
        .context("no result")?
        .to_owned();

    body(result)
}