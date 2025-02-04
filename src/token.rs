use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    pub client_id: String,
    pub client_secret: String,
    pub sp_dc: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cache {
    pub access_token: String,
    pub expires_at: DateTime<Utc>,
}

impl Cache {
    pub fn is_expired(&self) -> bool {
        self.expires_at < Utc::now()
    }
}
