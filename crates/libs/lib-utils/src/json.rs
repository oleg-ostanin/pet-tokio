use axum::response::Response;
use http_body_util::BodyExt;
use hyper::body::Buf;
use hyper::body::Incoming;
use serde::Deserialize;
use serde_json::Value;
use tower::Service;

pub async fn value(response: Response<Incoming>) -> Value {
    let body = response.collect().await.unwrap().aggregate();
    let json_value: Value = serde_json::from_reader(body.reader()).unwrap();
    json_value
}

//pub(crate) fn body<T: Deserialize>(json: Value) -> T { // todo investigate why does not work
pub fn body<T: for<'a> Deserialize<'a>>(json: Value) -> T {
    serde_json::from_value::<T>(json).unwrap()
}