use std::fmt::Display;

use super::Lyrics;

pub struct Lrc {
    lyrics: Lyrics,
}

impl Lrc {
    pub fn new(lyrics: Lyrics) -> Self {
        Self { lyrics }
    }
}

impl Display for Lrc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for line in self.lyrics.lines.iter() {
            let (min, sec, millis) = calc_readable_time(line.start_time);
            writeln!(
                f,
                "[{: <02}:{: <02}.{: <03}] {}",
                min, sec, millis, line.words
            )?;
        }
        Ok(())
    }
}

fn calc_readable_time(ms: u64) -> (u64, u64, u64) {
    let millis = ms % 1000;
    let sec = ms / 1000 % 60;
    let min = ms / 60000;
    (min, sec, millis)
}
