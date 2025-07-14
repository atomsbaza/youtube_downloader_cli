mod domain;
mod application;
mod infrastructure;

use clap::{Parser, ValueEnum};
use application::DownloadVideoUseCase;
use domain::Video;
use infrastructure::YtDlpDownloader;

/// Supported file types for download
#[derive(ValueEnum, Clone, Debug)]
pub enum FileType {
    Mp4,
    Webm,
    Mp3,
    M4a,
    Wav,
}

impl FileType {
    pub fn as_str(&self) -> &'static str {
        match self {
            FileType::Mp4 => "mp4",
            FileType::Webm => "webm",
            FileType::Mp3 => "mp3",
            FileType::M4a => "m4a",
            FileType::Wav => "wav",
        }
    }
}

/// Download YouTube videos via yt-dlp
#[derive(Parser, Debug)]
#[command(name = "ytcli", version = "0.1", author = "คุณ")]
#[command(about = "A simple CLI YouTube downloader using yt-dlp")]
struct Cli {
    /// YouTube video URL
    url: String,

    /// Output filename (without extension)
    #[arg(short, long)]
    output: Option<String>,

    /// Download audio only
    #[arg(short = 'a', long)]
    audio_only: bool,

    /// Video quality (best, worst, 720p, etc.)
    #[arg(short = 'q', long, default_value = "bv*+ba/b")]
    quality: String,

    /// File type/extension to download (mp4, webm, mp3, m4a, wav)
    #[arg(short = 't', long, value_enum)]
    file_type: Option<FileType>,
}

fn main() {
    let cli = Cli::parse();
    let video = Video {
        url: cli.url,
        output: cli.output,
        quality: cli.quality,
        audio_only: cli.audio_only,
        file_type: cli.file_type.map(|ft| ft.as_str().to_string()),
    };
    let downloader = YtDlpDownloader;
    let usecase = DownloadVideoUseCase { downloader: &downloader };
    println!("🎬 Starting download: {}", video.url);
    match usecase.execute(&video) {
        Ok(()) => {
            println!("✅ Download complete!");
            if video.audio_only {
                println!("🎵 Audio extracted to {} format", video.file_type.as_deref().unwrap_or("mp3"));
            }
        },
        Err(e) => {
            eprintln!("❌ Download failed: {}", e);
            eprintln!("💡 Make sure yt-dlp is installed: pip install yt-dlp");
        }
    }
}