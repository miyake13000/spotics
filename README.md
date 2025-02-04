# Spotics
Download lyrics (lrc file) from spotify with reading mp3 tag

## Installation

1. [Download from Releases](./releases/latest/)
```bash
wget https://github.com/miyake13000/spotics/releases/latest/download/spotics
chmod +x spotics
```
2. Create app from [Spotify Developers Dashbord](https://developer.spotify.com/dashboard)
3. Get `Client ID` and `Client secret` and write them to `~/.config/spotics/tokens.json`
```json
{
    "id": "0123456789abcdef",
    "secret": "zyxwvuts9876543210"
}
```
4. Get `sp_dc` Cookie from your Browser on Spotify site.
    1. Open Soptify
    2. Enter <F12> to open development tool
    3. Open 'Application' tab
    4. Select 'Cookies' tag
    5. Find `sp_dc` key and copy its value
5. Write `sp_dc` to tokens.json
```json
{
    "id": "0123456789abcdef",
    "secret": "zyxwvuts9876543210",
    "sp_dc": "ABCdefGHIjklMNOpqrSTUvwxZ"
}
```

## Usage
```bash
spotics your_music.mp3
```
* mp3 file must have title, album, artist with id3 v2.x tag
```
$ spotics your_music.mp3

? Select track for Title: 'Your music', Artist: 'You', Album: 'Your best album'
Select one or quit with 'q' ›
› Title: "Your music",  Artist: "You",  Album: "Your best album"

[00:00.500] Once upon a time, there was you
...

? Write above lyrics to specified file? (y/n) › y


$ id3v2 --list your_music.mp3
USLT (Unsynchronized lyric/text transcription): (Episode X)[jpn]: [00:00.500] Once upon a time, there was you
...
```

* Read `--help` to known other option
