# youtube_downloader_cli

A simple, extensible YouTube downloader CLI written in Rust, using [yt-dlp](https://github.com/yt-dlp/yt-dlp) under the hood. Built with Clean Architecture for maintainability and testability.

## Features
- Download YouTube videos or extract audio as MP3, M4A, or WAV
- Choose video quality (e.g., 720p, best, worst)
- Specify output filename
- Select file type/extension to download (mp4, webm, mp3, m4a, wav)
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

**Example:**
```sh
cargo run -- https://www.youtube.com/watch?v=dQw4w9WgXcQ -a -o my_audio -t mp3
cargo run -- https://www.youtube.com/watch?v=dQw4w9WgXcQ -t mp4
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
