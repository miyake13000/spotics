mod lrc;
mod spotify_lyrics;

pub type Result<T> = std::result::Result<T, Error>;

pub use lrc::Lrc;
pub use spotify_lyrics::SpotifyLyric;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid Lyric JSON format")]
    InvalidJSON,

    #[error("Unknown error")]
    Unknown,
}
