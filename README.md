# Spotics
Download lyrics (lrc file) from spotify with reading mp3 tag

## Installation

1. [Download from Releases](./releases/latest/)
1. Create app from [Spotify Developers Dashbord](https://developer.spotify.com/dashboard)
3. Get `Client ID` and `Client secret` and write them to `~/.config/spotics/tokens.json`
```json
{
    "id": "0123456789abcdef",
    "secret": "zyxwvuts9876543210"
}
```

## Usage
```bash
spotics your_music.mp3
```
* mp3 file must have title, album, artist with id3 v2.x tag

And you can embed lyrics to tag
```bash
eyeD3 --add-lyrics your_music.lrc your_music.mp3
```

> [!CAUTION]
> Now use external Python program [syrics](https://github.com/akashrchandran/syrics) for download lyrics
> This will change for the future to pure Rust program
