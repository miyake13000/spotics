mod args;

use anyhow::{Context, Result};
use args::Args;
use clap::Parser;
use dialoguer::{theme::ColorfulTheme, Confirm};
use id3::{frame::Lyrics, Tag, TagLike};
use log::debug;
use spotics::{Lrc, SpotifyLyric};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    env_logger::init();
    debug!("{:?}", args);

    // Get tag from specified file
    let tag = Tag::read_from_path(&args.music_file)
        .with_context(|| format!("Failed to read specified file: {}", args.music_file))?;

    // Get lyrics from specified source
    let lyric_str = if args.stdin {
        debug!("Reading lyric JSON from STDIN");
        let mut buf = String::new();
        std::io::stdin()
            .read_line(&mut buf)
            .context("Failed to read from STDIN")?;
        buf
    } else if let Some(lyric_file) = args.lyric_file {
        debug!("Reading lyric JSON from file: {}", lyric_file);
        std::fs::read_to_string(lyric_file).context("Failed to read lyric file")?
    } else {
        return Err(anyhow::anyhow!(
            "No lyric source specified. Please read usage with `-h`"
        ));
    };

    // Convert lyric JSON to LRC
    let lyric_json: serde_json::Value =
        serde_json::from_str(&lyric_str).context("Failed to parse lyric JSON")?;
    let spotify_lyric =
        SpotifyLyric::try_from(&lyric_json).context("Failed to parse lyric JSON")?;
    let lyrics = Lrc::new(spotify_lyric);
    debug!("Lyrics: {}", lyrics);

    // Print converted LRC and confirm writing to file
    if !args.silent {
        println!("{}", lyrics);
    }
    if !args.yes && !confirm_write_lyrics()? {
        println!("Aborted");
        return Ok(());
    }

    // Write lyrics to file
    let tag = add_lyrics_to_tag(lyrics, tag);
    write_lyrics(tag, &args.music_file)?;

    Ok(())
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
