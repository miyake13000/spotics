use anyhow::{bail, Context, Result};
use chrono::{DateTime, TimeDelta, Utc};
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Select};
use id3::{Tag, TagLike};
use log::{debug, info, trace};
use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
};
use tokio::{fs, process::Command};

const PATH_TOKENS_STORE: &str = ".config/spotics/tokens.json";
const PATH_CACHE: &str = ".cache/spotics/tokens.json";
const URL_ACCESS_TOKEN: &str = "https://accounts.spotify.com/api/token";
const URL_SEARCH: &str = "https://api.spotify.com/v1/search";

#[derive(Debug, Parser)]
struct Args {
    /// Target file
    ///
    /// This file should be audio file that has ID3 tag
    file: String,

    /// Automatically download lyrics
    ///
    /// If this flag is set, download lyrics of the track that has same title, artist, and album
    #[arg(short = 'a', long)]
    auto: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenData {
    id: String,
    secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthResult {
    access_token: String,
    token_type: String,
    expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthCache {
    access_token: String,
    expires_at: DateTime<Utc>,
}

#[derive(Debug)]
struct TrackInfo {
    title: String,
    artist: String,
    album: String,
    url: String,
}

impl From<&Value> for TrackInfo {
    fn from(json: &Value) -> Self {
        // TODO: Handle error
        let title = json.get("name").unwrap().as_str().unwrap().to_string();
        let artist = json
            .get("artists")
            .unwrap()
            .get(0)
            .unwrap()
            .get("name")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let album = json
            .get("album")
            .unwrap()
            .get("name")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let url = json
            .get("external_urls")
            .unwrap()
            .get("spotify")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        Self {
            title,
            artist,
            album,
            url,
        }
    }
}

impl From<Tag> for TrackInfo {
    fn from(tag: Tag) -> Self {
        let title = tag.title().unwrap().to_string();
        let artist = tag.artist().unwrap().to_string();
        let album = tag.album().unwrap().to_string();
        let url = "This is not spotify data".to_string();
        Self {
            title,
            artist,
            album,
            url,
        }
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

impl PartialEq for TrackInfo {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title && self.artist == other.artist && self.album == other.album
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse arguments
    let args = Args::parse();
    let file = PathBuf::from_str(&args.file)
        .context("Invalid file name")?
        .canonicalize()
        .context("Invalid file path")?;
    debug!("Args: {:?}", args);

    // Initialize logger
    env_logger::init();

    // Get Access Token from cache
    let homedir = PathBuf::from(std::env::var("HOME").context("Env 'HOME' is not set")?);
    let cache_file = homedir.join(PATH_CACHE);
    let cache = if cache_file.is_file() {
        Some(
            get_cache(&cache_file)
                .await
                .context("Failed to read cache")?,
        )
    } else {
        None
    };

    // Authenticate client to create new access token if cache is not found or expired
    let token = if cache.is_none() || cache.as_ref().unwrap().expires_at < Utc::now() {
        info!("Cache is not found or expired");

        // Get Spotify API Tokens
        let token_store = homedir.join(PATH_TOKENS_STORE);
        let token_data = get_token(&token_store)
            .await
            .context("Failed to read token")?;
        debug!("Token data: {:?}", token_data);

        // Authenticate
        let auth_res = authenticate_client(token_data).await?;
        debug!("Auth result: {:?}", auth_res);

        // Save cache
        let expired_at = Utc::now() + TimeDelta::seconds(auth_res.expires_in as i64);
        let cache_data = AuthCache {
            access_token: auth_res.access_token.clone(),
            expires_at: expired_at,
        };
        debug!("Cache data: {:?}", cache_data);
        fs::create_dir_all(cache_file.parent().unwrap())
            .await
            .context("Failed to create cache directory")?;
        save_cache(cache_file, &cache_data)
            .await
            .context("Failed to save cache")?;
        info!("Cache saved");

        auth_res.access_token
    } else {
        cache.unwrap().access_token
    };

    println!("Start to search lyrics for {}", file.display());

    // Get tags from specified file
    let tag = Tag::read_from_path(&file).context("Failed to read tag")?;
    let target_track = TrackInfo::from(tag);
    debug!("Tag: {:?}", target_track);

    // Create search query
    let search_query = format_search_query(
        &target_track.title,
        &target_track.artist,
        &target_track.album,
    );
    debug!("Search query: {}", search_query);

    // Search track info
    info!("Track info search started");
    let search_res = search_track(token, &search_query).await?;
    let tracks_json = search_res
        .get("tracks")
        .unwrap()
        .get("items")
        .unwrap()
        .as_array()
        .unwrap();
    let tracks: Vec<TrackInfo> = tracks_json.iter().map(TrackInfo::from).collect();
    debug!("Tracks: {:?}", tracks);

    // Select track
    let track = if args.auto {
        let track = select_track_with_identity(&target_track, &tracks);
        if track.is_none() {
            println!("Same track not found");
            return Ok(());
        }
        track.unwrap()
    } else {
        let track = select_track_with_prompt(&target_track, &tracks);
        if track.is_none() {
            println!("Aborted");
            return Ok(());
        }
        track.unwrap()
    };
    debug!("Selected track: {:?}", track);

    // Start downloading lyrics
    info!("Start downloading lyrics");
    let cmd_res = Command::new("syrics")
        .arg(&track.url)
        .arg("--directory")
        .arg(file.parent().unwrap().to_str().unwrap())
        .spawn()
        .unwrap()
        .wait_with_output()
        .await
        .context("Failed to download lyrics")?;
    info!("Downloading lyrics done");

    // Check download result
    let output = String::from_utf8(cmd_res.stdout).context("Failed to read stdout")?;
    trace!("Lyrics download output: {}", output);
    if cmd_res.status.success() {
        println!("Complete to download lyrics");
        Ok(())
    } else if cmd_res.status.code().unwrap() == 3 {
        println!("Lyrics not found");
        Ok(())
    } else {
        bail!("Failed to download lyrics")
    }
}

async fn get_cache<P: AsRef<Path>>(cache_file: P) -> Result<AuthCache> {
    let cache_data = fs::read_to_string(cache_file.as_ref())
        .await
        .with_context(|| format!("Cannot read cache ({})", cache_file.as_ref().display()))?;
    let cache_data: AuthCache =
        serde_json::from_str(cache_data.as_str()).context("Failed to read JSON")?;
    trace!(
        "Get cache from {}, data: {:?}",
        cache_file.as_ref().display(),
        cache_data
    );
    Ok(cache_data)
}

async fn get_token<P: AsRef<Path>>(token_file: P) -> Result<TokenData> {
    let token_data = fs::read_to_string(token_file.as_ref())
        .await
        .with_context(|| format!("Cannot read token file ({})", token_file.as_ref().display()))?;
    let token_data: TokenData =
        serde_json::from_str(token_data.as_str()).context("Failed to read JSON")?;
    trace!(
        "Get token data from {}, data: {:?}",
        token_file.as_ref().display(),
        token_data
    );
    Ok(token_data)
}

async fn authenticate_client(token_data: TokenData) -> Result<AuthResult> {
    let req = Client::new()
        .post(URL_ACCESS_TOKEN)
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .form(&[
            ("grant_type", "client_credentials"),
            ("client_id", &token_data.id),
            ("client_secret", &token_data.secret),
        ]);
    trace!("Authenticate request: {:?}", req);
    let res = req
        .send()
        .await
        .context("Failed to send authenticate request")?
        .text()
        .await
        .context("Failed to read authenticate response")?;
    trace!("Authenticate response: {:?}", res);
    let auth_res: AuthResult = serde_json::from_str(res.as_str())
        .with_context(|| format!("Failed to decode JSON: {}", res))?;
    trace!("Authenticate response json: {:?}", auth_res);

    Ok(auth_res)
}

async fn save_cache<P: AsRef<Path>>(cache_file: P, cache_data: &AuthCache) -> Result<()> {
    let cache_data = serde_json::to_string(&cache_data).context("Failed to encode JSON")?;
    fs::write(cache_file.as_ref(), &cache_data)
        .await
        .with_context(|| format!("Failed to write cache ({})", cache_file.as_ref().display()))?;
    Ok(())
}

fn format_search_query<S1, S2, S3>(title: S1, artist: S2, album: S3) -> String
where
    S1: ToString,
    S2: AsRef<str>,
    S3: AsRef<str>,
{
    let mut search_query = title.to_string();
    search_query.push_str(&format!("%20artist:{}", artist.as_ref()));
    search_query.push_str(&format!("%20album:{}", album.as_ref()));
    search_query
}

async fn search_track<S1, S2>(token: S1, query: S2) -> Result<Value>
where
    S1: AsRef<str>,
    S2: AsRef<str>,
{
    let req = Client::new()
        .get(URL_SEARCH)
        .bearer_auth(token.as_ref())
        .query(&[("q", query.as_ref()), ("type", "track")]);
    trace!("Search request: {:?}", req);
    let res = req
        .send()
        .await
        .context("Failed to search Track info")?
        .text()
        .await
        .context("Failed to read search result")?;
    trace!("Search response: {}", res);
    let res_json: Value =
        serde_json::from_str(&res).with_context(|| format!("Failed to decode JSON: {}", res))?;

    Ok(res_json)
}

fn select_track_with_prompt<'a>(
    target_track: &TrackInfo,
    tracks: &'a [TrackInfo],
) -> Option<&'a TrackInfo> {
    let mut prompt = format!("Specified track: {}\n", target_track);
    prompt.push_str("Select track to download lyrics (press 'q' to cancel)");
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(tracks)
        .interact_opt()
        .unwrap();
    if let Some(index) = selection {
        Some(tracks.get(index).unwrap())
    } else {
        None
    }
}

fn select_track_with_identity<'a>(
    target_track: &TrackInfo,
    tracks: &'a [TrackInfo],
) -> Option<&'a TrackInfo> {
    tracks.iter().find(|track| *track == target_track)
}
