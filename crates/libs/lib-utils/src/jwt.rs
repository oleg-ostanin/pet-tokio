use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use sha2::Sha256;
use std::collections::BTreeMap;
use jwt::VerifyWithKey;

pub fn token(identity: impl Into<String>, token_key: &str) -> String {
    let key: Hmac<Sha256> = Hmac::new_from_slice(token_key.as_bytes()).unwrap();
    let mut claims = BTreeMap::new();
    claims.insert("sub", identity.into());

    claims.sign_with_key(&key).unwrap()
}

fn verify_token(identity: impl Into<String>, token: impl Into<String>, token_key: &str) -> bool {
    let key: Hmac<Sha256> = Hmac::new_from_slice(token_key.as_bytes()).unwrap();
    let claims: BTreeMap<String, String> = token.into().verify_with_key(&key).unwrap();
    if let Some(sub) = claims.get("sub") {
        return sub.eq(&identity.into())
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SUB: &str = "2128506";
    const TEST_TOKEN_KEY: &str = "SwF2ONNd6oTbRfKJwAsDusThvq1InbVv";
    const TEST_TOKEN: &str = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIyMTI4NTA2In0.5spPSdmwj3LSIOcTc3um93yP1CYW1fB7Ieslqw7vhhU";

    #[test]
    fn create() {
        let token = token(TEST_SUB, TEST_TOKEN_KEY);
        assert_eq!(TEST_TOKEN, token)
    }

    #[test]
    fn verify() {
        assert!(verify_token(TEST_SUB, TEST_TOKEN, TEST_TOKEN_KEY))
    }

    #[test]
    fn verify_fail() {
        assert_eq!(false, verify_token("wrong_sub", TEST_TOKEN, TEST_TOKEN_KEY))
    }
}