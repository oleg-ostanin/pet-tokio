use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct TempResult {
    pub phone: String,
    pub password: String,
}