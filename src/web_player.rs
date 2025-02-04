pub mod auth;
pub mod lyric;
pub mod resource;

pub type Result<T> = std::result::Result<T, WebPlayerError>;

const USER_AGENT_CONTENT: &str = "curl/8.5.0";

use auth::{AuthClient, AuthData};
use lyric::LyricsClient;
use resource::Lyrics;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WebPlayerError {
    #[error("Failed to access Web Player")]
    HttpError(#[from] reqwest::Error),

    #[error("'sp_dc' may be invalid")]
    InvalidToken,

    #[error("API return status: {0} with: {1}")]
    ApiError(u16, String),

    #[error("Failed to parse json: {0}")]
    JsonParseError(String),

    #[error("Unexpected response")]
    UnexpectedResponse,

    #[error("Unknown error")]
    Unknown,
}

#[derive(Debug)]
pub struct Client {
    auth_data: AuthData,
}

impl Client {
    pub async fn auth<S: AsRef<str>>(sp_dc: S) -> Result<Self> {
        let auth_client = AuthClient::new(sp_dc);
        let auth_data = auth_client.auth().await?;
        Ok(Self { auth_data })
    }

    pub fn from_token<S: ToString>(access_token: S) -> Self {
        Self {
            auth_data: AuthData {
                clientId: "".to_string(),
                accessToken: access_token.to_string(),
                accessTokenExpirationTimestampMs: 0,
                isAnonymous: false,
            },
        }
    }

    pub fn token(&self) -> &AuthData {
        &self.auth_data
    }

    pub async fn fetch_lyrics<S: AsRef<str>>(&self, track_id: S) -> Result<Lyrics> {
        let access_token = self.auth_data.accessToken.clone();
        let track_id = track_id.as_ref();
        let client = LyricsClient::new(access_token, track_id);
        client.fetch().await
    }
}
