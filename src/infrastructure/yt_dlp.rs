use std::io::{BufRead, BufReader, Read};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use crate::domain::{Downloader, Video};

pub struct YtDlpDownloader;

/// Progress snapshot reported by download_with_progress callback
#[derive(Debug, Clone)]
pub struct DownloadProgress {
    /// 0.0 to 1.0
    pub percent: f32,
    /// e.g. "1.23MiB/s" — empty string if not yet available
    pub speed: String,
    /// e.g. "00:30" or "Unknown" — empty string if not yet available
    pub eta: String,
}

/// Build a yt-dlp Command from a Video, shared by download() and download_with_progress()
fn build_yt_dlp_command(video: &Video) -> Result<Command, String> {
    let mut cmd = Command::new("yt-dlp");
    cmd.arg(&video.url);

    if video.audio_only {
        let audio_format = match video.file_type.as_deref().unwrap_or("mp3") {
            f @ ("mp3" | "m4a" | "wav") => f,
            other => return Err(format!(
                "Audio format '{}' is not supported. Use mp3, m4a, or wav.", other
            )),
        };
        cmd.args(["-f", "ba/b"]);
        cmd.args(["--extract-audio", "--audio-format", audio_format]);
    } else {
        let format = build_format_string(
            video.file_type.as_deref().unwrap_or("mp4"),
            &video.quality,
        )?;
        cmd.args(["-f", &format]);
    }

    // Output path: combine save_dir and output filename when both present
    match (&video.save_dir, &video.output) {
        (Some(dir), Some(name)) => {
            cmd.args(["-o", &format!("{}/{}.%(ext)s", dir, name)]);
        }
        (Some(dir), None) => {
            cmd.args(["-P", dir]);
        }
        (None, Some(name)) => {
            cmd.args(["-o", &format!("{}.%(ext)s", name)]);
        }
        (None, None) => {}
    }

    // Playlist options: prefer Video fields, fall back to env vars
    let playlist_items = video.playlist_items.clone().or_else(|| {
        std::env::var("YTCLI_PLAYLIST_ITEMS").ok().filter(|s| !s.is_empty())
    });
    if let Some(items) = playlist_items {
        cmd.args(["--playlist-items", &items]);
    }

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
    if let Ok(ignore) = std::env::var("YTCLI_IGNORE_ERRORS") {
        if ignore == "1" {
            cmd.arg("--ignore-errors");
        }
    }

    Ok(cmd)
}

/// Build a yt-dlp format selector string incorporating quality and container
pub(crate) fn build_format_string(file_type: &str, quality: &str) -> Result<String, String> {
    let height_filter = if quality.ends_with('p') {
        match quality.trim_end_matches('p').parse::<u32>() {
            Ok(h) => format!("[height<={}]", h),
            Err(_) => String::new(),
        }
    } else {
        String::new()
    };

    match file_type {
        "mp4" => {
            if quality == "worst" {
                Ok("worstvideo[ext=mp4]+worstaudio[ext=m4a]/worst[ext=mp4]".to_string())
            } else {
                Ok(format!(
                    "bestvideo{}[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]",
                    height_filter
                ))
            }
        }
        "webm" => {
            if quality == "worst" {
                Ok("worstvideo[ext=webm]+worstaudio[ext=webm]/worst[ext=webm]".to_string())
            } else {
                Ok(format!(
                    "bestvideo{}[ext=webm]+bestaudio[ext=webm]/best[ext=webm]",
                    height_filter
                ))
            }
        }
        other => Err(format!(
            "Video format '{}' is not supported. Use mp4 or webm.", other
        )),
    }
}

/// Parse percentage from a yt-dlp progress line.
/// Example: `[download]  12.3% of ~45.00MiB at 1.23MiB/s ETA 00:30`
pub(crate) fn parse_yt_dlp_progress(line: &str) -> Option<f32> {
    let trimmed = line.trim();
    let idx = trimmed.find('%')?;
    let start = trimmed[..idx].rfind(' ')?;
    trimmed[start..idx].trim().parse::<f32>().ok()
}

