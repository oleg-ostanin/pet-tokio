use base64::engine::{Engine, general_purpose};
use thiserror::Error;

pub fn b64u_encode(content: impl AsRef<[u8]>) -> String {
	general_purpose::URL_SAFE_NO_PAD.encode(content)
}

pub fn b64u_decode(b64u: &str) -> Result<Vec<u8>> {
	general_purpose::URL_SAFE_NO_PAD
		.decode(b64u)
		.map_err(|_| Error::FailToB64uDecode)
}

pub fn b64u_decode_to_string(b64u: &str) -> Result<String> {
	b64u_decode(b64u)
		.ok()
		.and_then(|r| String::from_utf8(r).ok())
		.ok_or(Error::FailToB64uDecode)
}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
	#[error("Failed to decode B64u")]
	FailToB64uDecode,
}
