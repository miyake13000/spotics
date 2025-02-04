use super::{Result, WebPlayerError, USER_AGENT_CONTENT};
use reqwest::header::{COOKIE, USER_AGENT};
use reqwest::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};

const URL_AUTH: &str = "https://open.spotify.com/get_access_token";
const QUERY: [(&str, &str); 2] = [("reason", "transport"), ("productType", "web_player")];

#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct AuthData {
    pub clientId: String,
    pub accessToken: String,
    pub accessTokenExpirationTimestampMs: i64,
    pub isAnonymous: bool,
}

#[derive(Debug)]
pub struct AuthClient {
    request: RequestBuilder,
}

impl AuthClient {
    pub fn new<S: AsRef<str>>(sp_dc: S) -> Self {
        let cookie = format!("sp_dc={}", sp_dc.as_ref());
        let request = Client::new()
            .get(URL_AUTH)
            .query(&QUERY)
            .header(COOKIE, cookie)
            .header(USER_AGENT, USER_AGENT_CONTENT);
        Self { request }
    }

    pub async fn auth(self) -> Result<AuthData> {
        let resp = self.request.send().await?;
        if !resp.status().is_success() {
            return Err(WebPlayerError::InvalidToken);
        }
        let res_json = resp.text().await?;
        let auth_res: AuthData = serde_json::from_str(res_json.as_str())
            .map_err(|_| WebPlayerError::JsonParseError(res_json))?;
        if auth_res.isAnonymous {
            return Err(WebPlayerError::InvalidToken);
        }
        Ok(auth_res)
    }
}