/// Parse a full DownloadProgress snapshot from a yt-dlp output line.
/// Returns None if the line is not a download progress line.
pub(crate) fn parse_yt_dlp_line(line: &str) -> Option<DownloadProgress> {
    let percent = parse_yt_dlp_progress(line)?;

    // Speed: "at <value>" — e.g. "at 1.23MiB/s" or "at  1.23MiB/s"
    let speed = line.find(" at ")
        .map(|i| line[i + 4..].split_whitespace().next().unwrap_or("").to_string())
        .unwrap_or_default();

    // ETA: "ETA <value>" — e.g. "ETA 00:30" or "ETA Unknown"
    let eta = line.find("ETA ")
        .map(|i| line[i + 4..].split_whitespace().next().unwrap_or("").to_string())
        .unwrap_or_default();

    Some(DownloadProgress { percent: percent / 100.0, speed, eta })
}

impl YtDlpDownloader {
    /// Download with progress callbacks.
    /// `cancel` — set to true from any thread to abort; the child process is killed and
    /// the function returns `Err("Download cancelled")`.
    /// `progress_callback` — called after each stdout line that contains progress.
    pub fn download_with_progress<F>(
        &self,
        video: &Video,
        cancel: Arc<AtomicBool>,
        mut progress_callback: F,
    ) -> Result<(), String>
    where
        F: FnMut(DownloadProgress) + Send + 'static,
    {
        let mut cmd = build_yt_dlp_command(video)?;
        cmd.arg("--newline");
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| e.to_string())?;
        let stdout = child.stdout.take()
            .ok_or_else(|| "failed to capture stdout from yt-dlp".to_string())?;
        let stderr = child.stderr.take()
            .ok_or_else(|| "failed to capture stderr from yt-dlp".to_string())?;

        // Read stderr on a separate thread so it doesn't block stdout
        let stderr_handle = std::thread::spawn(move || {
            let mut s = String::new();
            BufReader::new(stderr).read_to_string(&mut s).ok();
            s
        });

        let reader = BufReader::new(stdout);
        for line in reader.lines().flatten() {
            if let Some(progress) = parse_yt_dlp_line(&line) {
                progress_callback(progress);
            }
            // Check cancellation after each line so the process is killed promptly
            if cancel.load(Ordering::Relaxed) {
                let _ = child.kill();
                break;
            }
        }

        let status = child.wait().map_err(|e| e.to_string())?;
        let stderr_output = stderr_handle.join().unwrap_or_default();

        if cancel.load(Ordering::Relaxed) {
            return Err("Download cancelled".to_string());
        }

        if status.success() {
            progress_callback(DownloadProgress { percent: 1.0, speed: String::new(), eta: String::new() });
            Ok(())
        } else {
            let base = format!("yt-dlp exited with status: {}", status);
            if stderr_output.trim().is_empty() {
                Err(base)
            } else {
                Err(format!("{}\n{}", base, stderr_output.trim()))
            }
        }
    }
}

