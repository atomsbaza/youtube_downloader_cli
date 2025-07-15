# youtube_downloader_cli

A simple, extensible YouTube downloader CLI written in Rust, using [yt-dlp](https://github.com/yt-dlp/yt-dlp) under the hood. Built with Clean Architecture for maintainability and testability.

> **Releases:**
> Official releases and binaries are available on the [GitHub Releases page](https://github.com/<your-username>/youtube_downloader_cli/releases).

## Downloading and Using Release Binaries

Pre-built binaries for Linux, macOS, and Windows are available on the [GitHub Releases page](https://github.com/<your-username>/youtube_downloader_cli/releases) for each tagged release.

### 1. Download the Binary
Go to the [Releases page](https://github.com/<your-username>/youtube_downloader_cli/releases) and download the appropriate file for your platform:

| Platform | File Name            |
|----------|---------------------|
| Linux    | ytcli-linux         |
| macOS    | ytcli-macos         |
| Windows  | ytcli-windows.exe   |

### 2. Make the Binary Executable (Linux/macOS)
After downloading, you may need to give the binary execute permissions:
```sh
chmod +x ytcli-linux   # or ytcli-macos
```

### 3. Run the CLI
**Linux/macOS:**
```sh
./ytcli-linux "https://www.youtube.com/watch?v=..."
# or
./ytcli-macos "https://www.youtube.com/watch?v=..."
```

**Windows:**
```sh
ytcli-windows.exe "https://www.youtube.com/watch?v=..."
```

You can use all the same options and environment variables as described below.

## Features
- Download YouTube videos or extract audio as MP3, M4A, or WAV
- Choose video quality (e.g., 720p, best, worst)
- Specify output filename
- Select file type/extension to download (mp4, webm, mp3, m4a, wav)
- **Download entire playlists or specific items from playlists**
- Easily extensible for other downloaders or platforms

## Usage
Make sure you have [yt-dlp](https://github.com/yt-dlp/yt-dlp) installed and available in your PATH.

```sh
cargo run -- <URL> [OPTIONS]
```

**Options:**
- `-o`, `--output <output>`: Output filename (without extension)
- `-a`, `--audio-only`: Download audio only (as MP3, M4A, or WAV)
- `-q`, `--quality <quality>`: Video quality (default: `bv*+ba/b`)
- `-t`, `--file-type <type>`: File type/extension to download (mp4, webm, mp3, m4a, wav)

**Playlist and Error Handling Options (set via environment variables):**
- `YTCLI_PLAYLIST_START`: Download only from this index (1-based) in a playlist
- `YTCLI_PLAYLIST_END`: Download up to this index (inclusive, 1-based) in a playlist
- `YTCLI_PLAYLIST_ITEMS`: Comma-separated indices or ranges (e.g. `1,3,5-7`) to download specific items from a playlist
- `YTCLI_IGNORE_ERRORS`: Set to `1` to continue downloading even if some videos fail

**Examples:**
```sh
# Download an entire playlist as mp3
cargo run -- "https://www.youtube.com/playlist?list=..." -a -t mp3

# Download only the 2nd to 5th videos in a playlist as mp4
YTCLI_PLAYLIST_START=2 YTCLI_PLAYLIST_END=5 cargo run -- "https://www.youtube.com/playlist?list=..." -t mp4

# Download specific items (1, 3, 5-7) from a playlist as webm
YTCLI_PLAYLIST_ITEMS=1,3,5-7 cargo run -- "https://www.youtube.com/playlist?list=..." -t webm

# Download a playlist and ignore errors (continue even if some videos fail)
YTCLI_IGNORE_ERRORS=1 cargo run -- "https://www.youtube.com/playlist?list=..."

# Combine options: download items 2-7 and ignore errors
YTCLI_PLAYLIST_START=2 YTCLI_PLAYLIST_END=7 YTCLI_IGNORE_ERRORS=1 cargo run -- "https://www.youtube.com/playlist?list=..."
```

## Supported File Types

You can use the `-t` or `--file-type` option to select the file type/extension to download. The following types are supported:

| Type   | Audio Only (`-a`) | Video (default)      |
|--------|-------------------|----------------------|
| mp3    | ✅                | ❌ (not video)       |
| m4a    | ✅                | ❌ (not video)       |
| wav    | ✅                | ❌ (not video)       |
| mp4    | ❌                | ✅                   |
| webm   | ❌                | ✅                   |

- For **audio-only** downloads, you can specify only `mp3`, `m4a`, or `wav`.
- For **video** downloads, you can specify only `mp4` or `webm`.
- If you do **not** specify `-t`, the downloader will use the default yt-dlp quality selection, usually giving you the best available video and audio merged into the most common format (often `webm` or `mp4`).

## Playlist Support

- You can pass a playlist URL to download all videos in the playlist.
- Use the environment variables `YTCLI_PLAYLIST_START`, `YTCLI_PLAYLIST_END`, or `YTCLI_PLAYLIST_ITEMS` to control which videos are downloaded from the playlist.
- Set `YTCLI_IGNORE_ERRORS=1` to continue downloading even if some videos fail.
- Output files will be named automatically by yt-dlp to avoid overwriting.

## Troubleshooting: MP4 Files Fail to Open

If you find that downloaded MP4 files fail to open or play, consider the following:

- **Incomplete or Corrupted Download:**
  - The download may have been interrupted. Try re-downloading the file.
- **ffmpeg Issues:**
  - yt-dlp uses [ffmpeg](https://ffmpeg.org/) to merge video and audio. Make sure ffmpeg is installed and up to date.
- **Codec or Format Compatibility:**
  - Some MP4s may use codecs not supported by all players. Try playing the file in VLC, or re-encode it with ffmpeg:
    ```sh
    ffmpeg -i input.mp4 -c:v libx264 -c:a aac output_compatible.mp4
    ```
- **File Extension Mismatch:**
  - Sometimes a file may be saved as .mp4 but is actually another format. Check with `ffprobe` or `mediainfo`.
- **Partial Downloads:**
  - Unstable internet can cause incomplete files. Delete and try again.
- **DRM or Age Restrictions:**
  - Some videos are protected or restricted and may not download properly.

**General Tips:**
- Always use the latest version of yt-dlp and ffmpeg.
- Check the command output for errors or warnings.
- If you continue to have issues, try with a different video or format.

## Project Structure (Clean Architecture)

```
youtube_downloader_cli/
├── src/
│   ├── main.rs            # CLI entry point (Presentation layer)
│   ├── domain/            # Core business logic (entities, traits)
│   │   └── mod.rs
│   ├── application/       # Use cases (application services)
│   │   └── mod.rs
│   └── infrastructure/    # External implementations (yt-dlp)
│       ├── mod.rs
│       └── yt_dlp.rs
```

- **domain/**: Defines the `Video` entity and `Downloader` trait.
- **application/**: Contains the `DownloadVideoUseCase` orchestrating downloads.
- **infrastructure/**: Implements the `Downloader` trait using `yt-dlp`.
- **main.rs**: Handles CLI parsing and wires everything together.

## Extending
To add support for another video platform, implement the `Downloader` trait in `infrastructure/` and use it in the application layer.

## Requirements
- Rust (edition 2021 recommended)
- [yt-dlp](https://github.com/yt-dlp/yt-dlp) installed (`pip install yt-dlp`)
- [ffmpeg](https://ffmpeg.org/) installed and available in your PATH

## License
MIT

## Releasing

To publish a new release of this project on GitHub:

1. **Prepare your project:**
   - Ensure all code is committed and pushed to your main branch.
   - Update `README.md` and `Cargo.toml` as needed.

2. **Tag a release in git:**
   ```sh
   git tag v1.0.0
   git push origin v1.0.0
   ```
   Replace `v1.0.0` with your desired version.

3. **Create a GitHub Release:**
   - Go to your repository on GitHub.
   - Click the **"Releases"** tab.
   - Click **"Draft a new release"**.
   - Select the tag you just pushed (e.g., `v1.0.0`).
   - Fill in the release title and description (see below for a sample).
   - (Optional) Attach pre-built binaries (see next step).
   - Click **"Publish release"**.

4. **(Optional) Attach Pre-built Binaries:**
   - Build your binary:
     ```sh
     cargo build --release
     ```
   - The binary will be in `target/release/ytcli` (or your binary name).
   - Upload this file to your GitHub release for easy download.

### Sample Release Description

```
v1.0.0
- Initial public release
- Download YouTube videos and playlists
- Audio/video format selection
- Playlist range/item selection via environment variables
- Improved error handling
```
