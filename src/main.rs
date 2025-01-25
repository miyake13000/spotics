use anyhow::{bail, Context, Result};
use clap::Parser;
use id3::{Tag, TagLike};
use log::{debug, info, trace};
use reqwest::header::CONTENT_TYPE;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{path::PathBuf, str::FromStr};
use tokio::{fs, process::Command};

const PATH_TOKENS_STORE: &str = ".config/spotics/tokens.json";
const URL_ACCESS_TOKEN: &str = "https://accounts.spotify.com/api/token";
const URL_SEARCH: &str = "https://api.spotify.com/v1/search";

#[derive(Debug, Parser)]
struct Args {
    /// Target file
    file: String,
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
    expires_in: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse arguments
    let args = Args::parse();
    let file = PathBuf::from_str(&args.file)
        .context("Invalid file name")?
        .canonicalize()
        .context("Invalid file path")?;

    // Initialize logger
    env_logger::init();

    // Get Spotify API Tokens
    let homedir = std::env::var("HOME").context("Env 'HOME' is not set")?;
    let token_store = format!("{}/{}", homedir, PATH_TOKENS_STORE);
    let token_data = fs::read_to_string(&token_store)
        .await
        .with_context(|| format!("Cannot read token file ({})", &token_store))?;
    let token_data: TokenData =
        serde_json::from_str(token_data.as_str()).context("Failed to read JSON")?;
    trace!(
        "Get token data from {}, data: {:?}",
        &token_store,
        token_data
    );

    // Authenticate
    info!("Start authentication");
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
    let token = auth_res.access_token;
    info!("Authentication completed");

    println!("Initialization done");
    println!("Start processing {}", file.display());

    // Get tags from specified file
    let tag = Tag::read_from_path(&file).context("Failed to read tag")?;
    trace!("Tag: {:?}", tag);

    // Create search query
    let title = tag.title().context("Title is missing")?;
    let mut search_query = title.to_string();
    if let Some(artist) = tag.artist() {
        search_query.push_str(&format!("%20artist:{}", artist));
    }
    if let Some(album) = tag.album() {
        search_query.push_str(&format!("%20album:{}", album));
    }
    debug!("Search query: {}", search_query);

    // Search track info
    let req = Client::new()
        .get(URL_SEARCH)
        .bearer_auth(&token)
        .query(&[("q", search_query.as_str()), ("type", "track")]);
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
    trace!("Search response json: {}", res_json);

    // Look for track URL from search result
    info!("Track info search started");
    let tracks = res_json
        .get("tracks")
        .unwrap()
        .get("items")
        .unwrap()
        .as_array()
        .unwrap();
    let track = tracks
        .iter()
        .find(|track| track.get("name").unwrap().as_str().unwrap() == title)
        .context("Not found track from spotify")?;
    let track_url = track
        .get("external_urls")
        .unwrap()
        .get("spotify")
        .unwrap()
        .as_str()
        .unwrap();
    debug!("Track URL: {}", track_url);
    info!("Track info search completed");

    // Start downloading lyrics
    info!("Start downloading lyrics");
    let cmd_res = Command::new("syrics")
        .arg(track_url)
        .arg("--directory")
        .arg(file.parent().unwrap().to_str().unwrap())
        .spawn()
        .unwrap()
        .wait_with_output()
        .await
        .context("Failed to download lyrics")?;
    trace!("Downloading lyrics done");

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
