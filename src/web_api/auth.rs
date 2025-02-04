use super::{Result, WebApiError};
use reqwest::header::CONTENT_TYPE;
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};

const URL_AUTH: &str = "https://accounts.spotify.com/api/token";

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthData {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug)]
pub struct AuthClient {
    request: RequestBuilder,
}

impl AuthClient {
    pub fn new<S1, S2>(client_id: S1, client_secret: S2) -> Self
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
    {
        let request = Client::new()
            .post(URL_AUTH)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", client_id.as_ref()),
                ("client_secret", client_secret.as_ref()),
            ]);
        Self { request }
    }

    pub async fn auth(self) -> Result<AuthData> {
        let res = self.request.send().await?;
        if !res.status().is_success() {
            return Err(WebApiError::InvalidToken);
        }
        let auth_res_json = res.text().await?;
        let auth_res: AuthData = serde_json::from_str(auth_res_json.as_str())
            .map_err(|_| WebApiError::JsonParseError(auth_res_json))?;
        Ok(auth_res)
    }
}
