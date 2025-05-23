use axum::response::Response;
use http_body_util::BodyExt;
use hyper::body::Buf;
use hyper::body::Incoming;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
struct ErrorDetail {
    message: Option<String>,
    data: ErrorData,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct ErrorData {
    detail: Option<String>,
    req_uuid: Option<String>,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: ErrorDetail,
    _id: Option<String>,
}

pub async fn message_from_response(response: Response<Incoming>) -> String {
    let body = response.collect().await.unwrap().aggregate();
    let json_value: Value = serde_json::from_reader(body.reader()).unwrap();
    get_message(json_value)
}

pub async fn message_and_detail(response: Response<Incoming>) -> (String, String) {
    let body = response.collect().await.unwrap().aggregate();
    let json_value: Value = serde_json::from_reader(body.reader()).unwrap();
    (get_message(json_value.clone()), get_detail(json_value))
}

pub fn get_message(json: Value) -> String {
    let error_response: ErrorResponse = serde_json::from_value(json).unwrap();
    error_response.error.message.unwrap()
}

pub fn get_detail(json: Value) -> String {
    let error_response: ErrorResponse = serde_json::from_value(json).unwrap();
    error_response.error.data.detail.unwrap()
}