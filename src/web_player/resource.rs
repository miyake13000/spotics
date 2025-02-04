use super::{Result, WebPlayerError};
use serde_json::Value;

#[derive(Debug)]
pub struct Lyrics {
    pub lines: Vec<LyricLine>,
}

#[derive(Debug)]
pub struct LyricLine {
    pub words: String,
    pub start_time: u64,
}

impl TryFrom<&Value> for Lyrics {
    type Error = WebPlayerError;

    fn try_from(json: &Value) -> Result<Self> {
        let mut lines = Vec::new();
        for line in json.as_array().ok_or(WebPlayerError::UnexpectedResponse)? {
            let words = line
                .get("words")
                .ok_or(WebPlayerError::UnexpectedResponse)?
                .as_str()
                .ok_or(WebPlayerError::UnexpectedResponse)?
                .to_string();
            let start_time = line
                .get("startTimeMs")
                .ok_or(WebPlayerError::UnexpectedResponse)?
                .as_str()
                .ok_or(WebPlayerError::UnexpectedResponse)?
                .to_string();
            let start_time = start_time
                .parse::<u64>()
                .map_err(|_| WebPlayerError::UnexpectedResponse)?;
            lines.push(LyricLine { words, start_time });
        }
        Ok(Self { lines })
    }
}