impl Downloader for YtDlpDownloader {
    fn download(&self, video: &Video) -> Result<(), String> {
        let mut cmd = build_yt_dlp_command(video)?;
        let status = cmd.status().map_err(|e| e.to_string())?;
        if status.success() {
            println!("✅ All videos downloaded successfully!");
            Ok(())
        } else if let Some(code) = status.code() {
            if code == 1 {
                println!("⚠️  Some videos failed to download, but others may have succeeded.");
                Err("yt-dlp exited with status 1 (partial failure)".to_string())
            } else {
                println!("❌ Download failed (exit code {}).", code);
                Err(format!("yt-dlp exited with status: {}", code))
            }
        } else {
            println!("❌ Download failed: yt-dlp process terminated by signal.");
            Err("yt-dlp process terminated by signal".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- parse_yt_dlp_progress ---

    #[test]
    fn test_parse_progress_normal() {
        assert_eq!(parse_yt_dlp_progress("[download]  12.3% of ~45.00MiB"), Some(12.3));
    }

    #[test]
    fn test_parse_progress_100() {
        assert_eq!(parse_yt_dlp_progress("[download] 100% of ~45.00MiB"), Some(100.0));
    }

    #[test]
    fn test_parse_progress_leading_spaces() {
        assert_eq!(parse_yt_dlp_progress("[download]   5.3% of 100MiB at 1MiB/s"), Some(5.3));
    }

    #[test]
    fn test_parse_progress_no_percent() {
        assert_eq!(parse_yt_dlp_progress("[download] Destination: video.mp4"), None);
    }

    #[test]
    fn test_parse_progress_empty_line() {
        assert_eq!(parse_yt_dlp_progress(""), None);
    }

    #[test]
    fn test_parse_progress_other_line() {
        assert_eq!(parse_yt_dlp_progress("[info] Writing video description"), None);
    }

    // --- parse_yt_dlp_line ---

    #[test]
    fn test_parse_line_full() {
        let p = parse_yt_dlp_line("[download]  12.3% of ~45.00MiB at 1.23MiB/s ETA 00:30").unwrap();
        assert!((p.percent - 0.123).abs() < 0.001);
        assert_eq!(p.speed, "1.23MiB/s");
        assert_eq!(p.eta, "00:30");
    }

    #[test]
    fn test_parse_line_percent_only() {
        let p = parse_yt_dlp_line("[download]  50.0% of ~45.00MiB").unwrap();
        assert!((p.percent - 0.5).abs() < 0.001);
        assert_eq!(p.speed, "");
        assert_eq!(p.eta, "");
    }

    #[test]
    fn test_parse_line_100_percent() {
        let p = parse_yt_dlp_line("[download] 100% of ~45.00MiB at 3.45MiB/s ETA 00:00").unwrap();
        assert!((p.percent - 1.0).abs() < 0.001);
        assert_eq!(p.speed, "3.45MiB/s");
        assert_eq!(p.eta, "00:00");
    }

    #[test]
    fn test_parse_line_unknown_eta() {
        let p = parse_yt_dlp_line("[download]   0.1% of ~45.00MiB at 123.45KiB/s ETA Unknown").unwrap();
        assert_eq!(p.eta, "Unknown");
        assert_eq!(p.speed, "123.45KiB/s");
    }

    #[test]
    fn test_parse_line_non_progress() {
        assert!(parse_yt_dlp_line("[info] Writing video description").is_none());
        assert!(parse_yt_dlp_line("").is_none());
    }

    // --- build_format_string ---

    #[test]
    fn test_format_mp4_best() {
        let fmt = build_format_string("mp4", "bv*+ba/b").unwrap();
        assert!(fmt.contains("bestvideo"));
        assert!(fmt.contains("mp4"));
        assert!(!fmt.contains("height"));
    }

    #[test]
    fn test_format_mp4_720p() {
        let fmt = build_format_string("mp4", "720p").unwrap();
        assert!(fmt.contains("[height<=720]"));
        assert!(fmt.contains("mp4"));
    }

    #[test]
    fn test_format_mp4_480p() {
        let fmt = build_format_string("mp4", "480p").unwrap();
        assert!(fmt.contains("[height<=480]"));
    }

    #[test]
    fn test_format_mp4_worst() {
        let fmt = build_format_string("mp4", "worst").unwrap();
        assert!(fmt.contains("worstvideo"));
        assert!(fmt.contains("mp4"));
    }

    #[test]
    fn test_format_webm_best() {
        let fmt = build_format_string("webm", "bv*+ba/b").unwrap();
        assert!(fmt.contains("bestvideo"));
        assert!(fmt.contains("webm"));
    }

    #[test]
    fn test_format_webm_worst() {
        let fmt = build_format_string("webm", "worst").unwrap();
        assert!(fmt.contains("worstvideo"));
        assert!(fmt.contains("webm"));
    }

    #[test]
    fn test_format_invalid_type() {
        assert!(build_format_string("avi", "best").is_err());
        assert!(build_format_string("mkv", "720p").is_err());
    }
}
