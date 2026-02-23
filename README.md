# Spotics
Embed Lrics (LRC format) from spotify

## Changes
* It is very hard to get lyrics from spotify in this program due to spotify's highly complexed authentication process
* For more datails, read [this discussion](https://github.com/librespot-org/librespot/discussions/1562)
* From these reasons, I deleted the function to get lyrics.
* In the future, I want to try scraping using Selemium...

## Installation

1. [Download from Releases](./releases/latest/)
```bash
wget https://github.com/miyake13000/spotics/releases/latest/download/spotics
chmod +x spotics
```

## Usage
1. Download Spotify Lyric JSON
    1. Access [Spotify](https://open.spotify.com/)
    2. Open developer tool with F12 and click "Network" tab
    3. Enter "color" to "Filter" input area
    4. Search and open the music you want to embed the lyric
    5. After the lyric appeared, click and open JSON response in "network" tab
    6. Download or copy that JSON.
    * The JSON is like:
    ```json
    {"lyrics":{"syncType":"LINE_SYNCED","lines":[{"startTimeMs":"18404","words":"blahblahblah","syllables":[],"endTimeMs":"0","transliteratedWords":""},...
    ```

2. Embed Lyric to mp3 file
```bash
# Read Spotify Lyric JSON from file
spotics -f spotify_lyric.json your_music.mp3

# Or, from STDIN (This is usefull to paste from clipboard)
cat spotidy_lyric.json | spotics -i your_music.mp3
```

* mp3 file must have id3 v2.x tag
```
$ spotics -f spotiy_lyric.json your_music.mp3
[00:00.500] Once upon a time, there was you
...

? Write above lyrics to specified file? (y/n) › y


$ id3v2 --list your_music.mp3
USLT (Unsynchronized lyric/text transcription): (Episode X)[jpn]: [00:00.500] Once upon a time, there was you
...
```

* Read `--help` to known other option
