use http_body_util::BodyExt;
use hyper::body::Buf;
use tower::{Service, ServiceExt};

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;

    use lib_dto::user::{AuthCode, UserForCreate, UserForSignIn};
    use lib_utils::json::{body, value};

    use crate::context::context::{ServiceType, TestContext};

    #[tokio::test]
    async fn auth() {
        let mut ctx = TestContext::new(ServiceType::Auth).await;

        // when new user then OK
        let user_to_create = UserForCreate::new("2128506", "pwd", "John", "Doe");
        let response = ctx.create_user(&user_to_create).await;
        assert_eq!(response.status(), StatusCode::OK);

        let user_to_sigh_in = UserForSignIn::new("2128506", "pwd",);
        let response = ctx.sign_in_user(user_to_sigh_in).await;
        assert_eq!(response.status(), StatusCode::OK);
        let value = value(response).await;
        let auth_code = body::<AuthCode>(value);

        let user_to_sigh_in = AuthCode::new("2128506", auth_code.auth_code,);
        let response = ctx.check_code(user_to_sigh_in).await;
        assert_eq!(response.status(), StatusCode::OK);
    }
}