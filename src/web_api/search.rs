use super::resource::TrackInfo;
use super::{Result, WebApiError};
use reqwest::{Client, RequestBuilder};
use serde_json::Value;
use std::fmt::Display;

const URL_SEARCH: &str = "https://api.spotify.com/v1/search";

#[derive(Debug)]
pub struct SearchQuery {
    title: String,
    artist: String,
    album: String,
}

impl SearchQuery {
    pub fn new<S1, S2, S3>(title: S1, artist: S2, album: S3) -> Self
    where
        S1: ToString,
        S2: ToString,
        S3: ToString,
    {
        Self {
            title: title.to_string(),
            artist: artist.to_string(),
            album: album.to_string(),
        }
    }
}

impl Display for SearchQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title)?;
        write!(f, " track:{}", self.title)?;
        write!(f, " artist:{}", self.artist)?;
        write!(f, " album:{}", self.album)
    }
}

#[derive(Debug)]
pub struct SearchClient {
    request: RequestBuilder,
}

impl SearchClient {
    pub fn new<D: Display>(token: D, query: SearchQuery) -> Self {
        let request = Client::new()
            .get(URL_SEARCH)
            .bearer_auth(token)
            .query(&[("q", query.to_string()), ("type", "track".to_string())]);
        Self { request }
    }

    pub async fn search(self) -> Result<Vec<TrackInfo>> {
        let resp = self.request.send().await?;
        if !resp.status().is_success() {
            let status = resp.status().as_u16() as u16;
            let body = resp.text().await?;
            return Err(WebApiError::ApiError(status, body));
        }
        let res_json = resp.text().await?;
        let res: Value =
            serde_json::from_str(&res_json).map_err(|_| WebApiError::JsonParseError(res_json))?;
        let res_iter = res
            .get("tracks")
            .ok_or(WebApiError::UnexpectedResponse)?
            .get("items")
            .ok_or(WebApiError::UnexpectedResponse)?
            .as_array()
            .ok_or(WebApiError::UnexpectedResponse)?
            .iter();

        let mut tracks = vec![];
        for track in res_iter {
            tracks.push(TrackInfo::try_from(track)?);
        }

        Ok(tracks)
    }
}
