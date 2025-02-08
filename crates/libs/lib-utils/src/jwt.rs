use std::collections::BTreeMap;

use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use jwt::VerifyWithKey;
use sha2::Sha256;

use anyhow::{Result};

pub fn token(phone: impl Into<String>, token_key: &str) -> Result<String> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(token_key.as_bytes())?;
    let mut claims = BTreeMap::new();
    claims.insert("sub", phone.into());

    Ok(claims.sign_with_key(&key)?)
}

fn verify_token(phone: impl Into<String>, token: impl Into<String>, token_key: &str) -> Result<bool> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(token_key.as_bytes())?;
    let claims: BTreeMap<String, String> = token.into().verify_with_key(&key)?;
    if let Some(sub) = claims.get("sub") {
        return Ok(sub.eq(&phone.into()))
    }
    Ok(false)
}

pub fn phone_from_token(token: String, token_key: &str) -> Option<String> {
    let key: Hmac<Sha256> = Hmac::new_from_slice(token_key.as_bytes()).ok()?;
    let claims: BTreeMap<String, String> = token.verify_with_key(&key).ok()?;
    claims.get("sub").cloned() // todo remove clone
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
        assert_eq!(TEST_TOKEN, token.expect("should be there"))
    }

    #[test]
    fn verify() {
        assert!(verify_token(TEST_SUB, TEST_TOKEN, TEST_TOKEN_KEY).expect("should be ok"))
    }

    #[test]
    fn verify_fail() {
        assert_eq!(false, verify_token("wrong_sub", TEST_TOKEN, TEST_TOKEN_KEY).expect("should be ok"))
    }
}