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
        let status = cmd.status().map_err(|e| e.to_string())?;
        if status.success() {
            Ok(())
        } else {
            Err(format!("yt-dlp exited with status: {}", status))
        }
    }
} 