use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;

#[derive(Debug, Error)]
pub enum TokenStoreError {
    #[error("Not found token store")]
    NotFound,

    #[error("Failed to read token store: {0}")]
    FileAccessError(#[from] std::io::Error),

    #[error("Token file has invalid format")]
    InvalidToken,

    #[error("Unknown error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, TokenStoreError>;

#[derive(Debug)]
pub struct TokenStore<T> {
    location: PathBuf,
    _data: PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned> TokenStore<T> {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        let location = path.as_ref().to_path_buf();

        Self {
            location,
            _data: PhantomData,
        }
    }

    pub fn exists(&self) -> bool {
        self.location.exists()
    }

    pub async fn load(&self) -> Result<Option<T>> {
        if !self.exists() {
            return Ok(None);
        }
        let token_str = fs::read_to_string(&self.location).await?;
        let token = serde_json::from_str(&token_str).map_err(|_| TokenStoreError::InvalidToken)?;

        Ok(token)
    }

    pub async fn save(&self, token: T) -> Result<()> {
        let token_str = serde_json::to_string(&token).map_err(|_| TokenStoreError::Unknown)?;
        fs::create_dir_all(self.location.parent().unwrap()).await?;
        fs::write(&self.location, token_str.as_bytes()).await?;

        Ok(())
    }
}
