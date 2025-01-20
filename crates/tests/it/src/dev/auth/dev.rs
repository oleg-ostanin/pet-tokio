use http_body_util::BodyExt;
use hyper::body::Buf;
use tower::{Service, ServiceExt};

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    // use lib_core::model::user::{UserForCreate, UserForSignIn};
    use crate::context::context::TestContext;
    use crate::utils::body_utils::message_from_response;

    #[tokio::test]
    async fn web() {
        let mut ctx = TestContext::new().await;

        // when no token then forbidden
        let no_token_response = ctx.get_user_response_by_id(1).await;
        assert_eq!(no_token_response.status(), StatusCode::FORBIDDEN);
        let message = message_from_response(no_token_response).await;
        assert_eq!(message, "NO_AUTH");
    }
}