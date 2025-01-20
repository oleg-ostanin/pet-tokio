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
    async fn nils_second_att() {
        let mut ctx = TestContext::new().await;

        // when no token then forbidden
        let no_token_response = ctx.get_user_response_by_id(1).await;
        assert_eq!(no_token_response.status(), StatusCode::FORBIDDEN);
        let message = message_from_response(no_token_response).await;
        assert_eq!(message, "NO_AUTH");

        // // when new user then OK
        // let user_to_create = UserForCreate::new("2128506", "pwd", "John", "Doe");
        // let response = ctx.create_user(&user_to_create).await;
        // assert_eq!(response.status(), StatusCode::OK);
        //
        // // when existing user then 400
        // let already_exists_response = ctx.create_user(&user_to_create).await;
        // println!("error: {:?}", &already_exists_response);
        // assert_eq!(already_exists_response.status(), StatusCode::BAD_REQUEST);
        // let message = message_from_response(already_exists_response).await;
        // assert_eq!(message, "USER_EXISTS");
        //
        // // when login then OK
        // let user_body = UserForSignIn::new("2128506", "pwd");
        // let sign_in_response = ctx.sign_in_user(user_body).await;
        // assert_eq!(response.status(), StatusCode::OK);
        // let auth_token = ctx.get_auth_cookie(&sign_in_response).await;
        // assert!(auth_token.is_some());
        //
        // // todo get user
        // ctx.get_user_by_id(1).await;
        //
        // // when login with a wrong creds then 403
        // ctx.invalidate_token();
        // let user_body = UserForSignIn::new("2128506", "wrong password");
        // let sign_in_response = ctx.sign_in_user(user_body).await;
        // assert_eq!(sign_in_response.status(), StatusCode::FORBIDDEN);
        // let auth_token = ctx.get_auth_cookie(&sign_in_response).await;
        // assert!(auth_token.is_none());
        // let message = message_from_response(sign_in_response).await;
        // assert_eq!(message, "LOGIN_FAIL");
    }
}