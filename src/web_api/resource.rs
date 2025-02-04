use super::{Result, WebApiError};
use serde_json::Value;
use std::fmt::Display;

#[derive(Debug)]
pub struct TrackInfo {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
}

impl TrackInfo {
    pub fn new(id: String, title: String, artist: String, album: String) -> Self {
        Self {
            id,
            title,
            artist,
            album,
        }
    }
}

impl TryFrom<&Value> for TrackInfo {
    type Error = WebApiError;

    fn try_from(json: &Value) -> Result<Self> {
        let id = json
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or(WebApiError::UnexpectedResponse)?
            .to_string();
        let title = json
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or(WebApiError::UnexpectedResponse)?
            .to_string();
        let artist = json
            .get("artists")
            .and_then(|v| v.get(0))
            .and_then(|v| v.get("name"))
            .and_then(|v| v.as_str())
            .ok_or(WebApiError::UnexpectedResponse)?
            .to_string();
        let album = json
            .get("album")
            .and_then(|v| v.get("name"))
            .and_then(|v| v.as_str())
            .ok_or(WebApiError::UnexpectedResponse)?
            .to_string();

        Ok(Self::new(id, title, artist, album))
    }
}

impl Display for TrackInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Title: \"{}\",  Artist: \"{}\",  Album: \"{}\"",
            self.title, self.artist, self.album
        )
    }
}
