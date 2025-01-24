use http_body_util::BodyExt;
use hyper::body::Buf;
use tower::{Service, ServiceExt};

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use lib_dto::user::UserForCreate;
    // use lib_core::model::user::{UserForCreate, UserForSignIn};
    use crate::context::context::TestContext;
    use crate::utils::body_utils::message_from_response;

    #[tokio::test]
    async fn auth() {
        let mut ctx = TestContext::new().await;

        // when new user then OK
        let user_to_create = UserForCreate::new("2128506", "pwd", "John", "Doe");
        let response = ctx.create_user(&user_to_create).await;
        assert_eq!(response.status(), StatusCode::OK);
    }
}