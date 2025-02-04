mod args;

use anyhow::{Context, Result};
use args::Args;
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use id3::{frame::Lyrics, Tag, TagLike};
use log::debug;
use spotics::{ClientBuilder, Lrc, SearchQuery, TrackInfo};
use std::path::PathBuf;

const PATH_TOKEN: &str = ".config/spotics";
const PATH_CACHE: &str = ".cache/spotics";

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    env_logger::init();
    debug!("{:?}", args);

    // Get token and cache path
    let homedir = std::env::var("HOME").context("Env var 'HOME' is not set")?;
    let homedir = PathBuf::from(homedir);
    let token_path = homedir.join(PATH_TOKEN);
    let cache_path = homedir.join(PATH_CACHE);
    debug!("Token path: {:?}", token_path);
    debug!("Cache path: {:?}", token_path);

    // Create client
    let client = ClientBuilder::new(token_path)
        .use_cache(cache_path)
        .build()
        .await?;
    debug!("Client: {:?}", client);

    // Get tag from specified file
    let tag = Tag::read_from_path(&args.file)
        .with_context(|| format!("Failed to read specified file: {}", args.file))?;

    // Search track
    let query = create_search_query(&tag)?;
    debug!("Query: {:?}", query);
    let tracks = client.search(query).await?;
    debug!("Tracks: {:?}", tracks);

    // Select track
    let track = if args.mode.is_manual() {
        select_track_by_user(&tracks, &tag)?
    } else {
        let track = select_track_by_identity(&tracks, &tag);
        if track.is_some() {
            println!("Selected track: {:?}", track.as_ref().unwrap());
            track
        } else if args.mode.is_middle() {
            select_track_by_user(&tracks, &tag)?
        } else {
            None
        }
    };
    if track.is_none() {
        println!("Track not found");
        return Ok(());
    }
    let track = track.unwrap();
    debug!("Selected track: {:?}", track);

    // Fetch lyrics
    let lyrics = client.fetch_lyrics(track).await?;
    if !args.silent {
        println!("{}", lyrics);
    }
    if !args.yes && !confirm_write_lyrics()? {
        println!("Aborted");
        return Ok(());
    }
    debug!("Lyrics: {}", lyrics);

    // Write lyrics to file
    let tag = add_lyrics_to_tag(lyrics, tag);
    write_lyrics(tag, &args.file)?;

    Ok(())
}

fn create_search_query(tag: &Tag) -> Result<SearchQuery> {
    let title = tag.title().context("Not found 'title' tag")?;
    let artist = tag.artist().context("Not found 'artist' tag")?;
    let album = tag.album().context("Not found 'album' tag")?;

    Ok(SearchQuery::new(title, artist, album))
}

fn select_track_by_user<'a>(tracks: &'a [TrackInfo], tag: &Tag) -> Result<Option<&'a TrackInfo>> {
    let prompt = format!(
        "Select track for Title: '{}', Artist: '{}', Album: '{}'\n",
        tag.title().unwrap(),
        tag.artist().unwrap(),
        tag.album().unwrap()
    );
    let prompt = prompt + "Select one or quit with 'q'";
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(tracks)
        .interact_opt()
        .context("Failed to select track")?;

    Ok(selection.map(|i| &tracks[i]))
}

fn select_track_by_identity<'a>(tracks: &'a [TrackInfo], tag: &Tag) -> Option<&'a TrackInfo> {
    let title = tag.title().unwrap();
    let artist = tag.artist().unwrap();
    let album = tag.album().unwrap();

    let track = tracks
        .iter()
        .find(|track| track.title == title && track.artist == artist && track.album == album);

    track
}

fn confirm_write_lyrics() -> Result<bool> {
    let confirmation = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Write above lyrics to specified file?")
        .interact()
        .context("Failed to confirm")?;
    Ok(confirmation)
}

fn add_lyrics_to_tag(lyrics: Lrc, mut tag: Tag) -> Tag {
    let lyrics_frame = Lyrics {
        lang: "jpn".to_string(),
        description: tag.title().unwrap().to_string(),
        text: lyrics.to_string(),
    };
    tag.add_frame(lyrics_frame);

    tag
}

fn write_lyrics(tag: Tag, path: &String) -> Result<()> {
    let version = tag.version();
    tag.write_to_path(path, version)
        .context("Failed to write tag")?;

    Ok(())
}
