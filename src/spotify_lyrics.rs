use super::{Error, Result};
use serde_json::Value;

#[derive(Debug)]
pub struct SpotifyLyric {
    pub lines: Vec<LyricLine>,
}

#[derive(Debug)]
pub struct LyricLine {
    pub words: String,
    pub start_time: u64,
}

impl TryFrom<&Value> for SpotifyLyric {
    type Error = Error;

    fn try_from(json: &Value) -> Result<Self> {
        let lyric_lines = json
            .get("lyrics")
            .ok_or(Error::InvalidJSON)?
            .get("lines")
            .ok_or(Error::InvalidJSON)?;
        let mut lines = Vec::new();
        for line in lyric_lines.as_array().ok_or(Error::InvalidJSON)? {
            let words = line
                .get("words")
                .ok_or(Error::InvalidJSON)?
                .as_str()
                .ok_or(Error::InvalidJSON)?
                .to_string();
            let start_time = line
                .get("startTimeMs")
                .ok_or(Error::InvalidJSON)?
                .as_str()
                .ok_or(Error::InvalidJSON)?
                .to_string();
            let start_time = start_time.parse::<u64>().map_err(|_| Error::InvalidJSON)?;
            lines.push(LyricLine { words, start_time });
        }
        Ok(Self { lines })
    }
}
