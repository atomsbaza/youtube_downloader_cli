use std::process::Command;
use crate::domain::{Downloader, Video};

pub struct YtDlpDownloader;

impl Downloader for YtDlpDownloader {
    fn download(&self, video: &Video) -> Result<(), String> {
        let mut cmd = Command::new("yt-dlp");
        cmd.arg(&video.url);
        if video.audio_only {
            // Only allow mp3, m4a, wav
            let audio_format = match video.file_type.as_deref().unwrap_or("mp3") {
                "mp3" | "m4a" | "wav" => video.file_type.as_deref().unwrap_or("mp3"),
                other => return Err(format!("Audio format '{}' is not supported. Use mp3, m4a, or wav.", other)),
            };
            cmd.args(["-f", "ba/b"]);
            cmd.args(["--extract-audio", "--audio-format", audio_format]);
        } else {
            // Only allow mp4, webm
            let format = match video.file_type.as_deref().unwrap_or("mp4") {
                "mp4" => "bestvideo[ext=mp4]+bestaudio[ext=m4a]/mp4".to_string(),
                "webm" => "bestvideo[ext=webm]+bestaudio[ext=webm]/webm".to_string(),
                other => return Err(format!("Video format '{}' is not supported. Use mp4 or webm.", other)),
            };
            cmd.args(["-f", &format]);
        }
        if let Some(out) = &video.output {
            cmd.args(["-o", &format!("{}.%(ext)s", out)]);
        }
        // Playlist options from environment variables
        if let Ok(start) = std::env::var("YTCLI_PLAYLIST_START") {
            if !start.is_empty() {
                cmd.args(["--playlist-start", &start]);
            }
        }
        if let Ok(end) = std::env::var("YTCLI_PLAYLIST_END") {
            if !end.is_empty() {
                cmd.args(["--playlist-end", &end]);
            }
        }
        if let Ok(items) = std::env::var("YTCLI_PLAYLIST_ITEMS") {
            if !items.is_empty() {
                cmd.args(["--playlist-items", &items]);
            }
        }
        // Optionally add --ignore-errors if env var is set
        if let Ok(ignore) = std::env::var("YTCLI_IGNORE_ERRORS") {
            if ignore == "1" {
                cmd.arg("--ignore-errors");
            }
        }
        let status = cmd.status().map_err(|e| e.to_string())?;
        if status.success() {
            println!("✅ All videos downloaded successfully!");
            Ok(())
        } else if let Some(code) = status.code() {
            if code == 1 {
                println!("⚠️  Some videos failed to download, but others may have succeeded. Please check your output directory.");
                Err(format!("yt-dlp exited with status 1 (partial failure)"))
            } else {
                println!("❌ Download failed due to a critical error (exit code {}).", code);
                Err(format!("yt-dlp exited with status: {}", code))
            }
        } else {
            println!("❌ Download failed: yt-dlp process terminated by signal.");
            Err("yt-dlp process terminated by signal".to_string())
        }
    }
} 