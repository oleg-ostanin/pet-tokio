use tokio::sync::OnceCell;
use serde_json::json;
use tracing::info;
use lib_dto::book::BookList;
use lib_dto::order::{OrderContent, OrderId, OrderItem, OrderStored};
use lib_dto::user::{AuthCode, UserExists, UserForCreate, UserForSignIn};
use lib_utils::json::{body, value};
use lib_utils::rpc::request;
use crate::message_from_response;
use crate::requests::user_context::UserContext;
use crate::utils::file::from_file;

static BOOKS_INITIALIZED: OnceCell<()> = OnceCell::const_new();

pub(crate) async fn start_load() {

}

pub(crate) async fn start_user(idx: usize) -> UserContext {
    let phone = format!("{}", 2128500 + idx);
    let mut user_ctx = UserContext::new(idx, phone.clone()).await;
    let user_to_create = UserForCreate::new(phone.clone(), phone.clone(), "John", "Doe");
    let check_response = user_ctx.post("/check-if-exists", json!(&user_to_create)).await;

    info!("check_response: {:?}", &check_response);
    let check_user_value = value(check_response).await.expect("must be ok");
    info!("check_user_value: {:?}", &check_user_value);
    let user_exists: UserExists = body(check_user_value).expect("must be ok");

    if !user_exists.exists {
        user_ctx.post("/sign-up", json!(user_to_create)).await;
    }
    let user_to_sign_in = UserForSignIn::new(phone.clone(), phone.clone());
    let sign_in_response = user_ctx.post("/sign-in", json!(user_to_sign_in)).await;
    let auth_code = message_from_response(sign_in_response).await;

    info!("auth_code: {:?}", &auth_code);

    let auth_code = AuthCode::new(phone, auth_code);
    user_ctx.post("/login", json!(auth_code)).await;

    assert!(user_ctx.auth_token().is_some());

    // ensures only one user adds books
    BOOKS_INITIALIZED.get_or_init(|| async {
        info!("Initializing books");
        let book_list: BookList = from_file("books_refactored.json");
        let request = request("add_books", Some(book_list));
        user_ctx.post("/api/rpc", request).await;
    }).await;

    user_ctx
}