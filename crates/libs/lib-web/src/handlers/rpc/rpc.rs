use serde_json::{json, Value};

use crate::error::Result;

async fn get() -> Result<Value> {
	Ok(json!({
			"phone": "212",
			"password": "result_password",
	}))
}