use axum::response::Response;
use http_body_util::BodyExt;
use hyper::body::Buf;
use hyper::body::Incoming;
use serde::Deserialize;
use serde_json::Value;
use tower::Service;

#[derive(Deserialize)]
struct ErrorDetail {
    message: Option<String>,
    data: ErrorData,
}

#[derive(Deserialize)]
struct ErrorData {
    detail: Option<String>,
    req_uuid: Option<String>,
}

#[derive(Deserialize)]
struct ErrorResponse {
    error: ErrorDetail,
    id: Option<String>,
}

pub(crate) async fn message_from_response(response: Response<Incoming>) -> String {
    let body = response.collect().await.unwrap().aggregate();
    let json_value: Value = serde_json::from_reader(body.reader()).unwrap();
    get_message(json_value)
}

pub(crate) async fn value(response: Response<Incoming>) -> Value {
    let body = response.collect().await.unwrap().aggregate();
    let json_value: Value = serde_json::from_reader(body.reader()).unwrap();
    json_value
}

//pub(crate) fn body<T: Deserialize>(json: Value) -> T { // todo investigate why does not work
pub(crate) fn body<T: for<'a> Deserialize<'a>>(json: Value) -> T {
    serde_json::from_value::<T>(json).unwrap()
}

pub(crate) fn get_message(json: Value) -> String {
    let error_response: ErrorResponse = serde_json::from_value(json).unwrap();
    error_response.error.message.unwrap()
}