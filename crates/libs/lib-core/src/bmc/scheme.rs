//use super::{Error, Result};
//use crate::auth_config;
//use crate::pwd::scheme::Scheme;
use super::user::ContentToHash;
use hmac::{Hmac, Mac};
use lib_utils::b64::{b64u_decode, b64u_encode};
use sha2::Sha512;
use std::env;

use crate::error::{Result, Error};


pub struct Scheme;

impl Scheme  {
	pub fn hash(&self, to_hash: &ContentToHash) -> Result<String> {
		let key_str = env::var("SERVICE_PWD_KEY").expect("TOKEN must be set.");
		let key = b64u_decode(key_str.as_str()).unwrap();
		hash(&key, to_hash)
	}

	pub(crate) fn validate(&self, to_hash: &ContentToHash, raw_pwd_ref: String) -> Result<()> {
		let raw_pwd_new = self.hash(to_hash)?;
		if raw_pwd_new.eq(&raw_pwd_ref) {
			Ok(())
		} else {
			Err(Error::CoreError)
		}
	}
}

fn hash(key: &[u8], to_hash: &ContentToHash) -> Result<String> {
	let ContentToHash { content, salt } = to_hash;

	// -- Create a HMAC-SHA-512 from key.
	let mut hmac_sha512 =
		Hmac::<Sha512>::new_from_slice(key).map_err(|_| Error::CoreError)?;

	// -- Add content.
	hmac_sha512.update(content.as_bytes());
	hmac_sha512.update(salt.as_bytes());

	// -- Finalize and b64u encode.
	let hmac_result = hmac_sha512.finalize();
	let result_bytes = hmac_result.into_bytes();

	let result = b64u_encode(result_bytes);

	Ok(result)
}

// region:    --- Tests
#[cfg(test)]
mod tests {
	pub type Result<T> = core::result::Result<T, Error>;
	pub type Error = Box<dyn std::error::Error>; // For early tests.

	use super::*;
	//use crate::auth_config;
	use uuid::Uuid;

	const SERVICE_PWD_KEY: &str =
		"CKUGFOD9_2Qf6Pn3ZFRYgPYb8ht4vKqEG9PGMXTB7497bT0367DjoaD6ydFnEVaIRda0kKeBZVCT5Hb62m2sCA";

	#[test]
	fn test_scheme_hash_into_b64u_ok() -> Result<()> {
		// -- Setup & Fixtures
		let fx_salt = Uuid::parse_str("f05e8961-d6ad-4086-9e78-a6de065e5453")?;
		let fx_key = b64u_decode(SERVICE_PWD_KEY).unwrap();
		let fx_to_hash = ContentToHash {
			content: "hello world".to_string(),
			salt: fx_salt,
		};
		let fx_res = "qO9A90161DoewhNXFwVcnAaljRIVnajvd5zsVDrySCwxpoLwVCACzaz-8Ev2ZpI8RackUTLBVqFI6H5oMe-OIg";

		// -- Exec
		let res = hash(&fx_key, &fx_to_hash)?;

		// -- Check
		assert_eq!(res, fx_res);

		Ok(())
	}
}
// endregion: --- Tests
