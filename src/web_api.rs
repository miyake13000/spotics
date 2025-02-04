pub mod auth;
pub mod resource;
pub mod search;

pub type Result<T> = std::result::Result<T, WebApiError>;

use auth::{AuthClient, AuthData};
use resource::TrackInfo;
use search::{SearchClient, SearchQuery};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WebApiError {
    #[error("Failed to access Web API")]
    HttpError(#[from] reqwest::Error),

    #[error("'client_id' or 'client_secret' may be invalid")]
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
    pub async fn auth<S1, S2>(client_id: S1, client_secret: S2) -> Result<Self>
    where
        S1: AsRef<str>,
        S2: AsRef<str>,
    {
        let auth_client = AuthClient::new(client_id, client_secret);
        let auth_data = auth_client.auth().await?;
        Ok(Self { auth_data })
    }

    pub fn from_token<S: ToString>(access_token: S) -> Self {
        Self {
            auth_data: AuthData {
                access_token: access_token.to_string(),
                token_type: "".to_string(),
                expires_in: 0,
            },
        }
    }

    pub fn token(&self) -> &AuthData {
        &self.auth_data
    }

    pub async fn search(&self, query: SearchQuery) -> Result<Vec<TrackInfo>> {
        let access_token = self.auth_data.access_token.clone();
        let client = SearchClient::new(access_token, query);
        client.search().await
    }
}
