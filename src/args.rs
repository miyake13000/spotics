use clap::{Parser, ValueEnum};

#[derive(Debug, Parser)]
#[command(version, author)]
pub struct Args {
    /// Target file
    ///
    /// This file should be audio file that has ID3 tag
    pub file: String,

    /// Track Selection Mode
    #[arg(short = 'm', long, default_value = "manual")]
    pub mode: Mode,

    /// Skip confirmation
    #[arg(short = 'y', long, default_value_t = false)]
    pub yes: bool,

    /// Not print lyrics
    #[arg(short = 's', long, default_value_t = false)]
    pub silent: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Mode {
    /// Select track by user
    Manual,

    /// Select track which has completely same title, artist, album
    Auto,

    /// Select track by Auto mode, if not found, select track with Manual mode
    Middle,
}

#[allow(dead_code)]
impl Mode {
    pub fn is_manual(&self) -> bool {
        *self == Self::Manual
    }

    pub fn is_auto(&self) -> bool {
        *self == Self::Auto
    }

    pub fn is_middle(&self) -> bool {
        *self == Self::Middle
    }
}
