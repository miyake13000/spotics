use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, author)]
pub struct Args {
    /// Target file
    ///
    /// This file should be audio file that has ID3 tag
    pub music_file: String,

    /// Read Spotify Lyric JSON from file
    ///
    /// For mare details, see https://github.com/miyake13000/spotics
    /// If this flag and `--stdin` are both specified, this flag will be ignored
    #[arg(short = 'f', long)]
    pub lyric_file: Option<String>,

    /// Read Spotify Lyric JSON from STDIN
    ///
    /// If this flag and `--lyric-file` are both specified, `--lyric-file` will be ignored
    #[arg(short = 'i', long)]
    pub stdin: bool,

    /// Skip confirmation for writing lyc to file
    #[arg(short = 'y', long, default_value_t = false)]
    pub yes: bool,

    /// Not print converted LRC
    #[arg(short = 's', long, default_value_t = false)]
    pub silent: bool,
}
