use super::{Lyrics, Result, WebPlayerError, USER_AGENT_CONTENT};
use reqwest::{header::USER_AGENT, Client, RequestBuilder};
use serde_json::Value;
use std::fmt::Display;

const URL_LYRICS: &str = "https://spclient.wg.spotify.com/color-lyrics/v2/track";
const QUERY: [(&str, &str); 3] = [
    ("format", "json"),
    ("vocalRemoval", "false"),
    ("market", "from_token"),
];

#[derive(Debug)]
pub struct LyricsClient {
    request: RequestBuilder,
}

impl LyricsClient {
    pub fn new<S: AsRef<str>, D: Display>(token: D, track_id: S) -> Self {
        let url = format!("{}/{}", URL_LYRICS, track_id.as_ref());
        let request = Client::new()
            .get(url)
            .query(&QUERY)
            .bearer_auth(token)
            .header(USER_AGENT, USER_AGENT_CONTENT)
            .header("app-platform", "WebPlayer");
        Self { request }
    }

    pub async fn fetch(self) -> Result<Lyrics> {
        let resp = self.request.send().await?;
        if !resp.status().is_success() {
            let status = resp.status().as_u16() as u16;
            let body = resp.text().await?;
            return Err(WebPlayerError::ApiError(status, body));
        }
        let res_json = resp.text().await?;
        let res: Value = serde_json::from_str(&res_json)
            .map_err(|_| WebPlayerError::JsonParseError(res_json))?;
        let lines = res
            .get("lyrics")
            .and_then(|v| v.get("lines"))
            .ok_or(WebPlayerError::UnexpectedResponse)?;
        let lyrics = Lyrics::try_from(lines)?;

        Ok(lyrics)
    }
}
