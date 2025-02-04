mod lrc;
mod token;
mod token_store;
mod web_api;
mod web_player;

pub use lrc::Lrc;
pub use web_api::{resource::TrackInfo, search::SearchQuery};
pub type Result<T> = std::result::Result<T, Error>;

const TOKEN_STORE_FILE: &str = "tokens.json";
const API_CACHE_STORE_FILE: &str = "api_cache.json";
const PLAYER_CACHE_STORE_FILE: &str = "player_cache.json";

use chrono::{DateTime, TimeDelta, Utc};
use futures::try_join;
use std::path::PathBuf;
use thiserror::Error;
use token::{Cache, Token};
use token_store::{TokenStore, TokenStoreError};
use web_api::auth::AuthData as ApiAuthData;
use web_api::WebApiError;
use web_player::auth::AuthData as PlayerAuthData;
use web_player::resource::Lyrics;
use web_player::WebPlayerError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to access token store or cache store")]
    TokenStoreError(#[from] TokenStoreError),

    #[error("Failed to access Web API")]
    WebApiError(#[from] WebApiError),

    #[error("Failed to access Web Player Interface")]
    WebPlayerError(#[from] WebPlayerError),

    #[error("Not found token")]
    TokenNotFound,

    #[error("Unknown error")]
    Unknown,
}

#[derive(Debug)]
pub struct Client {
    api_client: web_api::Client,
    player_client: web_player::Client,
}

impl Client {
    pub async fn search(&self, query: SearchQuery) -> Result<Vec<TrackInfo>> {
        let tracks = self.api_client.search(query).await?;
        Ok(tracks)
    }

    pub async fn fetch_lyrics(&self, track: &TrackInfo) -> Result<Lrc> {
        let track_id = track.id.as_str();
        let lyrics = self.player_client.fetch_lyrics(track_id).await?;
        let lrc = Lrc::new(lyrics);
        Ok(lrc)
    }
}

#[derive(Debug)]
pub struct ClientBuilder {
    token_store: PathBuf,
    use_cache: bool,
    api_cache_store: Option<PathBuf>,
    player_cache_store: Option<PathBuf>,
}

impl ClientBuilder {
    pub fn new<P: Into<PathBuf>>(token_store: P) -> Self {
        let token_store_dir = token_store.into();
        let token_store = token_store_dir.join(TOKEN_STORE_FILE);

        Self {
            token_store,
            use_cache: false,
            api_cache_store: None,
            player_cache_store: None,
        }
    }

    pub fn use_cache<P: Into<PathBuf>>(mut self, cache_store: P) -> Self {
        let cache_store_dir = cache_store.into();
        let api_cache_store = cache_store_dir.join(API_CACHE_STORE_FILE);
        let player_cache_store = cache_store_dir.join(PLAYER_CACHE_STORE_FILE);

        self.use_cache = true;
        self.api_cache_store = Some(api_cache_store);
        self.player_cache_store = Some(player_cache_store);

        self
    }

    pub async fn build(self) -> Result<Client> {
        let token_store = TokenStore::<Token>::new(self.token_store);
        let api_cache_store = TokenStore::<Cache>::new(self.api_cache_store.unwrap());
        let player_cache_store = TokenStore::<Cache>::new(self.player_cache_store.unwrap());
        let token = token_store.load().await?.ok_or(Error::TokenNotFound)?;

        if self.use_cache {
            let (api_cache, player_cache) =
                try_join!(api_cache_store.load(), player_cache_store.load())?;

            let (api_client, player_client) = try_join!(
                auth_and_update_api_token(api_cache, api_cache_store, &token),
                auth_and_update_player_token(player_cache, player_cache_store, &token)
            )?;

            return Ok(Client {
                api_client,
                player_client,
            });
        }

        let (api_client, player_client) =
            try_join!(auth_api_token(&token), auth_player_token(&token))?;

        Ok(Client {
            api_client,
            player_client,
        })
    }
}

#[allow(clippy::unnecessary_unwrap)]
async fn auth_and_update_api_token(
    cache: Option<Cache>,
    cache_store: TokenStore<Cache>,
    token: &Token,
) -> Result<web_api::Client> {
    if cache.is_none() || cache.as_ref().unwrap().is_expired() {
        let client = auth_api_token(token).await?;
        let auth_data = client.token();
        save_api_cache(auth_data, cache_store).await?;

        Ok(client)
    } else {
        Ok(web_api::Client::from_token(&cache.unwrap().access_token))
    }
}

#[allow(clippy::unnecessary_unwrap)]
async fn auth_and_update_player_token(
    cache: Option<Cache>,
    cache_store: TokenStore<Cache>,
    token: &Token,
) -> Result<web_player::Client> {
    if cache.is_none() || cache.as_ref().unwrap().is_expired() {
        let client = auth_player_token(token).await?;
        let auth_data = client.token();
        save_player_cache(auth_data, cache_store).await?;

        Ok(client)
    } else {
        Ok(web_player::Client::from_token(&cache.unwrap().access_token))
    }
}

async fn auth_api_token(token_data: &Token) -> Result<web_api::Client> {
    let client_id = token_data.client_id.as_str();
    let client_secret = token_data.client_secret.as_str();
    let client = web_api::Client::auth(client_id, client_secret).await?;

    Ok(client)
}

async fn auth_player_token(token_data: &Token) -> Result<web_player::Client> {
    let sp_dc = token_data.sp_dc.as_str();
    let client = web_player::Client::auth(sp_dc).await?;

    Ok(client)
}

async fn save_api_cache(auth_data: &ApiAuthData, token_store: TokenStore<Cache>) -> Result<()> {
    let access_token = auth_data.access_token.clone();
    let expires_at = Utc::now() + TimeDelta::seconds(auth_data.expires_in);

    let cache = Cache {
        access_token,
        expires_at,
    };
    token_store.save(cache).await?;

    Ok(())
}

async fn save_player_cache(
    auth_data: &PlayerAuthData,
    token_store: TokenStore<Cache>,
) -> Result<()> {
    let access_token = auth_data.accessToken.clone();
    let expires_at = DateTime::from_timestamp_millis(auth_data.accessTokenExpirationTimestampMs)
        .ok_or(Error::Unknown)?;

    let cache = Cache {
        access_token,
        expires_at,
    };
    token_store.save(cache).await?;

    Ok(())
}
